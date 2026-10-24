// Windows platform adapter stub (Planned for post-macOS stable release)
use crate::traits::{ProcessIdentity, ProcessManager};
use localforge_core::{LocalForgeError, Result};
use std::path::Path;

pub struct WindowsProcessManager;

impl WindowsProcessManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager for WindowsProcessManager {
    fn inspect_process(&self, _pid: u32) -> Result<Option<ProcessIdentity>> {
        Err(LocalForgeError::PlatformError(
            "Windows process management not yet implemented".into(),
        ))
    }

    fn verify_process_identity(
        &self,
        _pid: u32,
        _expected_executable: &Path,
        _expected_start_secs: u64,
        _expected_start_usecs: u32,
    ) -> bool {
        false
    }

    fn terminate_process_group_verified(
        &self,
        _pid: u32,
        _expected_executable: &Path,
        _expected_start_secs: u64,
        _expected_start_usecs: u32,
        _grace_period_secs: u64,
    ) -> Result<()> {
        Err(LocalForgeError::PlatformError(
            "Windows process management not yet implemented".into(),
        ))
    }

    fn signal_process_verified(
        &self,
        _pid: u32,
        _expected_executable: &Path,
        _expected_start_secs: u64,
        _expected_start_usecs: u32,
        _signal: i32,
    ) -> Result<bool> {
        Err(LocalForgeError::PlatformError(
            "Windows process management not yet implemented".into(),
        ))
    }
}
