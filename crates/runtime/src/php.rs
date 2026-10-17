use crate::error::{Result, RuntimeError};
use localforge_core::ProjectId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;

pub struct PhpFpmSupervisor {
    run_dir: PathBuf,
    active_processes: Mutex<HashMap<String, Child>>,
}

impl PhpFpmSupervisor {
    pub fn new(run_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&run_dir);
        Self {
            run_dir,
            active_processes: Mutex::new(HashMap::new()),
        }
    }

    /// Discovers all available installed PHP versions on the host.
    pub fn discover_installed_versions() -> Vec<String> {
        let mut versions = Vec::new();
        let common = ["8.4", "8.3", "8.2", "8.1", "8.0", "7.4"];
        for v in common {
            if Self::find_php_fpm_binary(v).is_ok() {
                versions.push(v.to_string());
            }
        }
        if versions.is_empty() {
            if let Ok(output) = Command::new("php")
                .arg("-r")
                .arg("echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;")
                .output()
            {
                if output.status.success() {
                    let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !v.is_empty() {
                        versions.push(v);
                    }
                }
            }
        }
        versions
    }

    /// Discovers the path to php-fpm on macOS Apple Silicon.
    pub fn find_php_fpm_binary(version: &str) -> Result<PathBuf> {
        let candidates = vec![
            format!("/opt/homebrew/opt/php@{version}/sbin/php-fpm"),
            format!("/opt/homebrew/bin/php-fpm"),
            format!("/usr/local/opt/php@{version}/sbin/php-fpm"),
            format!("/usr/local/bin/php-fpm"),
        ];

        for c in candidates {
            let p = PathBuf::from(&c);
            if p.exists() {
                return Ok(p);
            }
        }

        // Check if plain php-fpm is in PATH
        if let Ok(output) = Command::new("which").arg("php-fpm").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }
        }

        Err(RuntimeError::RuntimeNotFound {
            binary: "php-fpm".into(),
            version: version.into(),
        })
    }

    /// Returns the dedicated Unix socket path for a project.
    pub fn socket_path(&self, project_id: &ProjectId) -> PathBuf {
        self.run_dir.join(format!("{}.sock", project_id.0))
    }

    /// Generates isolated FPM pool configuration content.
    pub fn generate_pool_config(&self, project_id: &ProjectId, project_dir: &Path) -> String {
        let sock = self.socket_path(project_id);
        format!(
            "[global]\n\
             error_log = {run_dir}/{pid}_error.log\n\
             daemonize = no\n\n\
             [{pid}]\n\
             listen = {sock}\n\
             listen.mode = 0660\n\
             pm = ondemand\n\
             pm.max_children = 5\n\
             pm.process_idle_timeout = 15s\n\
             chdir = {project_dir}\n\
             php_admin_value[error_log] = {run_dir}/{pid}_fpm_errors.log\n\
             php_admin_flag[log_errors] = on\n",
            run_dir = self.run_dir.display(),
            pid = project_id.0,
            sock = sock.display(),
            project_dir = project_dir.display()
        )
    }

    /// Starts or ensures an FPM pool process is running for the given project.
    pub fn start_pool(
        &self,
        project_id: &ProjectId,
        php_version: &str,
        project_dir: &Path,
    ) -> Result<PathBuf> {
        let mut map = self.active_processes.lock().unwrap();

        // Check if already active and alive
        if let Some(child) = map.get_mut(&project_id.0) {
            match child.try_wait() {
                Ok(None) => return Ok(self.socket_path(project_id)),
                _ => {
                    // Process exited; remove from map and restart
                    map.remove(&project_id.0);
                }
            }
        }

        let fpm_bin = Self::find_php_fpm_binary(php_version)?;
        let conf_content = self.generate_pool_config(project_id, project_dir);
        let conf_path = self.run_dir.join(format!("{}.conf", project_id.0));
        std::fs::write(&conf_path, conf_content).map_err(RuntimeError::Io)?;

        let child = Command::new(&fpm_bin)
            .arg("--fpm-config")
            .arg(&conf_path)
            .arg("-F") // Force foreground so supervisor owns the child
            .spawn()
            .map_err(|e| RuntimeError::ProcessSpawnFailed {
                process: format!("php-fpm-{}", project_id.0),
                reason: e.to_string(),
            })?;

        map.insert(project_id.0.clone(), child);
        Ok(self.socket_path(project_id))
    }

    /// Stops a running project FPM pool gracefully.
    pub fn stop_pool(&self, project_id: &ProjectId) -> Result<()> {
        let mut map = self.active_processes.lock().unwrap();
        if let Some(mut child) = map.remove(&project_id.0) {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Clean socket
        let sock = self.socket_path(project_id);
        if sock.exists() {
            let _ = std::fs::remove_file(sock);
        }

        Ok(())
    }
}

impl Drop for PhpFpmSupervisor {
    fn drop(&mut self) {
        if let Ok(mut map) = self.active_processes.lock() {
            for (_, mut child) in map.drain() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pool_config_generation() {
        let dir = tempdir().unwrap();
        let sup = PhpFpmSupervisor::new(dir.path().to_path_buf());
        let pid = ProjectId::from_string("proj_abc");
        let proj_dir = Path::new("/Volumes/Mac Harici Disk/Projects/mytest");

        let conf = sup.generate_pool_config(&pid, proj_dir);
        assert!(conf.contains("[proj_abc]"));
        assert!(conf.contains("proj_abc.sock"));
        assert!(conf.contains("pm = ondemand"));
    }
}
