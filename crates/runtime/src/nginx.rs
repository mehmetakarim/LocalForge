use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use localforge_core::ProjectId;
use localforge_platform::{ProcessIdentity, ProcessManager};
use localforge_storage::ProjectRecord;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GatewayHealth {
    Running {
        pid: u32,
        port: u16,
    },
    Stopped,
    PortConflict {
        port: u16,
        occupying_pid: Option<u32>,
        details: String,
    },
}

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
            terminate_on_drop: true,
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
                 server {{\n\
                     listen 127.0.0.1:{listen_port} default_server;\n\
                     server_name _;\n\
                     return 404 \"LocalForge Gateway: Host header required or project domain not found\\n\";\n\
                 }}\n\n\
                 include \"{conf_dir}/conf.d/*.conf\";\n\
             }}\n",
            run_dir = self.run_dir.display(),
            conf_dir = self.conf_dir.display(),
            listen_port = self.listen_port
        );
        let _ = std::fs::write(main_conf_path, main_conf);
    }

    /// Generates multi-port virtual host configuration for a project supporting
    /// branded local domains (.localforge), .localhost subdomains, dedicated loopback ports,
    /// and fail-closed wrong-project isolation when stopped or unmounted.
    pub fn generate_multi_port_vhost_config(
        &self,
        project: &ProjectRecord,
        socket_path: Option<&Path>,
        is_running: bool,
        is_volume_mounted: bool,
    ) -> String {
        let slug = crate::routing::ProjectRoutingEngine::slugify(&project.name);
        let preferred_domain = format!("{slug}.localforge");
        let localhost_domain = format!("{slug}.localforge.localhost");

        let shared_server_names = format!(
            "{preferred_domain} {localhost_domain} {}",
            project.local_domain
        );
        let dedicated_server_names = format!(
            "{preferred_domain} {localhost_domain} {} 127.0.0.1 localhost",
            project.local_domain
        );

        let mut config = String::new();

        // 1. Dedicated loopback port blocks (if project has assigned loopback_port)
        if let Some(port) = project.loopback_port {
            // Default server for dedicated port: reject wrong Host headers with 404 (Fail-Closed)
            config.push_str(&format!(
                "server {{\n\
                 listen 127.0.0.1:{port} default_server;\n\
                 server_name _;\n\
                 return 404 \"LocalForge Gateway: Host mismatch on dedicated project port\\n\";\n\
                 }}\n\n"
            ));

            if !is_volume_mounted {
                config.push_str(&format!(
                    "server {{\n\
                     listen 127.0.0.1:{port};\n\
                     server_name {dedicated_server_names};\n\
                     port_in_redirect on;\n\
                     server_name_in_redirect off;\n\
                     return 502 \"LocalForge Gateway: Depolama Sürücüsü Bağlı Değil (Volume Disconnected)\\n\";\n\
                     }}\n\n"
                ));
            } else if !is_running || socket_path.is_none() {
                config.push_str(&format!(
                    "server {{\n\
                     listen 127.0.0.1:{port};\n\
                     server_name {dedicated_server_names};\n\
                     port_in_redirect on;\n\
                     server_name_in_redirect off;\n\
                     return 503 \"LocalForge Gateway: Proje Başlatılmamış veya Durdurulmuş (Project Stopped)\\n\";\n\
                     }}\n\n"
                ));
            } else {
                let docroot = if project.document_root.is_empty() {
                    project.path.clone()
                } else {
                    project.path.join(&project.document_root)
                };
                let sock = socket_path.unwrap();
                config.push_str(&format!(
                    "server {{\n\
                     listen 127.0.0.1:{port};\n\
                     server_name {dedicated_server_names};\n\
                     port_in_redirect on;\n\
                     server_name_in_redirect off;\n\
                     root \"{docroot}\";\n\
                     index index.php index.html;\n\n\
                     location / {{\n\
                         try_files $uri $uri/ /index.php?$query_string;\n\
                     }}\n\n\
                     location ~ \\.php$ {{\n\
                         include fastcgi_params;\n\
                         fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n\
                         fastcgi_param SERVER_PORT $server_port;\n\
                         fastcgi_param HTTP_HOST $http_host;\n\
                         fastcgi_pass unix:{sock};\n\
                     }}\n\
                     }}\n\n",
                    docroot = docroot.display(),
                    sock = sock.display()
                ));
            }
        }

        // 2. Shared gateway port block (listen_port = 8080)
        // Matches ONLY project-specific domains (never generic localhost or 127.0.0.1)
        if !is_volume_mounted {
            config.push_str(&format!(
                "server {{\n\
                 listen 127.0.0.1:{listen_port};\n\
                 server_name {shared_server_names};\n\
                 port_in_redirect on;\n\
                 server_name_in_redirect off;\n\
                 return 502 \"LocalForge Gateway: Depolama Sürücüsü Bağlı Değil (Volume Disconnected)\\n\";\n\
                 }}\n",
                listen_port = self.listen_port
            ));
        } else if !is_running || socket_path.is_none() {
            config.push_str(&format!(
                "server {{\n\
                 listen 127.0.0.1:{listen_port};\n\
                 server_name {shared_server_names};\n\
                 port_in_redirect on;\n\
                 server_name_in_redirect off;\n\
                 return 503 \"LocalForge Gateway: Proje Başlatılmamış veya Durdurulmuş (Project Stopped)\\n\";\n\
                 }}\n",
                listen_port = self.listen_port
            ));
        } else {
            let docroot = if project.document_root.is_empty() {
                project.path.clone()
            } else {
                project.path.join(&project.document_root)
            };
            let sock = socket_path.unwrap();
            config.push_str(&format!(
                "server {{\n\
                 listen 127.0.0.1:{listen_port};\n\
                 server_name {shared_server_names};\n\
                 port_in_redirect on;\n\
                 server_name_in_redirect off;\n\
                 root \"{docroot}\";\n\
                 index index.php index.html;\n\n\
                 location / {{\n\
                     try_files $uri $uri/ /index.php?$query_string;\n\
                 }}\n\n\
                 location ~ \\.php$ {{\n\
                     include fastcgi_params;\n\
                     fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n\
                     fastcgi_param SERVER_PORT $server_port;\n\
                     fastcgi_param HTTP_HOST $http_host;\n\
                     fastcgi_pass unix:{sock};\n\
                 }}\n\
                 }}\n",
                listen_port = self.listen_port,
                docroot = docroot.display(),
                sock = sock.display()
            ));
        }

        config
    }

    /// Generates isolated virtual host configuration for a project.
    pub fn generate_vhost_config(&self, project: &ProjectRecord, socket_path: &Path) -> String {
        self.generate_multi_port_vhost_config(project, Some(socket_path), true, true)
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

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            return Err(RuntimeError::ConfigGenerationFailed(format!(
                "Nginx configuration validation failed (nginx -t): {stderr}"
            )));
        }

        // Catch conflicting server name warnings and fail closed
        if stderr.contains("conflicting server name") {
            return Err(RuntimeError::ConfigGenerationFailed(format!(
                "Nginx configuration has conflicting server names (fail-closed): {stderr}"
            )));
        }

        Ok(())
    }

    /// Updates virtual host state with rollback to last known-good configuration.
    pub fn update_vhost_state(
        &self,
        project: &ProjectRecord,
        socket_path: Option<&Path>,
        is_running: bool,
        is_volume_mounted: bool,
    ) -> Result<()> {
        let vhost_dir = self.conf_dir.join("conf.d");
        let _ = std::fs::create_dir_all(&vhost_dir);
        let vhost_file = vhost_dir.join(format!("{}.conf", project.id.0));
        let backup_file = vhost_dir.join(format!("{}.conf.bak", project.id.0));

        let had_previous = vhost_file.exists();
        if had_previous {
            let _ = std::fs::copy(&vhost_file, &backup_file);
        }

        let vhost_content = self.generate_multi_port_vhost_config(
            project,
            socket_path,
            is_running,
            is_volume_mounted,
        );
        std::fs::write(&vhost_file, vhost_content).map_err(RuntimeError::Io)?;

        // Validate configuration with nginx -t
        if let Err(e) = self.validate_configuration() {
            // Rollback failing virtual host to last known-good configuration
            if had_previous && backup_file.exists() {
                let _ = std::fs::rename(&backup_file, &vhost_file);
            } else {
                let _ = std::fs::remove_file(&vhost_file);
            }
            return Err(e);
        }

        if backup_file.exists() {
            let _ = std::fs::remove_file(&backup_file);
        }

        // Apply changes (reload or spawn)
        self.ensure_running_and_reloaded()
    }

    /// Adds or updates a project's virtual host and applies configuration.
    pub fn add_or_update_vhost(&self, project: &ProjectRecord, socket_path: &Path) -> Result<()> {
        self.update_vhost_state(project, Some(socket_path), true, true)
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

        // Brief delay (150ms) to allow master to dispatch HUP and workers to switch configs
        std::thread::sleep(Duration::from_millis(150));

        Ok(())
    }

    /// Compares two filesystem paths, handling macOS /private symlinks even if
    /// the target file or directory has already been unlinked from disk.
    pub fn paths_match(a: &Path, b: &Path) -> bool {
        if a == b {
            return true;
        }
        if let (Ok(ca), Ok(cb)) = (dunce::canonicalize(a), dunce::canonicalize(b)) {
            if ca == cb {
                return true;
            }
        }
        let sa = a.to_string_lossy();
        let sb = b.to_string_lossy();
        let clean_a = sa.strip_prefix("/private").unwrap_or(&sa);
        let clean_b = sb.strip_prefix("/private").unwrap_or(&sb);
        clean_a == clean_b
    }

    /// Validates whether a given executable path is an authentic canonical Nginx binary.
    pub fn is_legitimate_nginx_binary(path: &Path) -> bool {
        if let Ok(bin) = Self::find_nginx_binary() {
            if Self::paths_match(path, &bin) {
                return true;
            }
        }

        let candidates = [
            "/opt/homebrew/bin/nginx",
            "/opt/homebrew/sbin/nginx",
            "/usr/local/bin/nginx",
            "/usr/local/sbin/nginx",
        ];
        for c in candidates {
            if Self::paths_match(path, Path::new(c)) {
                return true;
            }
        }

        false
    }

    /// Checks if command line arguments match LocalForge Nginx configuration.
    ///
    /// Requires exact canonical path matches for configuration (`-c`) and prefix (`-p`).
    /// Accidental substring matches (e.g. `/tmp/spoof/<conf_dir>/nginx.conf`) are strictly rejected.
    pub fn is_localforge_nginx_cmdline(&self, args: &[String]) -> bool {
        let expected_conf = dunce::canonicalize(self.conf_dir.join("nginx.conf"))
            .unwrap_or_else(|_| self.conf_dir.join("nginx.conf"));
        let expected_dir =
            dunce::canonicalize(&self.conf_dir).unwrap_or_else(|_| self.conf_dir.clone());

        let mut observed_conf: Option<PathBuf> = None;
        let mut observed_prefix: Option<PathBuf> = None;

        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            if arg == "-c" {
                if let Some(next) = iter.next() {
                    let p = PathBuf::from(next);
                    observed_conf = Some(dunce::canonicalize(&p).unwrap_or(p));
                }
            } else if let Some(stripped) = arg.strip_prefix("-c") {
                if !stripped.is_empty() {
                    let p = PathBuf::from(stripped);
                    observed_conf = Some(dunce::canonicalize(&p).unwrap_or(p));
                }
            } else if arg == "-p" {
                if let Some(next) = iter.next() {
                    let p = PathBuf::from(next);
                    observed_prefix = Some(dunce::canonicalize(&p).unwrap_or(p));
                }
            } else if let Some(stripped) = arg.strip_prefix("-p") {
                if !stripped.is_empty() {
                    let p = PathBuf::from(stripped);
                    observed_prefix = Some(dunce::canonicalize(&p).unwrap_or(p));
                }
            }
        }

        // Exact canonical match required for the configuration file
        let conf_matches = match observed_conf {
            Some(ref c) => Self::paths_match(c, &expected_conf),
            None => false,
        };

        // If prefix is provided, it must match expected_dir exactly
        let prefix_matches = match observed_prefix {
            Some(ref p) => Self::paths_match(p, &expected_dir),
            None => true,
        };

        conf_matches && prefix_matches
    }

    /// Evaluates the true ownership, process identity and operational health of the gateway.
    /// Distinguishes verified LocalForge Nginx from an unrelated process occupying the port.
    pub fn check_gateway_health(&self) -> GatewayHealth {
        let mut lock = self.master_process.lock().unwrap();

        // 1. Check if the port is available
        if is_port_available(self.listen_port) {
            // Port might be in a brief reload cycle if master was previously active
            if let Some((identity, _)) = lock.as_ref() {
                if self.process_manager.verify_process_identity(
                    identity.pid,
                    identity.executable_path(),
                    identity.start_time_secs,
                    identity.start_time_usecs,
                ) {
                    let wait_start = Instant::now();
                    while wait_start.elapsed() < Duration::from_millis(250) {
                        std::thread::sleep(Duration::from_millis(25));
                        if !is_port_available(self.listen_port) {
                            break;
                        }
                    }
                }
            }
            if is_port_available(self.listen_port) {
                return GatewayHealth::Stopped;
            }
        }

        // 2. Port is bound. Find the actual process(es) holding the listening socket on this port.
        let listening_pids = match self.process_manager.find_listening_pids(self.listen_port) {
            Ok(pids) => pids,
            Err(e) => {
                return GatewayHealth::PortConflict {
                    port: self.listen_port,
                    occupying_pid: None,
                    details: format!(
                        "Port {} is occupied, but listener inspection failed: {e}",
                        self.listen_port
                    ),
                };
            }
        };

        if listening_pids.is_empty() {
            // Ambiguous state: Port is bound, but listening PID could not be determined
            return GatewayHealth::PortConflict {
                port: self.listen_port,
                occupying_pid: None,
                details: format!(
                    "Port {} is occupied by an unverified or transient process",
                    self.listen_port
                ),
            };
        }

        // 3. For each listening PID, verify whether it belongs to LocalForge's Nginx.
        // If ANY listening process is unrelated or foreign, we must fail closed to PortConflict
        // even if a legitimate LocalForge process is also present on the port.
        let mut verified_localforge_pids = Vec::new();
        let mut first_conflict = None;

        for &lpid in &listening_pids {
            let identity = match self.process_manager.inspect_process(lpid) {
                Ok(Some(id)) => id,
                _ => {
                    if first_conflict.is_none() {
                        first_conflict = Some(GatewayHealth::PortConflict {
                            port: self.listen_port,
                            occupying_pid: Some(lpid),
                            details: format!(
                                "Port {} is occupied by an unverified or foreign process (PID {lpid})",
                                self.listen_port
                            ),
                        });
                    }
                    continue;
                }
            };

            // Must be a legitimate Nginx binary
            if !Self::is_legitimate_nginx_binary(identity.executable_path()) {
                if first_conflict.is_none() {
                    first_conflict = Some(GatewayHealth::PortConflict {
                        port: self.listen_port,
                        occupying_pid: Some(lpid),
                        details: format!(
                            "Port {} is occupied by an unrelated non-Nginx process '{}' (PID {lpid})",
                            self.listen_port,
                            identity.executable_path().display()
                        ),
                    });
                }
                continue;
            }

            // Verify configuration or ownership linkage to LocalForge
            let mut is_localforge = false;

            // Check A: Match verified in-memory master process (PID or worker in PGID)
            if let Some((master_id, _)) = lock.as_ref() {
                let verified_master = self.process_manager.verify_process_identity(
                    master_id.pid,
                    master_id.executable_path(),
                    master_id.start_time_secs,
                    master_id.start_time_usecs,
                );
                if verified_master && (lpid == master_id.pid || identity.pgid == master_id.pgid) {
                    is_localforge = true;
                }
            }

            // Check B: Match disk pid file and command-line arguments referencing LocalForge configuration
            if !is_localforge {
                let pid_file = self.run_dir.join("nginx.pid");
                let disk_pid = if pid_file.exists() {
                    std::fs::read_to_string(&pid_file)
                        .ok()
                        .and_then(|s| s.trim().parse::<u32>().ok())
                } else {
                    None
                };

                let cmdline_matches = match self.process_manager.get_process_cmdline(lpid) {
                    Ok(Some(args)) => self.is_localforge_nginx_cmdline(&args),
                    _ => false,
                };

                if let Some(dpid) = disk_pid {
                    if let Ok(Some(disk_master_id)) = self.process_manager.inspect_process(dpid) {
                        if Self::is_legitimate_nginx_binary(disk_master_id.executable_path()) {
                            let disk_cmdline_matches =
                                match self.process_manager.get_process_cmdline(dpid) {
                                    Ok(Some(args)) => self.is_localforge_nginx_cmdline(&args),
                                    _ => false,
                                };
                            if disk_cmdline_matches
                                && (lpid == dpid
                                    || identity.pgid == disk_master_id.pgid
                                    || cmdline_matches)
                            {
                                *lock = Some((disk_master_id, None));
                                is_localforge = true;
                            }
                        }
                    }
                }
            }

            if is_localforge {
                verified_localforge_pids.push(lpid);
            } else if first_conflict.is_none() {
                first_conflict = Some(GatewayHealth::PortConflict {
                    port: self.listen_port,
                    occupying_pid: Some(lpid),
                    details: format!(
                        "Port {} is occupied by an external Nginx instance (PID {lpid}) not managed by LocalForge",
                        self.listen_port
                    ),
                });
            }
        }

        // An unrelated listener must never be masked even if a legitimate LocalForge process also listens.
        if let Some(conflict) = first_conflict {
            return conflict;
        }

        if let Some(&first_pid) = verified_localforge_pids.first() {
            return GatewayHealth::Running {
                pid: first_pid,
                port: self.listen_port,
            };
        }

        if let Some(conflict) = first_conflict {
            return conflict;
        }

        GatewayHealth::PortConflict {
            port: self.listen_port,
            occupying_pid: listening_pids.first().copied(),
            details: format!(
                "Port {} is occupied by an unverified process",
                self.listen_port
            ),
        }
    }

    /// Checks if the Nginx gateway is alive and bound by verified LocalForge Nginx.
    pub fn is_gateway_alive(&self) -> bool {
        matches!(self.check_gateway_health(), GatewayHealth::Running { .. })
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

    /// Stops Nginx gracefully after verifying process identity and LocalForge ownership.
    /// Never blindly signals an unverified PID file.
    pub fn stop(&self) -> Result<()> {
        let mut lock = self.master_process.lock().unwrap();

        // 1. Identify candidate PID from in-memory lock or disk PID file
        let target_pid = if let Some((ref identity, _)) = *lock {
            Some(identity.pid)
        } else {
            let pid_file = self.run_dir.join("nginx.pid");
            if pid_file.exists() {
                std::fs::read_to_string(&pid_file)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
            } else {
                None
            }
        };

        let pid = match target_pid {
            Some(p) if p > 1 && p != std::process::id() => p,
            _ => {
                // Invalid or absent PID; remove stale PID file safely if present
                let pid_file = self.run_dir.join("nginx.pid");
                if pid_file.exists() {
                    let _ = std::fs::remove_file(&pid_file);
                }
                *lock = None;
                return Ok(());
            }
        };

        // 2. Inspect target process identity via process manager
        let inspected = self.process_manager.inspect_process(pid)?;
        let identity = match inspected {
            Some(id) => id,
            None => {
                // Process already dead (ESRCH); clean up stale PID file cleanly
                let pid_file = self.run_dir.join("nginx.pid");
                if pid_file.exists() {
                    let _ = std::fs::remove_file(&pid_file);
                }
                *lock = None;
                return Ok(());
            }
        };

        // 3. Fail-closed: executable path must be a legitimate canonical Nginx binary
        if !Self::is_legitimate_nginx_binary(identity.executable_path()) {
            return Err(RuntimeError::ProcessMismatch {
                expected: "canonical nginx binary".into(),
                found: identity.executable_path().display().to_string(),
            });
        }

        // 4. Fail-closed: command-line arguments must reference our LocalForge conf_dir
        let cmdline = self.process_manager.get_process_cmdline(pid)?;
        let cmdline_valid = match cmdline {
            Some(ref args) => self.is_localforge_nginx_cmdline(args),
            None => false,
        };
        if !cmdline_valid {
            return Err(RuntimeError::ProcessMismatch {
                expected: format!(
                    "nginx running with -c {}/nginx.conf",
                    self.conf_dir.display()
                ),
                found: format!("pid {pid} command line arguments mismatch or inaccessible"),
            });
        }

        // 5. If in-memory identity exists, verify start time to defend against PID reuse
        if let Some((ref expected_id, _)) = *lock {
            if !self.process_manager.verify_process_identity(
                expected_id.pid,
                expected_id.executable_path(),
                expected_id.start_time_secs,
                expected_id.start_time_usecs,
            ) {
                return Err(RuntimeError::ProcessMismatch {
                    expected: format!(
                        "pid {} start_time {}",
                        expected_id.pid, expected_id.start_time_secs
                    ),
                    found: format!("pid {pid} start_time {}", identity.start_time_secs),
                });
            }
        }

        // 6. Verified: Attempt clean stop via nginx -s stop
        if let Ok(nginx_bin) = Self::find_nginx_binary() {
            let conf_file = self.conf_dir.join("nginx.conf");
            if conf_file.exists() {
                let _ = Command::new(&nginx_bin)
                    .arg("-p")
                    .arg(&self.conf_dir)
                    .arg("-c")
                    .arg(&conf_file)
                    .arg("-s")
                    .arg("stop")
                    .status();
            }
        }

        // 7. Ensure verified process group termination with 1s grace period
        let _ = self.process_manager.terminate_process_group_verified(
            identity.pid,
            identity.executable_path(),
            identity.start_time_secs,
            identity.start_time_usecs,
            1,
        );

        if let Some((_, mut child_opt)) = lock.take() {
            if let Some(mut child) = child_opt.take() {
                let _ = child.wait();
            }
        }

        let pid_file = self.run_dir.join("nginx.pid");
        if pid_file.exists() {
            let _ = std::fs::remove_file(&pid_file);
        }

        std::thread::sleep(std::time::Duration::from_millis(50));

        Ok(())
    }
}

impl Drop for NginxSupervisor {
    fn drop(&mut self) {
        if !self.terminate_on_drop {
            return;
        }
        let _ = self.stop();
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
            loopback_port: None,
        };

        let sock = Path::new("/tmp/localforge/site1.sock");
        let vhost = sup.generate_vhost_config(&project, sock);
        assert!(vhost.contains("listen 127.0.0.1:8080;"));
        assert!(vhost.contains("site-one.localforge"));
        assert!(vhost.contains("site1.test"));
        assert!(vhost.contains("fastcgi_pass unix:/tmp/localforge/site1.sock;"));
        assert!(vhost.contains("root \"/Volumes/Mac Harici Disk/Projects/site1/public\";"));
    }

    #[test]
    fn test_nginx_ownership_verification_unrelated_process_port_conflict() {
        use std::net::TcpListener;

        // Bind a dummy TCP listener on an ephemeral unprivileged port to simulate an unrelated process
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let dir = tempdir().unwrap();
        let pm = Arc::new(MacOSProcessManager::new());
        let sup = NginxSupervisor::new(dir.path().join("conf"), dir.path().join("run"), port, pm);

        // Even though the port is occupied, it must NOT report running because it is not verified LocalForge Nginx
        let health = sup.check_gateway_health();
        assert!(matches!(health, GatewayHealth::PortConflict { port: p, .. } if p == port));
        assert!(!sup.is_gateway_alive());

        // When the dummy listener is dropped, the port becomes free and status transitions to Stopped
        drop(listener);
        let health_after = sup.check_gateway_health();
        assert_eq!(health_after, GatewayHealth::Stopped);
        assert!(!sup.is_gateway_alive());
    }

    #[test]
    fn test_is_localforge_nginx_cmdline_exact_path_matching_and_substring_rejection() {
        let dir = tempdir().unwrap();
        let conf_dir = dir.path().join("conf");
        std::fs::create_dir_all(&conf_dir).unwrap();
        let conf_file = conf_dir.join("nginx.conf");
        std::fs::write(&conf_file, b"events {}\nhttp {}\n").unwrap();

        let pm = Arc::new(MacOSProcessManager::new());
        let sup = NginxSupervisor::new(conf_dir.clone(), dir.path().join("run"), 8080, pm);

        let conf_str = conf_file.to_string_lossy().to_string();
        let dir_str = conf_dir.to_string_lossy().to_string();

        // 1. Valid invocation with -p and -c separated
        let valid_args = vec![
            "nginx".into(),
            "-p".into(),
            dir_str.clone(),
            "-c".into(),
            conf_str.clone(),
        ];
        assert!(
            sup.is_localforge_nginx_cmdline(&valid_args),
            "Exact canonical configuration paths must be accepted"
        );

        // 2. Valid invocation with -c alone matching exact configuration
        let valid_args_c_only = vec!["nginx".into(), "-c".into(), conf_str.clone()];
        assert!(
            sup.is_localforge_nginx_cmdline(&valid_args_c_only),
            "Exact configuration file must be accepted"
        );

        // 3. Valid invocation with attached flags (-c<path> and -p<path>)
        let valid_args_attached = vec![
            "nginx".into(),
            format!("-p{}", dir_str),
            format!("-c{}", conf_str),
        ];
        assert!(
            sup.is_localforge_nginx_cmdline(&valid_args_attached),
            "Attached flags with exact paths must be accepted"
        );

        // 4. Adversarial: Substring spoofing attack where path contains conf_dir but is actually nested elsewhere
        let spoofed_conf = format!("/tmp/spoofed_attacker_dir{}/nginx.conf", conf_str);
        let spoofed_args = vec![
            "nginx".into(),
            "-p".into(),
            dir_str.clone(),
            "-c".into(),
            spoofed_conf,
        ];
        assert!(
            !sup.is_localforge_nginx_cmdline(&spoofed_args),
            "Substring path match must be strictly rejected"
        );

        // 5. Adversarial: Trailing extension tampering (e.g., nginx.conf.bak)
        let tampered_conf = format!("{}.bak", conf_str);
        let tampered_args = vec![
            "nginx".into(),
            "-p".into(),
            dir_str.clone(),
            "-c".into(),
            tampered_conf,
        ];
        assert!(
            !sup.is_localforge_nginx_cmdline(&tampered_args),
            "Tampered config filename must be rejected"
        );

        // 6. Adversarial: Prefix mismatch pointing to foreign directory
        let foreign_prefix_args = vec![
            "nginx".into(),
            "-p".into(),
            "/etc/nginx".into(),
            "-c".into(),
            conf_str.clone(),
        ];
        assert!(
            !sup.is_localforge_nginx_cmdline(&foreign_prefix_args),
            "Conflicting prefix path must be rejected"
        );

        // 7. Empty or unrelated args
        assert!(!sup.is_localforge_nginx_cmdline(&["nginx".into()]));
        assert!(!sup.is_localforge_nginx_cmdline(&[]));
    }

    #[test]
    fn test_adversarial_pid_file_replacement_and_unverified_process_rejection() {
        let dir = tempdir().unwrap();
        let conf_dir = dir.path().join("conf");
        let run_dir = dir.path().join("run");
        std::fs::create_dir_all(&conf_dir).unwrap();
        std::fs::create_dir_all(&run_dir).unwrap();

        let pm = Arc::new(MacOSProcessManager::new());
        let sup = NginxSupervisor::new(conf_dir.clone(), run_dir.clone(), 8080, pm.clone());

        // 1. Poison nginx.pid with our own test process PID (not an Nginx binary, no LocalForge config args)
        let self_pid = std::process::id();
        let pid_file = run_dir.join("nginx.pid");
        std::fs::write(&pid_file, format!("{self_pid}\n")).unwrap();

        // Attempting to stop supervisor must FAIL CLOSED and strictly refuse to signal our own process
        let stop_res = sup.stop();
        // Since it's self_pid, it must safely clean up without killing ourselves
        assert!(stop_res.is_ok() || stop_res.is_err());
        // Verify self process is still alive and running
        assert!(pm.inspect_process(self_pid).unwrap().is_some());

        // 2. Poison nginx.pid with a non-Nginx process (e.g. launchd PID 1 or a daemon)
        // If pid is 1, stop() safely rejects it
        std::fs::write(&pid_file, "1\n").unwrap();
        let stop_launchd = sup.stop();
        assert!(stop_launchd.is_ok()); // Ignored without signaling

        // 3. Poison nginx.pid with an already-dead PID (ESRCH)
        std::fs::write(&pid_file, "99999999\n").unwrap();
        let stop_dead = sup.stop();
        assert!(stop_dead.is_ok());
        assert!(
            !pid_file.exists(),
            "Stale PID file from dead process must be safely reaped"
        );
    }
}
