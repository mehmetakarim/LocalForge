use localforge_core::{LocalForgeError, ProjectId, Result, VolumeId};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const MANIFEST_FILENAME: &str = "project.yaml";
pub const LOCALFORGE_DIR: &str = ".localforge";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub id: ProjectId,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub document_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeMetadata {
    pub php_version: String,
    pub web_server: String,
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingMetadata {
    pub local_domain: String,
    #[serde(default)]
    pub https: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageMetadata {
    pub volume_uuid: VolumeId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectManifest {
    pub version: u32,
    pub project: ProjectMetadata,
    pub runtime: RuntimeMetadata,
    pub routing: RoutingMetadata,
    pub storage: StorageMetadata,
}

impl ProjectManifest {
    pub fn new(
        id: ProjectId,
        name: String,
        document_root: String,
        php_version: String,
        local_domain: String,
        volume_uuid: VolumeId,
    ) -> Self {
        Self {
            version: 1,
            project: ProjectMetadata {
                id,
                name,
                project_type: "php".to_string(),
                document_root,
            },
            runtime: RuntimeMetadata {
                php_version,
                web_server: "nginx".to_string(),
                extensions: vec!["curl".into(), "pdo_mysql".into(), "mbstring".into()],
            },
            routing: RoutingMetadata {
                local_domain,
                https: false,
            },
            storage: StorageMetadata { volume_uuid },
        }
    }

    pub fn load_from_dir(project_root: &Path) -> Result<Self> {
        let manifest_path = project_root.join(LOCALFORGE_DIR).join(MANIFEST_FILENAME);
        if !manifest_path.exists() {
            return Err(LocalForgeError::ManifestNotFound {
                path: manifest_path,
            });
        }

        let content = std::fs::read_to_string(&manifest_path).map_err(LocalForgeError::Io)?;

        let manifest: ProjectManifest = serde_yaml::from_str(&content)
            .map_err(|e| LocalForgeError::ManifestParseError(e.to_string()))?;

        Ok(manifest)
    }

    pub fn save_to_dir(&self, project_root: &Path) -> Result<()> {
        let lf_dir = project_root.join(LOCALFORGE_DIR);
        std::fs::create_dir_all(&lf_dir).map_err(LocalForgeError::Io)?;

        let content = serde_yaml::to_string(self)
            .map_err(|e| LocalForgeError::Serialization(e.to_string()))?;

        let manifest_path = lf_dir.join(MANIFEST_FILENAME);
        std::fs::write(&manifest_path, content).map_err(LocalForgeError::Io)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_roundtrip() {
        let dir = tempdir().unwrap();
        let manifest = ProjectManifest::new(
            ProjectId::from_string("proj_demo"),
            "Demo Project".into(),
            "public".into(),
            "8.2".into(),
            "demo.test".into(),
            VolumeId::new("VOL-UUID-999"),
        );

        manifest.save_to_dir(dir.path()).unwrap();

        let loaded = ProjectManifest::load_from_dir(dir.path()).unwrap();
        assert_eq!(manifest, loaded);
        assert_eq!(loaded.runtime.php_version, "8.2");
        assert_eq!(loaded.routing.local_domain, "demo.test");
    }
}
