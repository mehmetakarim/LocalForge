use crate::error::{Result, RuntimeError};
use localforge_storage::ProjectRecord;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

pub struct NginxSupervisor {
    conf_dir: PathBuf,
    run_dir: PathBuf,
    listen_port: u16,
    process: Mutex<Option<Child>>,
}

impl NginxSupervisor {
    pub fn new(conf_dir: PathBuf, run_dir: PathBuf, listen_port: u16) -> Self {
        let _ = std::fs::create_dir_all(&conf_dir);
        let _ = std::fs::create_dir_all(&run_dir);
        Self {
            conf_dir,
            run_dir,
            listen_port,
            process: Mutex::new(None),
        }
    }

    /// Discovers Nginx binary on macOS.
    pub fn find_nginx_binary() -> Result<PathBuf> {
        let candidates = vec![
            "/opt/homebrew/bin/nginx",
            "/usr/local/bin/nginx",
            "/opt/homebrew/sbin/nginx",
            "/usr/local/sbin/nginx",
        ];

        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Ok(p);
            }
        }

        if let Ok(output) = Command::new("which").arg("nginx").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }
        }

        Err(RuntimeError::RuntimeNotFound {
            binary: "nginx".into(),
            version: "any".into(),
        })
    }

    /// Generates main non-root Nginx configuration file routing projects by Host header.
    pub fn generate_config(&self, projects: &[ProjectRecord]) -> String {
        let mut server_blocks = String::new();

        for p in projects {
            let docroot = if p.document_root.is_empty() {
                p.path.clone()
            } else {
                p.path.join(&p.document_root)
            };

            let socket_path = self.run_dir.join(format!("{}.sock", p.id.0));

            server_blocks.push_str(&format!(
                "    server {{\n\
                 \x20       listen 127.0.0.1:{port};\n\
                 \x20       server_name {domain};\n\
                 \x20       root {docroot};\n\
                 \x20       index index.php index.html;\n\n\
                 \x20       location / {{\n\
                 \x20           try_files $uri $uri/ /index.php?$query_string;\n\
                 \x20       }}\n\n\
                 \x20       location ~ \\.php$ {{\n\
                 \x20           include fastcgi_params;\n\
                 \x20           fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n\
                 \x20           fastcgi_pass unix:{sock};\n\
                 \x20       }}\n\
                 \x20   }}\n\n",
                port = self.listen_port,
                domain = p.local_domain,
                docroot = docroot.display(),
                sock = socket_path.display()
            ));
        }

        format!(
            "daemon off;\n\
             error_log {run_dir}/nginx_error.log warn;\n\
             pid {run_dir}/nginx.pid;\n\n\
             events {{\n\
             \x20   worker_connections 1024;\n\
             }}\n\n\
             http {{\n\
             \x20   include mime.types;\n\
             \x20   default_type application/octet-stream;\n\
             \x20   access_log {run_dir}/nginx_access.log;\n\
             \x20   sendfile on;\n\
             \x20   keepalive_timeout 65;\n\n\
             {server_blocks}\
             }}\n",
            run_dir = self.run_dir.display(),
            server_blocks = server_blocks
        )
    }

    /// Writes configuration and reloads or spawns Nginx process.
    pub fn apply_config(&self, projects: &[ProjectRecord]) -> Result<()> {
        let conf_content = self.generate_config(projects);
        let conf_path = self.conf_dir.join("nginx.conf");
        std::fs::write(&conf_path, conf_content).map_err(RuntimeError::Io)?;

        // Ensure mime.types exists in conf_dir if missing
        let mime_path = self.conf_dir.join("mime.types");
        if !mime_path.exists() {
            let basic_mimes = "types {\n    text/html html;\n    text/css css;\n    application/javascript js;\n    image/png png;\n    image/jpeg jpeg jpg;\n}\n";
            let _ = std::fs::write(mime_path, basic_mimes);
        }

        let mut lock = self.process.lock().unwrap();

        // If running, reload with -s reload
        if let Some(child) = lock.as_mut() {
            if child.try_wait().unwrap_or(None).is_none() {
                let nginx_bin = Self::find_nginx_binary()?;
                let _ = Command::new(nginx_bin)
                    .arg("-p")
                    .arg(&self.conf_dir)
                    .arg("-c")
                    .arg(&conf_path)
                    .arg("-s")
                    .arg("reload")
                    .status();
                return Ok(());
            }
        }

        // Spawn new process
        let nginx_bin = Self::find_nginx_binary()?;
        let child = Command::new(nginx_bin)
            .arg("-p")
            .arg(&self.conf_dir)
            .arg("-c")
            .arg(&conf_path)
            .spawn()
            .map_err(|e| RuntimeError::ProcessSpawnFailed {
                process: "nginx".into(),
                reason: e.to_string(),
            })?;

        *lock = Some(child);
        Ok(())
    }

    /// Stops Nginx gracefully.
    pub fn stop(&self) -> Result<()> {
        let mut lock = self.process.lock().unwrap();
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for NginxSupervisor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use localforge_core::{ProjectId, ProjectStatus, VolumeId};
    use tempfile::tempdir;

    #[test]
    fn test_nginx_config_generation() {
        let dir = tempdir().unwrap();
        let sup = NginxSupervisor::new(dir.path().join("conf"), dir.path().join("run"), 8080);

        let projects = vec![ProjectRecord {
            id: ProjectId::from_string("proj_site1"),
            name: "Site One".into(),
            path: PathBuf::from("/Volumes/Mac Harici Disk/Projects/site1"),
            volume_uuid: VolumeId::new("UUID-1"),
            document_root: "public".into(),
            php_version: "8.2".into(),
            local_domain: "site1.test".into(),
            status: ProjectStatus::Running,
            created_at: "2026-10-18T00:00:00Z".into(),
            updated_at: "2026-10-18T00:00:00Z".into(),
        }];

        let conf = sup.generate_config(&projects);
        assert!(conf.contains("listen 127.0.0.1:8080;"));
        assert!(conf.contains("server_name site1.test;"));
        assert!(conf.contains("fastcgi_pass unix:"));
        assert!(conf.contains("proj_site1.sock"));
    }
}
