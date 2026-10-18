use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use localforge_core::ProjectId;
use localforge_platform::{ProcessIdentity, ProcessManager};
use localforge_storage::ProjectRecord;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct NginxSupervisor {
    conf_dir: PathBuf,
    run_dir: PathBuf,
    listen_port: u16,
    process_manager: Arc<dyn ProcessManager>,
    master_process: Mutex<Option<(ProcessIdentity, Option<Child>)>>,
    pub terminate_on_drop: bool,
}

impl NginxSupervisor {
    pub fn new(
        conf_dir: PathBuf,
        run_dir: PathBuf,
        listen_port: u16,
        process_manager: Arc<dyn ProcessManager>,
    ) -> Self {
        let _ = std::fs::create_dir_all(&conf_dir);
        let _ = std::fs::create_dir_all(conf_dir.join("conf.d"));
        let _ = std::fs::create_dir_all(&run_dir);

        let sup = Self {
            conf_dir,
            run_dir,
            listen_port,
            process_manager,
            master_process: Mutex::new(None),
            terminate_on_drop: false,
        };
        sup.ensure_base_support_files();
        sup
    }

    /// Discovers Nginx binary on macOS.
    pub fn find_nginx_binary() -> Result<PathBuf> {
        let candidates = vec![
            "/opt/homebrew/bin/nginx",
            "/opt/homebrew/sbin/nginx",
            "/usr/local/bin/nginx",
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

    /// Ensures `mime.types` and `fastcgi_params` are present in `conf_dir`.
    fn ensure_base_support_files(&self) {
        let mime_path = self.conf_dir.join("mime.types");
        if !mime_path.exists() {
            let mimes = "\
types {
    text/html                             html htm shtml;
    text/css                              css;
    text/xml                              xml;
    image/gif                             gif;
    image/jpeg                            jpeg jpg;
    application/javascript                js;
    application/json                      json;
    image/png                             png;
    image/svg+xml                         svg svgz;
    image/webp                            webp;
    text/plain                            txt;
}
";
            let _ = std::fs::write(mime_path, mimes);
        }

        let fastcgi_path = self.conf_dir.join("fastcgi_params");
        if !fastcgi_path.exists() {
            let params = "\
fastcgi_param  QUERY_STRING       $query_string;
fastcgi_param  REQUEST_METHOD     $request_method;
fastcgi_param  CONTENT_TYPE       $content_type;
fastcgi_param  CONTENT_LENGTH     $content_length;

fastcgi_param  SCRIPT_FILENAME    $document_root$fastcgi_script_name;
fastcgi_param  SCRIPT_NAME        $fastcgi_script_name;
fastcgi_param  REQUEST_URI        $request_uri;
fastcgi_param  DOCUMENT_URI       $document_uri;
fastcgi_param  DOCUMENT_ROOT      $document_root;
fastcgi_param  SERVER_PROTOCOL    $server_protocol;
fastcgi_param  REQUEST_SCHEME     $scheme;
fastcgi_param  HTTPS              $https if_not_empty;

fastcgi_param  GATEWAY_INTERFACE  CGI/1.1;
fastcgi_param  SERVER_SOFTWARE    nginx/$nginx_version;

fastcgi_param  REMOTE_ADDR        $remote_addr;
fastcgi_param  REMOTE_PORT        $remote_port;
fastcgi_param  SERVER_ADDR        $server_addr;
fastcgi_param  SERVER_PORT        $server_port;
fastcgi_param  SERVER_NAME        $server_name;

fastcgi_param  REDIRECT_STATUS    200;
";
            let _ = std::fs::write(fastcgi_path, params);
        }

        // Main nginx.conf
        let main_conf_path = self.conf_dir.join("nginx.conf");
        let main_conf = format!(
            "daemon off;\n\
             error_log \"{run_dir}/nginx_error.log\" warn;\n\
             pid \"{run_dir}/nginx.pid\";\n\n\
             events {{\n\
                 worker_connections 1024;\n\
             }}\n\n\
             http {{\n\
                 include mime.types;\n\
                 default_type application/octet-stream;\n\
                 access_log \"{run_dir}/nginx_access.log\";\n\
                 sendfile on;\n\
                 keepalive_timeout 65;\n\n\
                 include \"{conf_dir}/conf.d/*.conf\";\n\
             }}\n",
            run_dir = self.run_dir.display(),
            conf_dir = self.conf_dir.display()
        );
        let _ = std::fs::write(main_conf_path, main_conf);
    }

    /// Generates isolated virtual host configuration for a project.
    pub fn generate_vhost_config(&self, project: &ProjectRecord, socket_path: &Path) -> String {
        let docroot = if project.document_root.is_empty() {
            project.path.clone()
        } else {
            project.path.join(&project.document_root)
        };

        format!(
            "server {{\n\
                 listen 127.0.0.1:{port};\n\
                 server_name {domain};\n\
                 root \"{docroot}\";\n\
                 index index.php index.html;\n\n\
                 location / {{\n\
                     try_files $uri $uri/ /index.php?$query_string;\n\
                 }}\n\n\
                 location ~ \\.php$ {{\n\
                     include fastcgi_params;\n\
                     fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n\
                     fastcgi_pass unix:{sock};\n\
                 }}\n\
             }}\n",
            port = self.listen_port,
            domain = project.local_domain,
            docroot = docroot.display(),
            sock = socket_path.display()
        )
    }

    /// Validates the entire Nginx configuration via `nginx -t`.
    pub fn validate_configuration(&self) -> Result<()> {
        let nginx_bin = Self::find_nginx_binary()?;
        let conf_file = self.conf_dir.join("nginx.conf");

        let output = Command::new(&nginx_bin)
            .arg("-p")
            .arg(&self.conf_dir)
            .arg("-c")
            .arg(&conf_file)
            .arg("-t")
            .output()
            .map_err(|e| {
                RuntimeError::ConfigGenerationFailed(format!("Failed to invoke nginx -t: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RuntimeError::ConfigGenerationFailed(format!(
                "Nginx configuration validation failed (nginx -t): {stderr}"
            )));
        }

        Ok(())
    }

    /// Adds or updates a project's virtual host and applies configuration.
    pub fn add_or_update_vhost(&self, project: &ProjectRecord, socket_path: &Path) -> Result<()> {
        let vhost_dir = self.conf_dir.join("conf.d");
        let vhost_file = vhost_dir.join(format!("{}.conf", project.id.0));
        let vhost_content = self.generate_vhost_config(project, socket_path);

        // Write new virtual host
        std::fs::write(&vhost_file, vhost_content).map_err(RuntimeError::Io)?;

        // Validate configuration with nginx -t
        if let Err(e) = self.validate_configuration() {
            // Rollback failing virtual host so running projects are unharmed
            let _ = std::fs::remove_file(&vhost_file);
            return Err(e);
        }

        // Apply changes (reload or spawn)
        self.ensure_running_and_reloaded()
    }

    /// Removes a project's virtual host and reloads gateway.
    pub fn remove_vhost(&self, project_id: &ProjectId) -> Result<()> {
        let vhost_file = self
            .conf_dir
            .join("conf.d")
            .join(format!("{}.conf", project_id.0));
        if vhost_file.exists() {
            let _ = std::fs::remove_file(&vhost_file);
        }

        if self.is_gateway_alive() {
            let _ = self.reload();
        }
        Ok(())
    }

    /// Ensures the shared Nginx master is running, or reloads it if already running.
    pub fn ensure_running_and_reloaded(&self) -> Result<()> {
        let mut lock = self.master_process.lock().unwrap();

        // 1. Check if we have an active, verified master process
        if let Some((identity, child_opt)) = lock.as_mut() {
            let verified = self.process_manager.verify_process_identity(
                identity.pid,
                identity.executable_path(),
                identity.start_time_secs,
                identity.start_time_usecs,
            );
            if verified {
                // Master is alive, reload config
                return self.reload();
            } else {
                // Stale or dead master process
                if let Some(mut child) = child_opt.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                *lock = None;
            }
        }

        // 2. Check if port 8080 is available or if port conflict exists
        if !is_port_available(self.listen_port) {
            // Check if nginx.pid exists on disk and belongs to our Nginx
            let pid_file = self.run_dir.join("nginx.pid");
            if pid_file.exists() {
                if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                            let nginx_bin = Self::find_nginx_binary()?;
                            let canonical_bin =
                                dunce::canonicalize(&nginx_bin).unwrap_or(nginx_bin);
                            if identity.executable_path() == canonical_bin {
                                // Port 8080 is bound by our LocalForge Nginx master process!
                                *lock = Some((identity, None));
                                return self.reload();
                            }
                        }
                    }
                }
            }

            return Err(RuntimeError::PortConflict {
                port: self.listen_port,
                details: format!(
                    "Port {} is occupied by an external, non-LocalForge process.",
                    self.listen_port
                ),
            });
        }

        // 3. Spawn new Nginx master process (unprivileged)
        let nginx_bin = Self::find_nginx_binary()?;
        let conf_file = self.conf_dir.join("nginx.conf");

        #[cfg(unix)]
        let child = {
            use std::os::unix::process::CommandExt;
            let mut cmd = Command::new(&nginx_bin);
            cmd.arg("-p").arg(&self.conf_dir).arg("-c").arg(&conf_file);
            cmd.process_group(0);
            cmd.spawn()
        };

        #[cfg(not(unix))]
        let child = Command::new(&nginx_bin)
            .arg("-p")
            .arg(&self.conf_dir)
            .arg("-c")
            .arg(&conf_file)
            .spawn();

        let mut child = child.map_err(|e| RuntimeError::ProcessSpawnFailed {
            process: "nginx".into(),
            reason: e.to_string(),
        })?;

        let pid = child.id();

        // 4. Inspect process identity immediately
        let identity = match self.process_manager.inspect_process(pid)? {
            Some(id) => id,
            None => {
                let err_file = self.run_dir.join("nginx_error.log");
                let log = std::fs::read_to_string(err_file).unwrap_or_default();
                let _ = child.kill();
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: "nginx".into(),
                    reason: format!("Nginx exited immediately. Error log: {log}"),
                });
            }
        };

        // 5. Poll for port readiness (up to 3 seconds)
        let wait_start = Instant::now();
        let mut port_bound = false;
        while wait_start.elapsed() < Duration::from_secs(3) {
            if !is_port_available(self.listen_port) {
                port_bound = true;
                break;
            }
            if let Ok(Some(status)) = child.try_wait() {
                let err_file = self.run_dir.join("nginx_error.log");
                let log = std::fs::read_to_string(err_file).unwrap_or_default();
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: "nginx".into(),
                    reason: format!("Nginx master exited with {status}. Log: {log}"),
                });
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        if !port_bound {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RuntimeError::ServiceUnhealthy {
                service: "nginx".into(),
                reason: format!("Nginx failed to bind port {} within 3s", self.listen_port),
            });
        }

        *lock = Some((identity, Some(child)));
        Ok(())
    }

    /// Issues `nginx -s reload` to active master process.
    pub fn reload(&self) -> Result<()> {
        let nginx_bin = Self::find_nginx_binary()?;
        let conf_file = self.conf_dir.join("nginx.conf");

        let status = Command::new(nginx_bin)
            .arg("-p")
            .arg(&self.conf_dir)
            .arg("-c")
            .arg(&conf_file)
            .arg("-s")
            .arg("reload")
            .status()
            .map_err(|e| RuntimeError::ServiceUnhealthy {
                service: "nginx".into(),
                reason: format!("Failed to reload Nginx: {e}"),
            })?;

        if !status.success() {
            return Err(RuntimeError::ServiceUnhealthy {
                service: "nginx".into(),
                reason: format!("Nginx reload command exited with {status}"),
            });
        }

        Ok(())
    }

    /// Checks if the Nginx gateway is alive and bound.
    pub fn is_gateway_alive(&self) -> bool {
        let lock = self.master_process.lock().unwrap();
        if let Some((identity, _)) = lock.as_ref() {
            let verified = self.process_manager.verify_process_identity(
                identity.pid,
                identity.executable_path(),
                identity.start_time_secs,
                identity.start_time_usecs,
            );
            verified && !is_port_available(self.listen_port)
        } else {
            !is_port_available(self.listen_port)
        }
    }

    /// Returns process identity of the running Nginx gateway.
    pub fn get_gateway_identity(&self) -> Option<ProcessIdentity> {
        let lock = self.master_process.lock().unwrap();
        if let Some((id, _)) = lock.as_ref() {
            return Some(id.clone());
        }
        let pid_file = self.run_dir.join("nginx.pid");
        if pid_file.exists() {
            if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                        return Some(identity);
                    }
                }
            }
        }
        None
    }

    /// Stops Nginx gracefully.
    pub fn stop(&self) -> Result<()> {
        let mut lock = self.master_process.lock().unwrap();
        if let Some((identity, mut child_opt)) = lock.take() {
            let _ = self.process_manager.terminate_process_group_verified(
                identity.pid,
                identity.executable_path(),
                identity.start_time_secs,
                identity.start_time_usecs,
                5,
            );

            if let Some(mut child) = child_opt.take() {
                let _ = child.wait();
            }
        } else {
            let pid_file = self.run_dir.join("nginx.pid");
            if pid_file.exists() {
                if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
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

        let pid_file = self.run_dir.join("nginx.pid");
        if pid_file.exists() {
            let _ = std::fs::remove_file(&pid_file);
        }

        Ok(())
    }
}

impl Drop for NginxSupervisor {
    fn drop(&mut self) {
        if self.terminate_on_drop {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use localforge_core::{ProjectId, ProjectStatus, VolumeId};
    use localforge_platform::MacOSProcessManager;
    use tempfile::tempdir;

    #[test]
    fn test_nginx_config_generation() {
        let dir = tempdir().unwrap();
        let pm = Arc::new(MacOSProcessManager::new());
        let sup = NginxSupervisor::new(dir.path().join("conf"), dir.path().join("run"), 8080, pm);

        let project = ProjectRecord {
            id: ProjectId::from_string("proj_site1"),
            name: "Site One".into(),
            path: PathBuf::from("/Volumes/Mac Harici Disk/Projects/site1"),
            volume_uuid: VolumeId::new("UUID-1"),
            document_root: "public".into(),
            php_version: "8.5".into(),
            local_domain: "site1.test".into(),
            status: ProjectStatus::Running,
            created_at: "2026-10-18T00:00:00Z".into(),
            updated_at: "2026-10-18T00:00:00Z".into(),
        };

        let sock = Path::new("/tmp/localforge/site1.sock");
        let vhost = sup.generate_vhost_config(&project, sock);
        assert!(vhost.contains("listen 127.0.0.1:8080;"));
        assert!(vhost.contains("server_name site1.test"));
        assert!(vhost.contains("fastcgi_pass unix:/tmp/localforge/site1.sock;"));
        assert!(vhost.contains("root \"/Volumes/Mac Harici Disk/Projects/site1/public\";"));
    }
}
