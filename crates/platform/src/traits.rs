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
