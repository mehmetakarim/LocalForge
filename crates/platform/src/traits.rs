use localforge_core::{Result, VolumeId, VolumeInfo};
use std::path::Path;

/// Trait abstracting OS-specific volume and storage lifecycle operations.
pub trait VolumeManager: Send + Sync {
    /// List all currently mounted volumes.
    fn list_volumes(&self) -> Result<Vec<VolumeInfo>>;

    /// Find a volume by its persistent hardware or filesystem UUID.
    fn find_by_uuid(&self, uuid: &VolumeId) -> Result<Option<VolumeInfo>>;

    /// Identify which mounted volume contains the specified file or directory path.
    fn find_volume_for_path(&self, path: &Path) -> Result<Option<VolumeInfo>>;
}

/// Trait abstracting secure credential storage (Keychain on macOS, Credential Manager on Windows).
pub trait CredentialStore: Send + Sync {
    fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>>;
    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<()>;
    fn delete_secret(&self, service: &str, account: &str) -> Result<()>;
}

/// Verified identity of an active operating system process.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub pgid: u32,
    pub std_path: std::path::PathBuf,
    pub start_time_secs: u64,
    pub start_time_usecs: u32,
}

impl ProcessIdentity {
    pub fn executable_path(&self) -> &std::path::Path {
        &self.std_path
    }
}

/// Trait abstracting OS-specific process inspection, PID reuse defense, and verified termination.
pub trait ProcessManager: Send + Sync {
    /// Inspects the process currently running with the given PID.
    fn inspect_process(&self, pid: u32) -> Result<Option<ProcessIdentity>>;

    /// Verifies that the process with `pid` matches the expected binary path and launch timestamp.
    fn verify_process_identity(
        &self,
        pid: u32,
        expected_executable: &Path,
        expected_start_secs: u64,
        expected_start_usecs: u32,
    ) -> bool;

    /// Terminates the process or its process group with bounded escalation (SIGTERM then SIGKILL).
    /// Fails closed if the identity does not match or cannot be verified.
    fn terminate_process_group_verified(
        &self,
        pid: u32,
        expected_executable: &Path,
        expected_start_secs: u64,
        expected_start_usecs: u32,
        grace_period_secs: u64,
    ) -> Result<()>;
}
