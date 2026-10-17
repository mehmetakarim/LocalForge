use crate::traits::{CredentialStore, VolumeManager};
use localforge_core::{LocalForgeError, Result, VolumeId, VolumeInfo};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MacOSVolumeManager;

impl MacOSVolumeManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOSVolumeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VolumeManager for MacOSVolumeManager {
    fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        // Parse /Volumes and root /
        let mut volumes = Vec::new();

        // 1. Root volume
        if let Ok(Some(root_vol)) = self.find_volume_for_path(Path::new("/")) {
            volumes.push(root_vol);
        }

        // 2. /Volumes/* entries
        let volumes_dir = Path::new("/Volumes");
        if let Ok(entries) = std::fs::read_dir(volumes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(Some(vol)) = self.inspect_volume(&path) {
                        // Avoid duplicates if root is somehow symlinked
                        if !volumes.iter().any(|v| v.uuid == vol.uuid) {
                            volumes.push(vol);
                        }
                    }
                }
            }
        }

        Ok(volumes)
    }

    fn find_by_uuid(&self, uuid: &VolumeId) -> Result<Option<VolumeInfo>> {
        let all = self.list_volumes()?;
        Ok(all.into_iter().find(|v| &v.uuid == uuid))
    }

    fn find_volume_for_path(&self, path: &Path) -> Result<Option<VolumeInfo>> {
        let canonical = match dunce::canonicalize(path) {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        // Determine if path is under /Volumes/...
        if canonical.starts_with("/Volumes") {
            let mut components = canonical.components();
            // skip root and 'Volumes'
            components.next(); // RootDir
            components.next(); // Normal("Volumes")
            if let Some(vol_name) = components.next() {
                let vol_path = PathBuf::from("/Volumes").join(vol_name.as_os_str());
                return self.inspect_volume(&vol_path);
            }
        }

        // Otherwise it is on root /
        self.inspect_volume(Path::new("/"))
    }
}

impl MacOSVolumeManager {
    fn inspect_volume(&self, mount_point: &Path) -> Result<Option<VolumeInfo>> {
        let output = Command::new("diskutil")
            .arg("info")
            .arg(mount_point)
            .output()
            .map_err(|e| {
                LocalForgeError::PlatformError(format!("Failed to execute diskutil: {e}"))
            })?;

        if !output.status.success() {
            return Ok(None);
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut uuid = None;
        let mut name = mount_point
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        let mut filesystem = "apfs".to_string();
        let mut is_removable = false;
        let mut is_internal = true;
        let mut total_bytes = 0u64;
        let mut available_bytes = 0u64;

        for line in text.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim();
                let val = parts[1..].join(":").trim().to_string();

                match key {
                    "Volume UUID" | "Disk / Partition UUID" => {
                        if uuid.is_none() {
                            uuid = Some(val);
                        }
                    }
                    "Volume Name" => {
                        if !val.is_empty() && val != "Not applicable (no filesystem)" {
                            name = val;
                        }
                    }
                    "Type (Bundle)" | "File System Personality" => {
                        filesystem = val.to_lowercase();
                    }
                    "Removable Media" => {
                        is_removable = val.to_lowercase().contains("yes")
                            || val.to_lowercase().contains("removable");
                    }
                    "Internal" => {
                        is_internal = val.to_lowercase().contains("yes");
                    }
                    "Volume Total Space" | "Container Total Space" => {
                        if let Some(bytes) = parse_diskutil_bytes(&val) {
                            total_bytes = bytes;
                        }
                    }
                    "Volume Free Space" | "Container Free Space" => {
                        if let Some(bytes) = parse_diskutil_bytes(&val) {
                            available_bytes = bytes;
                        }
                    }
                    _ => {}
                }
            }
        }

        let volume_uuid = match uuid {
            Some(u) => VolumeId::new(u),
            None => {
                // Fallback for special volumes: use mount point hash
                VolumeId::new(format!("vol_{}", mount_point.display()))
            }
        };

        Ok(Some(VolumeInfo {
            uuid: volume_uuid,
            name,
            mount_point: mount_point.to_path_buf(),
            filesystem,
            is_removable,
            is_internal,
            total_bytes,
            available_bytes,
        }))
    }
}

fn parse_diskutil_bytes(val: &str) -> Option<u64> {
    // Format is like: "1000.2 GB (1000204886016 Bytes) (exactly ...)"
    if let Some(start) = val.find('(') {
        if let Some(end) = val[start + 1..].find(" Bytes") {
            let bytes_str = &val[start + 1..start + 1 + end].trim();
            return bytes_str.parse::<u64>().ok();
        }
    }
    None
}

/// macOS Keychain implementation of CredentialStore.
pub struct MacOSCredentialStore;

impl MacOSCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOSCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for MacOSCredentialStore {
    fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>> {
        let output = Command::new("security")
            .arg("find-generic-password")
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .arg("-w")
            .output()
            .map_err(|e| LocalForgeError::PlatformError(format!("Keychain query failed: {e}")))?;

        if output.status.success() {
            let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(secret))
        } else {
            Ok(None)
        }
    }

    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<()> {
        let status = Command::new("security")
            .arg("add-generic-password")
            .arg("-U") // update if exists
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .arg("-w")
            .arg(secret)
            .status()
            .map_err(|e| LocalForgeError::PlatformError(format!("Keychain write failed: {e}")))?;

        if status.success() {
            Ok(())
        } else {
            Err(LocalForgeError::PlatformError(
                "Failed to save credential in Keychain".into(),
            ))
        }
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<()> {
        let _ = Command::new("security")
            .arg("delete-generic-password")
            .arg("-s")
            .arg(service)
            .arg("-a")
            .arg(account)
            .status();
        Ok(())
    }
}
