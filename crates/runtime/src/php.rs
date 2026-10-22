use crate::error::{Result, RuntimeError};
use crate::health::is_unix_socket_alive;
use localforge_core::ProjectId;
use localforge_platform::{ProcessIdentity, ProcessManager};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct ManagedPool {
    pub identity: ProcessIdentity,
    pub socket_path: PathBuf,
    pub conf_path: PathBuf,
    pub child: Option<Child>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhpVersionDetail {
    pub version: String,
    pub major_minor: String,
    pub binary_path: String,
    pub is_fpm_verified: bool,
    pub architecture: String,
    pub is_supported: bool,
}

pub struct PhpFpmSupervisor {
    run_dir: PathBuf,
    process_manager: Arc<dyn ProcessManager>,
    active_pools: Mutex<HashMap<String, ManagedPool>>,
    pub terminate_on_drop: bool,
}

impl PhpFpmSupervisor {
    pub fn new(run_dir: PathBuf, process_manager: Arc<dyn ProcessManager>) -> Self {
        let _ = std::fs::create_dir_all(&run_dir);
        Self {
            run_dir,
            process_manager,
            active_pools: Mutex::new(HashMap::new()),
            terminate_on_drop: false,
        }
    }

    /// Discovers all available installed PHP versions with verified FPM binary and architecture telemetry.
    pub fn discover_installed_details() -> Vec<PhpVersionDetail> {
        let mut details = Vec::new();
        let common = ["8.5", "8.4", "8.3", "8.2", "8.1"];
        for v in common {
            if let Ok((bin_path, actual_v)) = Self::find_php_fpm_binary(v) {
                let bin_str = bin_path.display().to_string();
                if details
                    .iter()
                    .any(|d: &PhpVersionDetail| d.binary_path == bin_str)
                {
                    continue;
                }
                let is_fpm = Self::is_fpm_binary(&bin_path);
                let arch = Self::detect_architecture(&bin_path);
                let major_minor = if let Some((maj, rest)) = actual_v.split_once('.') {
                    if let Some((min_only, _)) = rest.split_once('.') {
                        format!("{maj}.{min_only}")
                    } else {
                        format!("{maj}.{rest}")
                    }
                } else {
                    actual_v.clone()
                };
                let is_supported =
                    ["8.5", "8.4", "8.3", "8.2", "8.1"].contains(&major_minor.as_str());

                details.push(PhpVersionDetail {
                    version: actual_v,
                    major_minor,
                    binary_path: bin_str,
                    is_fpm_verified: is_fpm,
                    architecture: arch,
                    is_supported,
                });
            }
        }
        details
    }

    pub fn is_fpm_binary(bin_path: &Path) -> bool {
        if let Ok(output) = Command::new(bin_path).arg("-v").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains("fpm-fcgi");
        }
        false
    }

    pub fn detect_architecture(bin_path: &Path) -> String {
        if let Ok(output) = Command::new("file").arg(bin_path).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("arm64") {
                return "arm64".to_string();
            } else if stdout.contains("x86_64") {
                return "x86_64".to_string();
            }
        }
        std::env::consts::ARCH.to_string()
    }

    /// Discovers all available installed PHP versions on the host.
    pub fn discover_installed_versions() -> Vec<String> {
        let details = Self::discover_installed_details();
        let mut versions = Vec::new();
        for d in details {
            if !versions.contains(&d.major_minor) {
                versions.push(d.major_minor.clone());
            }
            if !versions.contains(&d.version) {
                versions.push(d.version);
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

    /// Parses the exact PHP version from `php-fpm -v` output.
    pub fn extract_version_from_binary(bin_path: &Path) -> Option<String> {
        let output = Command::new(bin_path).arg("-v").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: "PHP 8.5.3 (fpm-fcgi) ..."
        for line in stdout.lines() {
            if line.starts_with("PHP ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string());
                }
            }
        }
        None
    }

    /// Discovers the path and validates version to php-fpm on macOS Apple Silicon.
    pub fn find_php_fpm_binary(requested_version: &str) -> Result<(PathBuf, String)> {
        let candidates = vec![
            format!("/opt/homebrew/opt/php@{requested_version}/sbin/php-fpm"),
            "/opt/homebrew/opt/php/sbin/php-fpm".to_string(),
            "/opt/homebrew/sbin/php-fpm".to_string(),
            "/opt/homebrew/bin/php-fpm".to_string(),
            format!("/usr/local/opt/php@{requested_version}/sbin/php-fpm"),
            "/usr/local/sbin/php-fpm".to_string(),
            "/usr/local/bin/php-fpm".to_string(),
        ];

        for c in candidates {
            let p = PathBuf::from(&c);
            if p.exists() {
                if let Some(actual_version) = Self::extract_version_from_binary(&p) {
                    // Match: requested "8.5" matches "8.5.3", or requested "8.5.3" matches "8.5.3"
                    if actual_version.starts_with(requested_version)
                        || requested_version.starts_with(&actual_version)
                        || requested_version == "any"
                    {
                        return Ok((p, actual_version));
                    }
                }
            }
        }

        // Check if plain php-fpm is in PATH
        if let Ok(output) = Command::new("which").arg("php-fpm").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let p = PathBuf::from(&path_str);
                    if let Some(actual_version) = Self::extract_version_from_binary(&p) {
                        if actual_version.starts_with(requested_version)
                            || requested_version.starts_with(&actual_version)
                            || requested_version == "any"
                        {
                            return Ok((p, actual_version));
                        }
                    }
                }
            }
        }

        Err(RuntimeError::RuntimeNotFound {
            binary: "php-fpm".into(),
            version: requested_version.into(),
        })
    }

    /// Validates if the requested PHP version is available on the host.
    pub fn validate_php_version(&self, requested_version: &str) -> Result<PathBuf> {
        let (bin, _) = Self::find_php_fpm_binary(requested_version)?;
        Ok(bin)
    }

    /// Verifies if a given executable path is an authentic canonical PHP-FPM binary.
    /// Strictly validates against discovered legitimate PHP-FPM binaries on the system.
    pub fn is_legitimate_php_fpm_binary(path: &Path) -> bool {
        let canonical = match path.canonicalize() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let common = ["8.5", "8.4", "8.3", "8.2", "8.1", "8.0", "7.4", "any"];
        for v in common {
            if let Ok((bin, _)) = Self::find_php_fpm_binary(v) {
                if let Ok(c) = bin.canonicalize() {
                    if c == canonical {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns the dedicated Unix socket path for a project, safely bounded for sockaddr_un 104-byte limit.
    pub fn socket_path(&self, project_id: &ProjectId) -> PathBuf {
        let standard = self.run_dir.join(format!("{}.sock", project_id.0));
        if standard.as_os_str().len() >= 90 {
            PathBuf::from(format!("/tmp/lf_{}.sock", project_id.0))
        } else {
            standard
        }
    }

    /// Generates isolated FPM pool configuration content.
    pub fn generate_pool_config(&self, project_id: &ProjectId, project_dir: &Path) -> String {
        let sock = self.socket_path(project_id);
        format!(
            "[global]\n\
             pid = {run_dir}/{pid}.pid\n\
             error_log = {run_dir}/{pid}_error.log\n\
             daemonize = no\n\n\
             [{pid}]\n\
             listen = {sock}\n\
             listen.mode = 0660\n\
             pm = ondemand\n\
             pm.max_children = 5\n\
             pm.process_idle_timeout = 10s\n\
             chdir = {project_dir}\n\
             catch_workers_output = yes\n\
             clear_env = no\n\
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
        let mut map = self.active_pools.lock().unwrap();

        // 1. Validate PHP binary
        let (fpm_bin, _) = Self::find_php_fpm_binary(php_version)?;
        let canonical_fpm = fpm_bin.canonicalize().unwrap_or(fpm_bin.clone());

        // 2. Check if already active, alive, and verified
        if let Some(pool) = map.get_mut(&project_id.0) {
            let verified = self.process_manager.verify_process_identity(
                pool.identity.pid,
                pool.identity.executable_path(),
                pool.identity.start_time_secs,
                pool.identity.start_time_usecs,
            );
            if verified && is_unix_socket_alive(&pool.socket_path) {
                return Ok(pool.socket_path.clone());
            } else {
                // Stale or dead process; clean up
                let _ = self.process_manager.terminate_process_group_verified(
                    pool.identity.pid,
                    pool.identity.executable_path(),
                    pool.identity.start_time_secs,
                    pool.identity.start_time_usecs,
                    1,
                );
                map.remove(&project_id.0);
            }
        } else {
            let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
            let sock_path = self.socket_path(project_id);
            if pid_file.exists() && is_unix_socket_alive(&sock_path) {
                if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                            if identity.executable_path() == canonical_fpm {
                                return Ok(sock_path);
                            }
                        }
                    }
                }
            }
        }

        // 3. Stale socket cleanup
        let sock_path = self.socket_path(project_id);
        if sock_path.exists() {
            if !is_unix_socket_alive(&sock_path) {
                let _ = std::fs::remove_file(&sock_path);
            } else {
                // Socket is alive but not tracked in our active pools map. Verify if foreign or orphan.
                let _ = std::fs::remove_file(&sock_path);
            }
        }

        // 4. Generate isolated pool config
        let conf_content = self.generate_pool_config(project_id, project_dir);
        let conf_path = self.run_dir.join(format!("{}.conf", project_id.0));
        std::fs::write(&conf_path, conf_content).map_err(RuntimeError::Io)?;

        // 5. Spawn child process in dedicated process group (unprivileged)
        #[cfg(unix)]
        let child = {
            use std::os::unix::process::CommandExt;
            let mut cmd = Command::new(&fpm_bin);
            cmd.arg("--fpm-config").arg(&conf_path).arg("-F");
            cmd.process_group(0);
            cmd.spawn()
        };

        #[cfg(not(unix))]
        let child = Command::new(&fpm_bin)
            .arg("--fpm-config")
            .arg(&conf_path)
            .arg("-F")
            .spawn();

        let mut child = child.map_err(|e| RuntimeError::ProcessSpawnFailed {
            process: format!("php-fpm-{}", project_id.0),
            reason: e.to_string(),
        })?;

        let pid = child.id();
        let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
        let _ = std::fs::write(&pid_file, pid.to_string());

        // 6. Inspect and record process identity immediately
        let identity = match self.process_manager.inspect_process(pid)? {
            Some(id) => id,
            None => {
                // Immediately check if process failed
                let error_log_path = self.run_dir.join(format!("{}_error.log", project_id.0));
                let details = std::fs::read_to_string(&error_log_path).unwrap_or_default();
                let _ = child.kill();
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: format!("php-fpm-{}", project_id.0),
                    reason: format!("Process exited immediately upon launch. Log: {details}"),
                });
            }
        };

        // 7. Poll for socket readiness (bounded window up to 3 seconds)
        let wait_start = Instant::now();
        let mut socket_ready = false;
        while wait_start.elapsed() < Duration::from_secs(3) {
            if is_unix_socket_alive(&sock_path) {
                socket_ready = true;
                break;
            }
            if let Ok(Some(status)) = child.try_wait() {
                let error_log_path = self.run_dir.join(format!("{}_error.log", project_id.0));
                let details = std::fs::read_to_string(&error_log_path).unwrap_or_default();
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: format!("php-fpm-{}", project_id.0),
                    reason: format!("Process exited with status {status}. Log: {details}"),
                });
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        if !socket_ready {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RuntimeError::ServiceUnhealthy {
                service: format!("php-fpm-{}", project_id.0),
                reason: format!(
                    "Socket '{}' failed to become ready within 3 seconds",
                    sock_path.display()
                ),
            });
        }

        map.insert(
            project_id.0.clone(),
            ManagedPool {
                identity,
                socket_path: sock_path.clone(),
                conf_path,
                child: Some(child),
            },
        );

        Ok(sock_path)
    }

    /// Stops a running project FPM pool gracefully with verified process identity.
    pub fn stop_pool(&self, project_id: &ProjectId) -> Result<()> {
        let mut map = self.active_pools.lock().unwrap();
        if let Some(mut pool) = map.remove(&project_id.0) {
            // Terminate verified process group (5s graceful window)
            let _ = self.process_manager.terminate_process_group_verified(
                pool.identity.pid,
                pool.identity.executable_path(),
                pool.identity.start_time_secs,
                pool.identity.start_time_usecs,
                5,
            );

            if let Some(mut child) = pool.child.take() {
                let _ = child.wait();
            }

            // Remove socket and config
            if pool.socket_path.exists() {
                let _ = std::fs::remove_file(&pool.socket_path);
            }
            if pool.conf_path.exists() {
                let _ = std::fs::remove_file(&pool.conf_path);
            }
        } else {
            let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
            if pid_file.exists() {
                if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                            if Self::is_legitimate_php_fpm_binary(identity.executable_path()) {
                                let _ = self.process_manager.terminate_process_group_verified(
                                    identity.pid,
                                    identity.executable_path(),
                                    identity.start_time_secs,
                                    identity.start_time_usecs,
                                    5,
                                );
                            }
                        }
                    }
                }
            }
        }

        let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
        if pid_file.exists() {
            let _ = std::fs::remove_file(&pid_file);
        }
        let sock = self.socket_path(project_id);
        if sock.exists() {
            let _ = std::fs::remove_file(&sock);
        }
        let conf = self.run_dir.join(format!("{}.conf", project_id.0));
        if conf.exists() {
            let _ = std::fs::remove_file(&conf);
        }

        Ok(())
    }

    /// Checks if a project's FPM pool is actively running and verified.
    pub fn is_pool_alive(&self, project_id: &ProjectId) -> bool {
        let map = self.active_pools.lock().unwrap();
        if let Some(pool) = map.get(&project_id.0) {
            let verified = self.process_manager.verify_process_identity(
                pool.identity.pid,
                pool.identity.executable_path(),
                pool.identity.start_time_secs,
                pool.identity.start_time_usecs,
            );
            verified && is_unix_socket_alive(&pool.socket_path)
        } else {
            let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
            let sock_path = self.socket_path(project_id);
            if pid_file.exists() && is_unix_socket_alive(&sock_path) {
                if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                            return Self::is_legitimate_php_fpm_binary(identity.executable_path());
                        }
                    }
                }
            }
            false
        }
    }

    /// Returns process identity for an active pool.
    pub fn get_pool_identity(&self, project_id: &ProjectId) -> Option<ProcessIdentity> {
        let map = self.active_pools.lock().unwrap();
        if let Some(p) = map.get(&project_id.0) {
            return Some(p.identity.clone());
        }
        let pid_file = self.run_dir.join(format!("{}.pid", project_id.0));
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                    if Self::is_legitimate_php_fpm_binary(identity.executable_path()) {
                        return Some(identity);
                    }
                }
            }
        }
        None
    }
}

impl Drop for PhpFpmSupervisor {
    fn drop(&mut self) {
        if !self.terminate_on_drop {
            return;
        }
        if let Ok(mut map) = self.active_pools.lock() {
            for (_, mut pool) in map.drain() {
                let _ = self.process_manager.terminate_process_group_verified(
                    pool.identity.pid,
                    pool.identity.executable_path(),
                    pool.identity.start_time_secs,
                    pool.identity.start_time_usecs,
                    1,
                );
                if let Some(mut child) = pool.child.take() {
                    let _ = child.wait();
                }
                if pool.socket_path.exists() {
                    let _ = std::fs::remove_file(&pool.socket_path);
                }
                if pool.conf_path.exists() {
                    let _ = std::fs::remove_file(&pool.conf_path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use localforge_platform::MacOSProcessManager;
    use tempfile::tempdir;

    #[test]
    fn test_pool_config_generation() {
        let dir = tempdir().unwrap();
        let pm = Arc::new(MacOSProcessManager::new());
        let sup = PhpFpmSupervisor::new(dir.path().to_path_buf(), pm);
        let pid = ProjectId::from_string("proj_abc");
        let proj_dir = Path::new("/Volumes/Mac Harici Disk/Projects/mytest");

        let conf = sup.generate_pool_config(&pid, proj_dir);
        assert!(conf.contains("[proj_abc]"));
        assert!(conf.contains("proj_abc.sock"));
        assert!(conf.contains("pm = ondemand"));
    }

    #[test]
    fn test_invalid_php_version_fails_safely() {
        let res = PhpFpmSupervisor::find_php_fpm_binary("5.2-ancient");
        assert!(res.is_err());
        match res.unwrap_err() {
            RuntimeError::RuntimeNotFound { version, .. } => {
                assert_eq!(version, "5.2-ancient");
            }
            other => panic!("Unexpected error: {other:?}"),
        }
    }
}
