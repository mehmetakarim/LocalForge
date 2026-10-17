use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

pub struct MariaDbSupervisor {
    base_dir: PathBuf,
    data_dir: PathBuf,
    port: u16,
    process: Mutex<Option<Child>>,
}

impl MariaDbSupervisor {
    pub fn new(base_dir: PathBuf, preferred_port: u16) -> Self {
        let data_dir = base_dir.join("data");
        Self {
            base_dir,
            data_dir,
            port: preferred_port,
            process: Mutex::new(None),
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Finds mariadbd or mysqld binary on macOS.
    pub fn find_binary() -> Result<PathBuf> {
        let candidates = vec![
            "/opt/homebrew/bin/mariadbd",
            "/opt/homebrew/bin/mysqld",
            "/usr/local/bin/mariadbd",
            "/usr/local/bin/mysqld",
        ];

        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Ok(p);
            }
        }

        if let Ok(output) = Command::new("which").arg("mariadbd").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Ok(PathBuf::from(s));
                }
            }
        }

        if let Ok(output) = Command::new("which").arg("mysqld").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Ok(PathBuf::from(s));
                }
            }
        }

        Err(RuntimeError::RuntimeNotFound {
            binary: "mariadbd/mysqld".into(),
            version: "any".into(),
        })
    }

    /// Initializes data directory if not already created.
    pub fn ensure_datadir_initialized(&self) -> Result<()> {
        if self.data_dir.join("mysql").is_dir() {
            return Ok(());
        }

        let _ = std::fs::create_dir_all(&self.data_dir);

        // Try mariadb-install-db
        let mut install_cmd = Command::new("mariadb-install-db");
        install_cmd.arg(format!("--datadir={}", self.data_dir.display()));
        install_cmd.arg(format!("--basedir={}", self.base_dir.display()));

        let res = install_cmd.output();
        if let Ok(output) = res {
            if output.status.success() {
                return Ok(());
            }
        }

        // Fallback: mysql_install_db
        let mut fallback = Command::new("mysql_install_db");
        fallback.arg(format!("--datadir={}", self.data_dir.display()));
        let _ = fallback.output();

        Ok(())
    }

    /// Starts MariaDB daemon under current unprivileged user.
    pub fn start(&mut self) -> Result<()> {
        let mut lock = self.process.lock().unwrap();
        if let Some(child) = lock.as_mut() {
            if child.try_wait().unwrap_or(None).is_none() {
                return Ok(()); // Already running
            }
        }

        // Port check
        if !is_port_available(self.port) {
            // Find another port if default is taken
            if let Some(free) = crate::health::find_available_port(self.port + 1, 100) {
                self.port = free;
            } else {
                return Err(RuntimeError::PortConflict {
                    port: self.port,
                    details: "No available port found for MariaDB".into(),
                });
            }
        }

        let bin = Self::find_binary()?;
        self.ensure_datadir_initialized()?;

        let child = Command::new(bin)
            .arg(format!("--datadir={}", self.data_dir.display()))
            .arg(format!("--port={}", self.port))
            .arg("--bind-address=127.0.0.1")
            .arg(format!(
                "--pid-file={}/mariadb.pid",
                self.base_dir.display()
            ))
            .arg(format!("--socket={}/mariadb.sock", self.base_dir.display()))
            .spawn()
            .map_err(|e| RuntimeError::ProcessSpawnFailed {
                process: "mariadb".into(),
                reason: e.to_string(),
            })?;

        *lock = Some(child);
        Ok(())
    }

    /// Stops MariaDB process cleanly.
    pub fn stop(&self) -> Result<()> {
        let mut lock = self.process.lock().unwrap();
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for MariaDbSupervisor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mariadb_supervisor_paths() {
        let dir = tempdir().unwrap();
        let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), 3307);
        assert_eq!(sup.port(), 3307);
        assert_eq!(sup.data_dir, dir.path().join("data"));
    }
}
