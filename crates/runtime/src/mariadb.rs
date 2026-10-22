use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use localforge_platform::{ProcessIdentity, ProcessManager};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MariaDbBinaries {
    pub daemon: PathBuf,
    pub client: Option<PathBuf>,
    pub install_db: Option<PathBuf>,
    pub dump: Option<PathBuf>,
    pub admin: Option<PathBuf>,
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseHealthStatus {
    pub is_alive: bool,
    pub socket_path: PathBuf,
    pub port: u16,
    pub version: Option<String>,
    pub latency_ms: Option<u64>,
    pub message: String,
}

/// Unapproved database deletion request handle.
/// NOTE: A deletion request ID is an audit handle, NOT an authorization credential.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatabaseDeletionRequest {
    pub request_id: String,
    pub database_name: String,
    pub requested_at_secs: u64,
    pub expires_at_secs: u64,
    pub requesting_context: String,
}

/// Kept for backward compatibility with existing interfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DatabaseDeletionApproval {
    pub database_name: String,
    pub token: String,
    pub issued_at_secs: u64,
    pub expires_at_secs: u64,
    pub requesting_context: String,
}

/// Reliably resolves the current user's Application Support directory without hardcoded usernames.
/// Priority:
/// 1. $HOME environment variable if non-empty and valid.
/// 2. POSIX / Darwin system user database (`libc::getpwuid(libc::getuid())`).
/// 3. Fail-closed: returns RuntimeError::ConfigurationError if neither can be resolved.
pub fn resolve_application_support_dir() -> Result<PathBuf> {
    if let Ok(home_val) = std::env::var("HOME") {
        let trimmed = home_val.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            return Ok(p
                .join("Library")
                .join("Application Support")
                .join("LocalForge"));
        }
    }

    #[cfg(unix)]
    unsafe {
        let uid = libc::getuid();
        let pw = libc::getpwuid(uid);
        if !pw.is_null() && !(*pw).pw_dir.is_null() {
            if let Ok(dir_str) = std::ffi::CStr::from_ptr((*pw).pw_dir).to_str() {
                let trimmed = dir_str.trim();
                if !trimmed.is_empty() {
                    let p = PathBuf::from(trimmed);
                    return Ok(p
                        .join("Library")
                        .join("Application Support")
                        .join("LocalForge"));
                }
            }
        }
    }

    Err(RuntimeError::ConfigurationError(
        "Failed to resolve current user Application Support directory: neither $HOME nor getpwuid() produced a valid home directory.".into(),
    ))
}

/// Resolves default MariaDB storage directory under user Application Support.
pub fn resolve_mariadb_base_dir() -> Result<PathBuf> {
    let app_support = resolve_application_support_dir()?;
    Ok(app_support.join("mariadb"))
}

pub struct MariaDbSupervisor {
    base_dir: PathBuf,
    data_dir: PathBuf,
    run_dir: PathBuf,
    port: std::sync::atomic::AtomicU16,
    socket_path: PathBuf,
    pid_file: PathBuf,
    config_file: PathBuf,
    process_manager: Arc<dyn ProcessManager>,
    master_process: Mutex<Option<(ProcessIdentity, Option<Child>)>>,
    pending_deletion_requests: Mutex<HashMap<String, DatabaseDeletionRequest>>,
    pub terminate_on_drop: bool,
}

impl MariaDbSupervisor {
    pub fn new(
        base_dir: PathBuf,
        preferred_port: u16,
        process_manager: Arc<dyn ProcessManager>,
    ) -> Self {
        let data_dir = base_dir.join("data");
        let run_dir = base_dir.join("run");
        let socket_path = run_dir.join("mariadb.sock");
        let pid_file = run_dir.join("mariadb.pid");
        let config_file = base_dir.join("my.cnf");

        Self {
            base_dir,
            data_dir,
            run_dir,
            port: std::sync::atomic::AtomicU16::new(preferred_port),
            socket_path,
            pid_file,
            config_file,
            process_manager,
            master_process: Mutex::new(None),
            pending_deletion_requests: Mutex::new(HashMap::new()),
            terminate_on_drop: false,
        }
    }

    /// Convenience constructor for testing or standalone usage on macOS.
    pub fn new_simple(base_dir: PathBuf, preferred_port: u16) -> Self {
        #[cfg(target_os = "macos")]
        {
            let pm = Arc::new(localforge_platform::MacOSProcessManager::new());
            Self::new(base_dir, preferred_port, pm)
        }
        #[cfg(not(target_os = "macos"))]
        {
            panic!("new_simple only supported on macOS");
        }
    }

    /// Default constructor pointing to internal storage under macOS Application Support.
    /// Fails safely if user Application Support directory cannot be resolved.
    pub fn create_default(
        process_manager: Arc<dyn ProcessManager>,
        preferred_port: u16,
    ) -> Result<Self> {
        let base_dir = resolve_mariadb_base_dir()?;
        Ok(Self::new(base_dir, preferred_port, process_manager))
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn run_dir(&self) -> &Path {
        &self.run_dir
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn pid_file(&self) -> &Path {
        &self.pid_file
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn port(&self) -> u16 {
        self.port.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Checks whether the data directory contains an initialized MariaDB system schema ('mysql/').
    pub fn is_datadir_initialized(&self) -> bool {
        self.data_dir.join("mysql").is_dir()
    }

    /// Discovers installed MariaDB / MySQL binaries on the host without installing anything.
    pub fn discover_binaries() -> Result<MariaDbBinaries> {
        let daemon_candidates = [
            "/opt/homebrew/bin/mariadbd",
            "/opt/homebrew/sbin/mariadbd",
            "/opt/homebrew/opt/mariadb/bin/mariadbd",
            "/opt/homebrew/bin/mysqld",
            "/opt/homebrew/sbin/mysqld",
            "/usr/local/bin/mariadbd",
            "/usr/local/sbin/mariadbd",
            "/usr/local/bin/mysqld",
            "/usr/local/sbin/mysqld",
        ];

        let mut daemon_path = None;
        for c in daemon_candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                daemon_path = Some(p);
                break;
            }
        }

        if daemon_path.is_none() {
            if let Ok(output) = Command::new("which").arg("mariadbd").output() {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        daemon_path = Some(PathBuf::from(s));
                    }
                }
            }
        }

        if daemon_path.is_none() {
            if let Ok(output) = Command::new("which").arg("mysqld").output() {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        daemon_path = Some(PathBuf::from(s));
                    }
                }
            }
        }

        let daemon = daemon_path.ok_or_else(|| RuntimeError::RuntimeNotFound {
            binary: "mariadbd/mysqld".into(),
            version: "any".into(),
        })?;

        // Extract version
        let version = match Command::new(&daemon).arg("--version").output() {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                text.lines()
                    .next()
                    .unwrap_or("MariaDB unknown")
                    .trim()
                    .to_string()
            }
            Err(_) => "MariaDB unknown".to_string(),
        };

        // Discover client CLI
        let client_candidates = [
            "/opt/homebrew/bin/mariadb",
            "/opt/homebrew/opt/mariadb/bin/mariadb",
            "/opt/homebrew/bin/mysql",
            "/usr/local/bin/mariadb",
            "/usr/local/bin/mysql",
        ];
        let mut client = None;
        for c in client_candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                client = Some(p);
                break;
            }
        }
        if client.is_none() {
            if let Ok(output) = Command::new("which").arg("mariadb").output() {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        client = Some(PathBuf::from(s));
                    }
                }
            }
        }

        // Discover install-db
        let install_candidates = [
            "/opt/homebrew/bin/mariadb-install-db",
            "/opt/homebrew/opt/mariadb/bin/mariadb-install-db",
            "/opt/homebrew/bin/mysql_install_db",
            "/usr/local/bin/mariadb-install-db",
            "/usr/local/bin/mysql_install_db",
        ];
        let mut install_db = None;
        for c in install_candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                install_db = Some(p);
                break;
            }
        }
        if install_db.is_none() {
            if let Ok(output) = Command::new("which").arg("mariadb-install-db").output() {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !s.is_empty() {
                        install_db = Some(PathBuf::from(s));
                    }
                }
            }
        }

        // Discover dump utility
        let dump_candidates = [
            "/opt/homebrew/bin/mariadb-dump",
            "/opt/homebrew/opt/mariadb/bin/mariadb-dump",
            "/opt/homebrew/bin/mysqldump",
            "/usr/local/bin/mariadb-dump",
            "/usr/local/bin/mysqldump",
        ];
        let mut dump = None;
        for c in dump_candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                dump = Some(p);
                break;
            }
        }

        // Discover mysqladmin
        let admin_candidates = [
            "/opt/homebrew/bin/mysqladmin",
            "/opt/homebrew/opt/mariadb/bin/mysqladmin",
            "/usr/local/bin/mysqladmin",
        ];
        let mut admin = None;
        for c in admin_candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                admin = Some(p);
                break;
            }
        }

        Ok(MariaDbBinaries {
            daemon,
            client,
            install_db,
            dump,
            admin,
            version,
        })
    }

    /// Backwards compatible find_binary returning daemon executable.
    pub fn find_binary() -> Result<PathBuf> {
        Self::discover_binaries().map(|b| b.daemon)
    }

    /// Generates isolated `my.cnf` configuration strictly bounded to this supervisor.
    pub fn generate_config(&self) -> String {
        format!(
            r#"[mysqld]
datadir = {data_dir}
socket = {socket_path}
pid-file = {pid_file}
port = {port}
bind-address = 127.0.0.1
skip-name-resolve
default_storage_engine = InnoDB
innodb_buffer_pool_size = 64M
innodb_file_per_table = 1
character-set-server = utf8mb4
collation-server = utf8mb4_unicode_ci

[client]
socket = {socket_path}
port = {port}
user = root
"#,
            data_dir = self.data_dir.display(),
            socket_path = self.socket_path.display(),
            pid_file = self.pid_file.display(),
            port = self.port(),
        )
    }

    /// Initializes data directory if not already created. Never overwrites existing tables or unknown directories.
    pub fn ensure_datadir_initialized(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir).map_err(RuntimeError::Io)?;
        std::fs::create_dir_all(&self.data_dir).map_err(RuntimeError::Io)?;
        std::fs::create_dir_all(&self.run_dir).map_err(RuntimeError::Io)?;

        // Validate datadir ownership on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let meta = std::fs::metadata(&self.data_dir).map_err(RuntimeError::Io)?;
            let current_uid = unsafe { libc::getuid() };
            if meta.uid() != current_uid {
                return Err(RuntimeError::DatabaseError(format!(
                    "Datadir ownership mismatch: '{}' is owned by UID {} but current process is running as UID {}",
                    self.data_dir.display(),
                    meta.uid(),
                    current_uid
                )));
            }
        }

        // Check if database is already initialized
        if self.data_dir.join("mysql").is_dir() {
            // Ensure config exists
            if !self.config_file.exists() {
                std::fs::write(&self.config_file, self.generate_config())
                    .map_err(RuntimeError::Io)?;
            }
            return Ok(());
        }

        // Refuse to initialize into an existing non-empty directory that lacks the mysql/ schema
        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            let non_hidden_entries: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .collect();
            if !non_hidden_entries.is_empty() {
                return Err(RuntimeError::DatabaseError(format!(
                    "Existing datadir '{}' contains unexpected files but lacks MariaDB system schema ('mysql/'). Refusing to overwrite unknown data directory.",
                    self.data_dir.display()
                )));
            }
        }

        // Write configuration file
        std::fs::write(&self.config_file, self.generate_config()).map_err(RuntimeError::Io)?;

        let binaries = Self::discover_binaries()?;

        if let Some(install_bin) = binaries.install_db {
            // First attempt with --auth-root-authentication-method=normal
            let mut cmd = Command::new(&install_bin);
            cmd.arg(format!("--defaults-file={}", self.config_file.display()));
            cmd.arg(format!("--datadir={}", self.data_dir.display()));
            cmd.arg("--auth-root-authentication-method=normal");
            cmd.arg("--skip-test-db");

            let res = cmd.output();
            if let Ok(out) = res {
                if out.status.success() || self.data_dir.join("mysql").is_dir() {
                    return Ok(());
                }
            }

            // Fallback attempt without auth-root parameter (older mysql_install_db)
            let mut fallback = Command::new(&install_bin);
            fallback.arg(format!("--defaults-file={}", self.config_file.display()));
            fallback.arg(format!("--datadir={}", self.data_dir.display()));
            let fallback_res = fallback.output();
            if let Ok(out) = fallback_res {
                if out.status.success() || self.data_dir.join("mysql").is_dir() {
                    return Ok(());
                }
            }
        }

        // Second fallback: direct daemon initialize-insecure
        let mut daemon_init = Command::new(&binaries.daemon);
        daemon_init.arg(format!("--defaults-file={}", self.config_file.display()));
        daemon_init.arg("--initialize-insecure");
        daemon_init.arg(format!("--datadir={}", self.data_dir.display()));
        if let Ok(out) = daemon_init.output() {
            if out.status.success() || self.data_dir.join("mysql").is_dir() {
                return Ok(());
            }
        }

        if !self.data_dir.join("mysql").is_dir() {
            return Err(RuntimeError::ConfigGenerationFailed(
                "Failed to initialize MariaDB data directory with mariadb-install-db".into(),
            ));
        }

        Ok(())
    }

    /// Starts MariaDB daemon under current unprivileged user.
    pub fn start(&self) -> Result<ProcessIdentity> {
        let mut lock = self.master_process.lock().unwrap();

        // 1. Check if already active and verified in memory
        if let Some((identity, ref mut child_opt)) = lock.as_mut() {
            let verified = self.process_manager.verify_process_identity(
                identity.pid,
                identity.executable_path(),
                identity.start_time_secs,
                identity.start_time_usecs,
            );
            if verified && crate::health::is_unix_socket_alive(&self.socket_path) {
                return Ok(identity.clone());
            } else {
                // Dead or corrupted; clean up
                if let Some(child) = child_opt.as_mut() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                *lock = None;
            }
        }

        // 2. Discover binaries and validate
        let binaries = Self::discover_binaries()?;
        let canonical_bin =
            dunce::canonicalize(&binaries.daemon).unwrap_or(binaries.daemon.clone());

        // 3. Check for existing PID file on disk
        if self.pid_file.exists() && crate::health::is_unix_socket_alive(&self.socket_path) {
            if let Ok(pid_str) = std::fs::read_to_string(&self.pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                        if identity.executable_path() == canonical_bin {
                            *lock = Some((identity.clone(), None));
                            return Ok(identity);
                        }
                    }
                }
            }
        }

        // 4. Safety Guard: Never automatically initialize production datadir without authorization
        if let Ok(default_base) = resolve_mariadb_base_dir() {
            if self.base_dir == default_base && !self.is_datadir_initialized() {
                return Err(RuntimeError::DatabaseError(
                    "MariaDB veri dizini başlatılmamış. Güvenlik politikası ve Product Owner direktifi gereği otomatik disk ilklendirme durdurulmuştur. Lütfen yetkilendirme talep ediniz.".into(),
                ));
            }
        }
        self.ensure_datadir_initialized()?;

        // 5. Port conflict check & unprivileged fallback
        let current_port = self.port();
        if !is_port_available(current_port) {
            if let Some(free_port) = crate::health::find_available_port(current_port + 1, 100) {
                self.port
                    .store(free_port, std::sync::atomic::Ordering::SeqCst);
                // Re-write config with adjusted port
                let _ = std::fs::write(&self.config_file, self.generate_config());
            } else {
                return Err(RuntimeError::PortConflict {
                    port: current_port,
                    details: "No available TCP port found for MariaDB daemon".into(),
                });
            }
        }

        // 6. Clean stale socket file
        if self.socket_path.exists() && !crate::health::is_unix_socket_alive(&self.socket_path) {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        // 7. Spawn child process in dedicated process group (unprivileged)
        #[cfg(unix)]
        let child = {
            use std::os::unix::process::CommandExt;
            let mut cmd = Command::new(&binaries.daemon);
            cmd.arg(format!("--defaults-file={}", self.config_file.display()));
            cmd.process_group(0);
            cmd.spawn()
        };

        #[cfg(not(unix))]
        let child = Command::new(&binaries.daemon)
            .arg(format!("--defaults-file={}", self.config_file.display()))
            .spawn();

        let mut child = child.map_err(|e| RuntimeError::ProcessSpawnFailed {
            process: "mariadbd".into(),
            reason: e.to_string(),
        })?;

        let pid = child.id();
        let _ = std::fs::write(&self.pid_file, pid.to_string());

        // 8. Inspect process identity immediately
        let identity = match self.process_manager.inspect_process(pid)? {
            Some(id) => id,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: "mariadbd".into(),
                    reason: "MariaDB daemon process terminated immediately upon launch".into(),
                });
            }
        };

        // 9. Wait for socket readiness (bounded window up to 8 seconds)
        let wait_start = Instant::now();
        let mut socket_ready = false;
        while wait_start.elapsed() < Duration::from_secs(8) {
            if crate::health::is_unix_socket_alive(&self.socket_path) {
                socket_ready = true;
                break;
            }
            if let Ok(Some(status)) = child.try_wait() {
                return Err(RuntimeError::ProcessSpawnFailed {
                    process: "mariadbd".into(),
                    reason: format!("MariaDB exited prematurely with status: {status}"),
                });
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        if !socket_ready {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RuntimeError::ServiceUnhealthy {
                service: "mariadb".into(),
                reason: format!(
                    "Timed out waiting for socket at {} after 8 seconds",
                    self.socket_path.display()
                ),
            });
        }

        *lock = Some((identity.clone(), Some(child)));
        Ok(identity)
    }

    /// Stops MariaDB process cleanly with bounded grace period.
    pub fn stop(&self) -> Result<()> {
        let mut lock = self.master_process.lock().unwrap();

        // Check if mysqladmin shutdown is feasible
        if let Ok(binaries) = Self::discover_binaries() {
            if let Some(admin_bin) = binaries.admin {
                if crate::health::is_unix_socket_alive(&self.socket_path) {
                    let _ = Command::new(admin_bin)
                        .arg(format!("--socket={}", self.socket_path.display()))
                        .arg("-u")
                        .arg("root")
                        .arg("shutdown")
                        .output();

                    // Wait up to 3 seconds for graceful shutdown
                    let start = Instant::now();
                    while start.elapsed() < Duration::from_secs(3) {
                        if !crate::health::is_unix_socket_alive(&self.socket_path) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        }

        if let Some((identity, mut child_opt)) = lock.take() {
            if let Some(mut child) = child_opt.take() {
                if let Ok(Some(_)) = child.try_wait() {
                    // Process already terminated cleanly via mysqladmin
                } else {
                    #[cfg(unix)]
                    unsafe {
                        libc::kill(identity.pid as i32, libc::SIGTERM);
                    }
                    let wait_start = Instant::now();
                    let mut exited = false;
                    while wait_start.elapsed() < Duration::from_secs(3) {
                        if let Ok(Some(_)) = child.try_wait() {
                            exited = true;
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    if !exited {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
            } else {
                let _ = self.process_manager.terminate_process_group_verified(
                    identity.pid,
                    identity.executable_path(),
                    identity.start_time_secs,
                    identity.start_time_usecs,
                    2,
                );
            }
        } else if self.pid_file.exists() {
            if let Ok(pid_str) = std::fs::read_to_string(&self.pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if let Ok(Some(identity)) = self.process_manager.inspect_process(pid) {
                        let _ = self.process_manager.terminate_process_group_verified(
                            identity.pid,
                            identity.executable_path(),
                            identity.start_time_secs,
                            identity.start_time_usecs,
                            2,
                        );
                    }
                }
            }
        }

        let _ = std::fs::remove_file(&self.pid_file);
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        Ok(())
    }

    /// Performs authenticated SQL health check via socket.
    /// Never labels RUNNING solely because port is open!
    pub fn check_health(&self) -> Result<DatabaseHealthStatus> {
        let socket_alive = crate::health::is_unix_socket_alive(&self.socket_path);
        if !socket_alive {
            return Ok(DatabaseHealthStatus {
                is_alive: false,
                socket_path: self.socket_path.clone(),
                port: self.port(),
                version: None,
                latency_ms: None,
                message: "MariaDB Unix domain socket unavailable or process inactive.".into(),
            });
        }

        let binaries = match Self::discover_binaries() {
            Ok(b) => b,
            Err(e) => {
                return Ok(DatabaseHealthStatus {
                    is_alive: false,
                    socket_path: self.socket_path.clone(),
                    port: self.port(),
                    version: None,
                    latency_ms: None,
                    message: format!("Binaries discovery failed during health check: {e}"),
                });
            }
        };

        let start = Instant::now();
        // Execute real SQL probe: SELECT 1;
        if let Some(client_bin) = binaries.client {
            let mut cmd = Command::new(client_bin);
            cmd.arg(format!("--socket={}", self.socket_path.display()));
            cmd.arg("-u");
            cmd.arg("root");
            cmd.arg("-e");
            cmd.arg("SELECT 1;");

            match cmd.output() {
                Ok(out) if out.status.success() => {
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(DatabaseHealthStatus {
                        is_alive: true,
                        socket_path: self.socket_path.clone(),
                        port: self.port(),
                        version: Some(binaries.version),
                        latency_ms: Some(latency),
                        message: format!("MariaDB verified alive and responsive over Unix socket ({latency} ms)."),
                    });
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Ok(DatabaseHealthStatus {
                        is_alive: false,
                        socket_path: self.socket_path.clone(),
                        port: self.port(),
                        version: Some(binaries.version),
                        latency_ms: None,
                        message: format!("SQL ping failed: {err}"),
                    });
                }
                Err(e) => {
                    return Ok(DatabaseHealthStatus {
                        is_alive: false,
                        socket_path: self.socket_path.clone(),
                        port: self.port(),
                        version: Some(binaries.version),
                        latency_ms: None,
                        message: format!("Failed to execute SQL client: {e}"),
                    });
                }
            }
        }

        // Fallback: ping via mysqladmin
        if let Some(admin_bin) = binaries.admin {
            let mut cmd = Command::new(admin_bin);
            cmd.arg(format!("--socket={}", self.socket_path.display()));
            cmd.arg("-u");
            cmd.arg("root");
            cmd.arg("ping");

            if let Ok(out) = cmd.output() {
                if out.status.success() {
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(DatabaseHealthStatus {
                        is_alive: true,
                        socket_path: self.socket_path.clone(),
                        port: self.port(),
                        version: Some(binaries.version),
                        latency_ms: Some(latency),
                        message: format!("MariaDB admin ping succeeded ({latency} ms)."),
                    });
                }
            }
        }

        Ok(DatabaseHealthStatus {
            is_alive: false,
            socket_path: self.socket_path.clone(),
            port: self.port(),
            version: Some(binaries.version),
            latency_ms: None,
            message: "Unable to verify SQL readiness through client tools.".into(),
        })
    }

    /// Executes raw SQL query via client CLI through the Unix domain socket, redacting secret from errors.
    fn execute_query_internal(&self, sql: &str, secret_to_redact: Option<&str>) -> Result<String> {
        let binaries = Self::discover_binaries()?;
        let client = binaries
            .client
            .ok_or_else(|| RuntimeError::RuntimeNotFound {
                binary: "mariadb/mysql client".into(),
                version: "any".into(),
            })?;

        let mut child = Command::new(client)
            .arg(format!("--socket={}", self.socket_path.display()))
            .arg("-u")
            .arg("root")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RuntimeError::DatabaseError(format!("Failed to spawn client: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(sql.as_bytes())
                .map_err(|e| RuntimeError::DatabaseError(format!("Failed to write SQL: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| RuntimeError::DatabaseError(format!("Query execution failed: {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            let sanitized_err = if let Some(secret) = secret_to_redact {
                if !secret.is_empty() {
                    err.replace(secret, "[REDACTED_PASSWORD]")
                } else {
                    err.into_owned()
                }
            } else {
                err.into_owned()
            };
            return Err(RuntimeError::DatabaseError(format!(
                "SQL execution error: {sanitized_err}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Executes raw SQL query via client CLI through the Unix domain socket.
    pub fn execute_query(&self, sql: &str) -> Result<String> {
        self.execute_query_internal(sql, None)
    }

    /// Creates an isolated database and dedicated project user with securely escaped credentials.
    /// Never logs plaintext passwords or allows direct SQL string interpolation injection.
    pub fn create_project_database(&self, db_name: &str, user: &str, pass: &str) -> Result<()> {
        Self::validate_identifier(db_name)?;
        Self::validate_identifier(user)?;

        if pass.is_empty() {
            return Err(RuntimeError::CredentialError(
                "Database password cannot be empty".into(),
            ));
        }
        if pass.len() > 256 {
            return Err(RuntimeError::CredentialError(
                "Database password exceeds maximum permitted length of 256 characters".into(),
            ));
        }

        let escaped_pass = escape_sql_string_literal(pass)?;

        let sql = format!(
            r#"CREATE DATABASE IF NOT EXISTS `{db}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER IF NOT EXISTS '{user}'@'localhost' IDENTIFIED BY '{pass}';
CREATE USER IF NOT EXISTS '{user}'@'127.0.0.1' IDENTIFIED BY '{pass}';
GRANT ALL PRIVILEGES ON `{db}`.* TO '{user}'@'localhost';
GRANT ALL PRIVILEGES ON `{db}`.* TO '{user}'@'127.0.0.1';
FLUSH PRIVILEGES;
"#,
            db = db_name,
            user = user,
            pass = escaped_pass,
        );

        self.execute_query_internal(&sql, Some(pass))?;
        Ok(())
    }

    /// Submits a database deletion request for review.
    ///
    /// SECURITY INVARIANT:
    /// Returns a `DatabaseDeletionRequest` with a `request_id`.
    /// The `request_id` is an audit handle, NOT an authorization decision.
    /// The requesting caller CANNOT self-authorize deletion with this handle.
    pub fn request_database_deletion_approval(
        &self,
        db_name: &str,
        requesting_context: &str,
    ) -> Result<DatabaseDeletionRequest> {
        Self::validate_identifier(db_name)?;
        if requesting_context.trim().is_empty() {
            return Err(RuntimeError::DatabaseUnauthorized(
                "Requesting context cannot be empty for destructive operations.".into(),
            ));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let req_id = format!("del_req_{}_{}", db_name, uuid::Uuid::new_v4().simple());
        let request = DatabaseDeletionRequest {
            request_id: req_id.clone(),
            database_name: db_name.to_string(),
            requested_at_secs: now,
            expires_at_secs: now + 60,
            requesting_context: requesting_context.to_string(),
        };

        let mut lock = self.pending_deletion_requests.lock().unwrap();
        // Clean expired requests
        lock.retain(|_, v| v.expires_at_secs >= now);
        lock.insert(req_id, request.clone());

        Ok(request)
    }

    /// Drops project database strictly requiring verified out-of-band human authorization.
    ///
    /// SECURITY INVARIANTS:
    /// 1. Self-authorization using the database name (`approval_token == db_name`) is strictly rejected.
    /// 2. Self-authorization using an unapproved request ID (`approval_token.starts_with("del_req_")`) is strictly rejected.
    /// 3. In Phase R1.2B, because a trusted desktop human approval interface has not yet been
    ///    integrated (scheduled for Phase R1.3 Spatial Studio), database deletion is explicitly kept DISABLED.
    /// 4. Returns a typed `DatabaseDeletionUnavailable` or `DatabaseUnauthorized` error.
    /// 5. No database will be dropped in Phase R1.2B.
    pub fn drop_project_database(&self, db_name: &str, approval_token: &str) -> Result<()> {
        Self::validate_identifier(db_name)?;

        // 1. Explicitly catch and reject self-approval attempt (using database name as confirmation token)
        if db_name == approval_token {
            return Err(RuntimeError::DatabaseUnauthorized(
                "Self-authorization rejected: using database name as confirmation token is strictly prohibited.".into(),
            ));
        }

        // 2. Explicitly reject attempts to self-authorize using a deletion request handle
        if approval_token.starts_with("del_req_") {
            return Err(RuntimeError::DatabaseUnauthorized(
                "Self-authorization rejected: a deletion request identifier is not a trusted human approval decision. Separate approval requests from approval decisions.".into(),
            ));
        }

        // 3. Fail-closed: Phase R1.2B keeps database deletion explicitly disabled
        // pending trusted desktop human approval interface integration.
        Err(RuntimeError::DatabaseDeletionUnavailable(format!(
            "Database deletion is disabled in Phase R1.2B for target '{db_name}'. A trusted desktop human approval interface is required and will be integrated in Phase R1.3 Spatial Studio."
        )))
    }

    /// Performs logical database dump to target file with atomic temp file and immediate 0600 permissions.
    /// Streams output directly to disk to prevent unbounded application memory buffering.
    /// Never silently overwrites an existing backup.
    pub fn dump_database(&self, db_name: &str, target_file: &Path) -> Result<()> {
        Self::validate_identifier(db_name)?;

        // 1. Prevent silent overwrite of existing backups
        if target_file.exists() {
            return Err(RuntimeError::BackupError(format!(
                "Backup target file already exists: {}. Refusing to overwrite existing backup.",
                target_file.display()
            )));
        }

        // 2. Path traversal & symlink protection
        if let Ok(meta) = std::fs::symlink_metadata(target_file) {
            if meta.file_type().is_symlink() {
                return Err(RuntimeError::BackupError(format!(
                    "Backup target path is a symlink: {}. Symlink targets are prohibited.",
                    target_file.display()
                )));
            }
        }

        let parent = target_file.parent().ok_or_else(|| {
            RuntimeError::BackupError(format!(
                "Target file has no parent directory: {}",
                target_file.display()
            ))
        })?;

        std::fs::create_dir_all(parent).map_err(RuntimeError::Io)?;
        let canonical_parent = dunce::canonicalize(parent).map_err(RuntimeError::Io)?;

        // 3. Create atomic temporary file in destination directory with 0600 permissions from creation
        let temp_filename = format!(
            ".tmp_dump_{}_{}.part",
            db_name,
            uuid::Uuid::new_v4().simple()
        );
        let temp_path = canonical_parent.join(temp_filename);

        #[cfg(unix)]
        let temp_file = {
            use std::os::unix::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&temp_path)
                .map_err(RuntimeError::Io)?
        };

        #[cfg(not(unix))]
        let temp_file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
            .map_err(RuntimeError::Io)?;

        let binaries = Self::discover_binaries()?;
        let dump_bin = binaries.dump.ok_or_else(|| RuntimeError::RuntimeNotFound {
            binary: "mariadb-dump/mysqldump".into(),
            version: "any".into(),
        })?;

        // 4. Stream dump output directly to file (zero process RAM buffering)
        let child = Command::new(dump_bin)
            .arg(format!("--socket={}", self.socket_path.display()))
            .arg("-u")
            .arg("root")
            .arg(db_name)
            .stdout(Stdio::from(temp_file))
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp_path);
                RuntimeError::BackupError(format!("Failed to spawn mariadb-dump: {e}"))
            })?;

        let output = child.wait_with_output().map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            RuntimeError::BackupError(format!("Failed to wait for mariadb-dump: {e}"))
        })?;

        if !output.status.success() {
            let _ = std::fs::remove_file(&temp_path);
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(RuntimeError::BackupError(format!(
                "mariadb-dump exited with error: {err}"
            )));
        }

        // 5. Verify backup file existence and basic integrity
        let file_meta = std::fs::metadata(&temp_path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            RuntimeError::BackupError(format!("Failed to inspect backup file metadata: {e}"))
        })?;

        if file_meta.len() == 0 {
            let _ = std::fs::remove_file(&temp_path);
            return Err(RuntimeError::BackupError(
                "Backup creation resulted in an empty file (0 bytes)".into(),
            ));
        }

        // Validate basic SQL header (e.g. starts with standard dump comment or SQL)
        use std::io::Read;
        let mut check_file = std::fs::File::open(&temp_path).map_err(RuntimeError::Io)?;
        let mut header_buf = [0u8; 64];
        let bytes_read = check_file.read(&mut header_buf).map_err(RuntimeError::Io)?;
        let header_str = String::from_utf8_lossy(&header_buf[..bytes_read]);
        if !header_str.contains("--")
            && !header_str.contains("/*")
            && !header_str.to_uppercase().contains("CREATE")
        {
            let _ = std::fs::remove_file(&temp_path);
            return Err(RuntimeError::BackupError(
                "Backup file does not contain valid SQL dump header".into(),
            ));
        }
        drop(check_file);

        // 6. Atomic rename from temporary file to final target file
        std::fs::rename(&temp_path, target_file).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            RuntimeError::BackupError(format!("Failed to atomically rename backup file: {e}"))
        })?;

        Ok(())
    }

    /// Restores database from a SQL dump file with safety guards.
    /// Refuses to restore into an existing non-empty database automatically.
    /// Streams dump file into client stdin to prevent memory exhaustion.
    /// Note: MySQL/MariaDB DDL statements execute with implicit commits and cannot provide all-or-nothing rollback.
    pub fn restore_database(&self, db_name: &str, source_file: &Path) -> Result<()> {
        Self::validate_identifier(db_name)?;

        // 1. Validate source file existence and regular file
        if !source_file.exists() {
            return Err(RuntimeError::RestoreError(format!(
                "Dump source file does not exist: {}",
                source_file.display()
            )));
        }

        let meta = std::fs::symlink_metadata(source_file).map_err(RuntimeError::Io)?;
        if meta.file_type().is_symlink() {
            return Err(RuntimeError::RestoreError(format!(
                "Source file is a symlink: {}. Symlink restoration is rejected.",
                source_file.display()
            )));
        }

        if meta.len() == 0 {
            return Err(RuntimeError::RestoreError(
                "Source backup file is empty (0 bytes).".into(),
            ));
        }

        // 2. Validate backup header / integrity before executing
        use std::io::Read;
        let mut check_file = std::fs::File::open(source_file).map_err(RuntimeError::Io)?;
        let mut header_buf = [0u8; 128];
        let bytes_read = check_file.read(&mut header_buf).map_err(RuntimeError::Io)?;
        let header_str = String::from_utf8_lossy(&header_buf[..bytes_read]);
        if !header_str.contains("--")
            && !header_str.contains("/*")
            && !header_str.to_uppercase().contains("CREATE")
        {
            return Err(RuntimeError::RestoreError(
                "Source file does not appear to be a valid SQL dump archive.".into(),
            ));
        }
        drop(check_file);

        // 3. Inspect target database occupancy (FAIL-CLOSED)
        let check_sql = format!(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = '{db_name}';"
        );
        let count_output = self.execute_query(&check_sql).map_err(|e| {
            RuntimeError::RestoreError(format!(
                "Failed to inspect target database occupancy before restore: {e}. Refusing to restore into unverified destination."
            ))
        })?;

        let count_line = count_output
            .lines()
            .last()
            .ok_or_else(|| {
                RuntimeError::RestoreError(
                    "Database inspection returned empty output; cannot verify destination occupancy.".into(),
                )
            })?
            .trim();

        let count_num: u64 = count_line.parse().map_err(|e| {
            RuntimeError::RestoreError(format!(
                "Failed to parse database table count '{count_line}': {e}. Cannot guarantee destination is empty."
            ))
        })?;

        if count_num > 0 {
            return Err(RuntimeError::RestoreError(format!(
                "Target database '{db_name}' already exists and contains {count_num} tables. Refusing to restore into non-empty database automatically."
            )));
        }

        // 4. Ensure target database exists
        let create_db_sql = format!(
            "CREATE DATABASE IF NOT EXISTS `{db_name}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;\n"
        );
        self.execute_query(&create_db_sql)?;

        // 5. Stream SQL dump directly from file into client stdin (zero process RAM buffering)
        let binaries = Self::discover_binaries()?;
        let client = binaries
            .client
            .ok_or_else(|| RuntimeError::RuntimeNotFound {
                binary: "mariadb/mysql client".into(),
                version: "any".into(),
            })?;

        let input_file = std::fs::File::open(source_file).map_err(RuntimeError::Io)?;

        let child = Command::new(client)
            .arg(format!("--socket={}", self.socket_path.display()))
            .arg("-u")
            .arg("root")
            .arg(db_name)
            .stdin(Stdio::from(input_file))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                RuntimeError::RestoreError(format!("Failed to spawn client for restore: {e}"))
            })?;

        let output = child
            .wait_with_output()
            .map_err(|e| RuntimeError::RestoreError(format!("Restore execution failed: {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            // Check if partial restore occurred
            let partial_count = match self.execute_query(&check_sql) {
                Ok(out) => out
                    .lines()
                    .last()
                    .and_then(|l| l.trim().parse::<u64>().ok())
                    .unwrap_or(0),
                Err(_) => 0,
            };

            if partial_count > 0 {
                return Err(RuntimeError::RestoreError(format!(
                    "Database restore failed partially (exit status {}): {err}. Warning: {partial_count} tables were created before failure. Note: MariaDB DDL operations execute with implicit commits and cannot be rolled back transactionally.",
                    output.status
                )));
            } else {
                return Err(RuntimeError::RestoreError(format!(
                    "Database restore exited with error (exit status {}): {err}",
                    output.status
                )));
            }
        }

        Ok(())
    }

    /// Validates SQL identifiers (table, database, user names) to prevent injection.
    pub fn validate_identifier(name: &str) -> Result<()> {
        if name.is_empty()
            || name.len() > 64
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(RuntimeError::DatabaseError(format!(
                "Invalid database identifier: '{name}'. Only alphanumeric characters and underscores are permitted."
            )));
        }
        Ok(())
    }
}

/// Securely escapes a string literal for MariaDB SQL statements.
/// Rejects null bytes (\0) to prevent truncation attacks.
/// Escapes single quotes as '' (standard ANSI SQL) and backslashes as \\.
pub fn escape_sql_string_literal(input: &str) -> Result<String> {
    if input.contains('\0') {
        return Err(RuntimeError::CredentialError(
            "SQL string literal contains prohibited null byte (\\0)".into(),
        ));
    }
    let mut escaped = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '\'' => escaped.push_str("''"),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\x1a' => escaped.push_str("\\Z"),
            _ => escaped.push(ch),
        }
    }
    Ok(escaped)
}

impl Drop for MariaDbSupervisor {
    fn drop(&mut self) {
        if self.terminate_on_drop {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mariadb_supervisor_paths() {
        let dir = tempdir().unwrap();
        let sup = MariaDbSupervisor::new_simple(dir.path().to_path_buf(), 3307);
        assert_eq!(sup.port(), 3307);
        assert_eq!(sup.data_dir(), dir.path().join("data"));
        assert_eq!(
            sup.socket_path(),
            dir.path().join("run").join("mariadb.sock")
        );
        assert_eq!(sup.pid_file(), dir.path().join("run").join("mariadb.pid"));
    }

    #[test]
    fn test_mariadb_config_generation() {
        let dir = tempdir().unwrap();
        let sup = MariaDbSupervisor::new_simple(dir.path().to_path_buf(), 3308);
        let conf = sup.generate_config();

        assert!(conf.contains("port = 3308"));
        assert!(conf.contains("mariadb.sock"));
        assert!(conf.contains("innodb_buffer_pool_size = 64M"));
        assert!(conf.contains("character-set-server = utf8mb4"));
    }

    #[test]
    fn test_mariadb_identifier_validation() {
        assert!(MariaDbSupervisor::validate_identifier("valid_db_123").is_ok());
        assert!(MariaDbSupervisor::validate_identifier("").is_err());
        assert!(MariaDbSupervisor::validate_identifier("invalid-db").is_err());
        assert!(MariaDbSupervisor::validate_identifier("invalid db; DROP TABLE").is_err());
        assert!(MariaDbSupervisor::validate_identifier("root").is_ok());
        assert!(MariaDbSupervisor::validate_identifier(
            "user_with_very_long_name_that_exceeds_sixty_four_characters_limit_which_is_invalid"
        )
        .is_err());
    }

    #[test]
    fn test_escape_sql_string_literal() {
        // Simple string
        assert_eq!(escape_sql_string_literal("hello").unwrap(), "hello");

        // Quotes escaping (ANSI standard '')
        assert_eq!(
            escape_sql_string_literal("p'ass'word").unwrap(),
            "p''ass''word"
        );

        // Backslash escaping
        assert_eq!(
            escape_sql_string_literal("p\\ass\\word").unwrap(),
            "p\\\\ass\\\\word"
        );

        // Combined quotes and backslashes
        assert_eq!(
            escape_sql_string_literal("p\\'\"word").unwrap(),
            "p\\\\''\"word"
        );

        // SQL injection attempt
        let inj = "secret'; DROP DATABASE mysql; --";
        let escaped = escape_sql_string_literal(inj).unwrap();
        assert_eq!(escaped, "secret''; DROP DATABASE mysql; --");

        // Unicode and emojis
        let unicode = "P@sswørd🚀2026_Şçğüö";
        assert_eq!(escape_sql_string_literal(unicode).unwrap(), unicode);

        // Prohibit null bytes
        assert!(escape_sql_string_literal("null\0byte").is_err());
    }

    #[test]
    fn test_database_deletion_approval_lifecycle() {
        let dir = tempdir().unwrap();
        let sup = MariaDbSupervisor::new_simple(dir.path().to_path_buf(), 3309);

        // 1. Direct self-authorization (using db name as token) must be rejected
        let self_auth = sup.drop_project_database("my_test_db", "my_test_db");
        assert!(self_auth.is_err());
        let err_msg = self_auth.unwrap_err().to_string();
        assert!(err_msg.contains("Self-authorization rejected"));

        // 2. Unissued token must be rejected
        let unissued = sup.drop_project_database("my_test_db", "fake_token_123");
        assert!(unissued.is_err());
        assert!(matches!(
            unissued.unwrap_err(),
            RuntimeError::DatabaseDeletionUnavailable(_)
        ));

        // 3. Request deletion handle (returns audit request_id, NOT authorization decision)
        let req = sup
            .request_database_deletion_approval("my_test_db", "unit_test_ui")
            .unwrap();
        assert_eq!(req.database_name, "my_test_db");
        assert!(req.request_id.starts_with("del_req_my_test_db_"));
        assert!(req.expires_at_secs > req.requested_at_secs);

        // 4. Requesting caller attempting to self-authorize using request_id must be strictly rejected
        let self_req_auth = sup.drop_project_database("my_test_db", &req.request_id);
        assert!(self_req_auth.is_err());
        assert!(matches!(
            self_req_auth.unwrap_err(),
            RuntimeError::DatabaseUnauthorized(_)
        ));

        // 5. Destructive database operations are disabled in Phase R1.2B pending trusted desktop human approval
        let disabled_res = sup.drop_project_database("my_test_db", "arbitrary_token");
        assert!(disabled_res.is_err());
        assert!(matches!(
            disabled_res.unwrap_err(),
            RuntimeError::DatabaseDeletionUnavailable(_)
        ));
    }

    #[test]
    fn test_application_support_directory_resolution() {
        // 1. Standard resolution should succeed in a normal environment
        let resolved = resolve_application_support_dir();
        assert!(
            resolved.is_ok(),
            "Directory resolution should succeed: {:?}",
            resolved
        );
        let path = resolved.unwrap();
        assert!(path.ends_with("Library/Application Support/LocalForge"));

        // 2. Base MariaDB dir should reside under LocalForge/mariadb
        let mariadb_base = resolve_mariadb_base_dir().unwrap();
        assert_eq!(mariadb_base, path.join("mariadb"));
    }
}
