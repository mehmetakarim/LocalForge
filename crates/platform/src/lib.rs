pub mod traits;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub use traits::{CredentialStore, VolumeManager};

#[cfg(target_os = "macos")]
pub use macos::{MacOSCredentialStore, MacOSVolumeManager};
