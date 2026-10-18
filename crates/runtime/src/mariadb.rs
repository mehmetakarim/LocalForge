use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use localforge_platform::{ProcessIdentity, ProcessManager};
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
    pub fn create_default(process_manager: Arc<dyn ProcessManager>, preferred_port: u16) -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users/mehmetakar"));
        let base_dir = home
            .join("Library")
            .join("Application Support")
            .join("LocalForge")
            .join("mariadb");
        Self::new(base_dir, preferred_port, process_manager)
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

    /// Initializes data directory if not already created. Never overwrites existing tables.
    pub fn ensure_datadir_initialized(&self) -> Result<()> {
        let _ = std::fs::create_dir_all(&self.base_dir);
        let _ = std::fs::create_dir_all(&self.data_dir);
        let _ = std::fs::create_dir_all(&self.run_dir);

        // Check if database is already initialized
        if self.data_dir.join("mysql").is_dir() {
            // Ensure config exists
            if !self.config_file.exists() {
                std::fs::write(&self.config_file, self.generate_config())
                    .map_err(RuntimeError::Io)?;
            }
            return Ok(());
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

        // 4. Ensure data directory & configuration initialized
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
                let _ = child.kill();
                let _ = child.wait();
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

    /// Executes raw SQL query via client CLI through the Unix domain socket.
    pub fn execute_query(&self, sql: &str) -> Result<String> {
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
            return Err(RuntimeError::DatabaseError(format!(
                "SQL execution error: {err}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Creates an isolated database and dedicated project user.
    pub fn create_project_database(&self, db_name: &str, user: &str, pass: &str) -> Result<()> {
        Self::validate_identifier(db_name)?;
        Self::validate_identifier(user)?;

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
            pass = pass,
        );

        self.execute_query(&sql)?;
        Ok(())
    }

    /// Drops project database strictly requiring confirmation token matching database name.
    pub fn drop_project_database(&self, db_name: &str, confirmation_token: &str) -> Result<()> {
        Self::validate_identifier(db_name)?;
        if db_name != confirmation_token {
            return Err(RuntimeError::DatabaseError(format!(
                "Confirmation token '{confirmation_token}' does not match database name '{db_name}'"
            )));
        }

        let sql = format!("DROP DATABASE IF EXISTS `{db_name}`;");
        self.execute_query(&sql)?;
        Ok(())
    }

    /// Performs logical database dump to target file with 0600 permissions.
    pub fn dump_database(&self, db_name: &str, target_file: &Path) -> Result<()> {
        Self::validate_identifier(db_name)?;

        if let Some(parent) = target_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let binaries = Self::discover_binaries()?;
        let dump_bin = binaries.dump.ok_or_else(|| RuntimeError::RuntimeNotFound {
            binary: "mariadb-dump/mysqldump".into(),
            version: "any".into(),
        })?;

        let output = Command::new(dump_bin)
            .arg(format!("--socket={}", self.socket_path.display()))
            .arg("-u")
            .arg("root")
            .arg(db_name)
            .output()
            .map_err(|e| RuntimeError::DatabaseError(format!("Dump failed: {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(RuntimeError::DatabaseError(format!(
                "mariadb-dump exited with error: {err}"
            )));
        }

        std::fs::write(target_file, &output.stdout).map_err(RuntimeError::Io)?;

        // Enforce 0600 permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(target_file)
                .map_err(RuntimeError::Io)?
                .permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(target_file, perms);
        }

        Ok(())
    }

    /// Restores database from a SQL dump file.
    pub fn restore_database(&self, db_name: &str, source_file: &Path) -> Result<()> {
        Self::validate_identifier(db_name)?;
        if !source_file.exists() {
            return Err(RuntimeError::DatabaseError(format!(
                "Dump source file does not exist: {}",
                source_file.display()
            )));
        }

        let sql = std::fs::read_to_string(source_file).map_err(RuntimeError::Io)?;
        let create_db_sql =
            format!("CREATE DATABASE IF NOT EXISTS `{db_name}`; USE `{db_name}`;\n");
        let full_sql = format!("{create_db_sql}{sql}");

        self.execute_query(&full_sql)?;
        Ok(())
    }

    /// Validates SQL identifiers (table, database, user names) to prevent injection.
    fn validate_identifier(name: &str) -> Result<()> {
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
    }
}
