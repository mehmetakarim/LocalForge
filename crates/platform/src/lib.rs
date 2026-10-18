pub mod traits;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub use traits::{CredentialStore, ProcessIdentity, ProcessManager, VolumeManager};

#[cfg(target_os = "macos")]
pub use macos::{MacOSCredentialStore, MacOSProcessManager, MacOSVolumeManager};

#[cfg(target_os = "windows")]
pub use windows::WindowsProcessManager;
