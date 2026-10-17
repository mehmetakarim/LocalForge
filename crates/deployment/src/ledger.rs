use localforge_core::{LocalForgeError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Computes hex-encoded SHA-256 checksum of a file.
pub fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(LocalForgeError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let count = file.read(&mut buffer).map_err(LocalForgeError::Io)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileChecksumEntry {
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentLedger {
    pub target_id: String,
    pub last_deployed_at: String,
    pub last_commit_sha: Option<String>,
    pub files: HashMap<String, FileChecksumEntry>,
}

impl DeploymentLedger {
    pub fn new(target_id: String) -> Self {
        Self {
            target_id,
            last_deployed_at: String::new(),
            last_commit_sha: None,
            files: HashMap::new(),
        }
    }

    pub fn ledger_path(project_root: &Path, target_id: &str) -> PathBuf {
        project_root
            .join(".localforge")
            .join("deployments")
            .join(format!("{}.json", target_id))
    }

    pub fn load(project_root: &Path, target_id: &str) -> Result<Self> {
        let path = Self::ledger_path(project_root, target_id);
        if !path.exists() {
            return Ok(Self::new(target_id.to_string()));
        }

        let content = std::fs::read_to_string(&path).map_err(LocalForgeError::Io)?;
        let ledger: Self = serde_json::from_str(&content)
            .map_err(|e| LocalForgeError::Serialization(e.to_string()))?;
        Ok(ledger)
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = Self::ledger_path(project_root, &self.target_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(LocalForgeError::Io)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| LocalForgeError::Serialization(e.to_string()))?;

        std::fs::write(&path, content).map_err(LocalForgeError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_checksum_computation() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "Hello LocalForge!").unwrap();

        let hash = compute_sha256(&file).unwrap();
        assert_eq!(hash.len(), 64);

        let hash2 = compute_sha256(&file).unwrap();
        assert_eq!(hash, hash2);
    }
}
