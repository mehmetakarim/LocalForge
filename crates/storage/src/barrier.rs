use localforge_core::{LocalForgeError, Result, VolumeId};
use localforge_platform::VolumeManager;
use std::path::Path;
use std::sync::Arc;

/// Safeguard preventing accidental folder recreation on internal disk when an external volume is disconnected.
pub struct VolumeBarrier {
    volume_manager: Arc<dyn VolumeManager>,
}

impl VolumeBarrier {
    pub fn new(volume_manager: Arc<dyn VolumeManager>) -> Self {
        Self { volume_manager }
    }

    /// Validates that the external volume holding `path` is currently mounted and matches `expected_uuid`.
    pub fn verify_volume_available(&self, path: &Path, expected_uuid: &VolumeId) -> Result<()> {
        let mounted_volume = self.volume_manager.find_volume_for_path(path)?;

        match mounted_volume {
            Some(vol) => {
                if &vol.uuid != expected_uuid {
                    return Err(LocalForgeError::VolumeMismatch {
                        expected: expected_uuid.0.clone(),
                        actual: vol.uuid.0,
                        mount_path: vol.mount_point,
                    });
                }
                Ok(())
            }
            None => Err(LocalForgeError::VolumeDisconnected {
                volume_uuid: expected_uuid.0.clone(),
                mount_path: path.to_path_buf(),
            }),
        }
    }

    /// Safe wrapper for directory creation that strictly refuses to create folders
    /// under `/Volumes/` if the volume is not actively mounted.
    pub fn safe_create_dir_all(&self, path: &Path, expected_uuid: Option<&VolumeId>) -> Result<()> {
        // Check if path attempts to target /Volumes/...
        if path.starts_with("/Volumes") {
            let mut components = path.components();
            components.next(); // RootDir
            components.next(); // "Volumes"
            let vol_name = components.next();

            if let Some(name) = vol_name {
                let vol_root = Path::new("/Volumes").join(name.as_os_str());
                // Verify volume is actually in volume list
                let volumes = self.volume_manager.list_volumes()?;
                let is_mounted = volumes.iter().any(|v| v.mount_point == vol_root);

                if !is_mounted {
                    return Err(LocalForgeError::UnmountedExternalPathProtection {
                        path: path.to_path_buf(),
                    });
                }

                if let Some(expected) = expected_uuid {
                    self.verify_volume_available(path, expected)?;
                }
            }
        }

        std::fs::create_dir_all(path).map_err(LocalForgeError::Io)?;
        Ok(())
    }
}
