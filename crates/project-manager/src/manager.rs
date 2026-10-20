use crate::manifest::ProjectManifest;
use chrono::Utc;
use localforge_core::{LocalForgeError, ProjectId, ProjectStatus, Result};
use localforge_platform::VolumeManager;
use localforge_storage::{MetadataStore, ProjectRecord, VolumeBarrier};
use std::path::Path;
use std::sync::Arc;

pub struct ProjectService {
    metadata_store: Arc<MetadataStore>,
    volume_manager: Arc<dyn VolumeManager>,
    volume_barrier: Arc<VolumeBarrier>,
}

impl ProjectService {
    pub fn new(
        metadata_store: Arc<MetadataStore>,
        volume_manager: Arc<dyn VolumeManager>,
        volume_barrier: Arc<VolumeBarrier>,
    ) -> Self {
        Self {
            metadata_store,
            volume_manager,
            volume_barrier,
        }
    }

    /// Creates a brand new project on disk and registers it in the local SQLite store.
    pub fn create_project(
        &self,
        name: &str,
        parent_dir: &Path,
        document_root_rel: &str,
        php_version: &str,
    ) -> Result<ProjectRecord> {
        let project_dir = parent_dir.join(name);

        // Identify volume
        let volume_info = self
            .volume_manager
            .find_volume_for_path(parent_dir)?
            .ok_or_else(|| {
                LocalForgeError::PlatformError("Could not determine storage volume".into())
            })?;

        // Guard against writing to unmounted /Volumes path
        self.volume_barrier
            .safe_create_dir_all(&project_dir, Some(&volume_info.uuid))?;

        // Create docroot
        let docroot_path = project_dir.join(document_root_rel);
        std::fs::create_dir_all(&docroot_path).map_err(LocalForgeError::Io)?;

        // Add a sample index.php
        let index_php = docroot_path.join("index.php");
        if !index_php.exists() {
            let sample = format!(
                "<?php\necho '<h1>Welcome to {}</h1><p>Powered by LocalForge (PHP ' . phpversion() . ')</p>';\n",
                name
            );
            let _ = std::fs::write(index_php, sample);
        }

        let project_id = ProjectId::new();
        let local_domain = format!("{}.test", name.to_lowercase().replace(' ', "-"));

        // Create and write manifest
        let manifest = ProjectManifest::new(
            project_id.clone(),
            name.to_string(),
            document_root_rel.to_string(),
            php_version.to_string(),
            local_domain.clone(),
            volume_info.uuid.clone(),
        );
        manifest.save_to_dir(&project_dir)?;

        let now = Utc::now().to_rfc3339();
        let record = ProjectRecord {
            id: project_id,
            name: name.to_string(),
            path: project_dir,
            volume_uuid: volume_info.uuid,
            document_root: document_root_rel.to_string(),
            php_version: php_version.to_string(),
            local_domain,
            status: ProjectStatus::Stopped,
            created_at: now.clone(),
            updated_at: now,
        };

        self.metadata_store.insert_project(&record)?;
        Ok(record)
    }

    /// Imports an existing project from disk.
    pub fn import_project(&self, project_dir: &Path) -> Result<ProjectRecord> {
        let canonical_dir = dunce::canonicalize(project_dir).map_err(LocalForgeError::Io)?;

        let volume_info = self
            .volume_manager
            .find_volume_for_path(&canonical_dir)?
            .ok_or_else(|| {
                LocalForgeError::PlatformError("Could not determine volume for project".into())
            })?;

        // Load existing manifest or generate default
        let manifest = match ProjectManifest::load_from_dir(&canonical_dir) {
            Ok(m) => m,
            Err(_) => {
                let name = canonical_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "project".into());
                let docroot = if canonical_dir.join("public").is_dir() {
                    "public".to_string()
                } else {
                    "".to_string()
                };
                let domain = format!("{}.test", name.to_lowercase().replace(' ', "-"));
                let m = ProjectManifest::new(
                    ProjectId::new(),
                    name,
                    docroot,
                    "8.2".into(),
                    domain,
                    volume_info.uuid.clone(),
                );
                m.save_to_dir(&canonical_dir)?;
                m
            }
        };

        let now = Utc::now().to_rfc3339();
        let record = ProjectRecord {
            id: manifest.project.id,
            name: manifest.project.name,
            path: canonical_dir,
            volume_uuid: volume_info.uuid,
            document_root: manifest.project.document_root,
            php_version: manifest.runtime.php_version,
            local_domain: manifest.routing.local_domain,
            status: ProjectStatus::Stopped,
            created_at: now.clone(),
            updated_at: now,
        };

        self.metadata_store.insert_project(&record)?;
        Ok(record)
    }

    /// Verifies all registered projects against actively mounted volumes.
    /// Updates status to `Disconnected` if the volume is absent or UUID mismatched.
    pub fn sync_all_projects_status(&self) -> Result<Vec<ProjectRecord>> {
        let projects = self.metadata_store.list_projects()?;
        let mut updated = Vec::new();

        for mut proj in projects {
            let check = self
                .volume_barrier
                .verify_volume_available(&proj.path, &proj.volume_uuid);

            let new_status = match check {
                Ok(_) => {
                    if proj.status == ProjectStatus::Disconnected {
                        ProjectStatus::Stopped
                    } else {
                        proj.status
                    }
                }
                Err(LocalForgeError::VolumeDisconnected { .. })
                | Err(LocalForgeError::VolumeMismatch { .. }) => ProjectStatus::Disconnected,
                Err(_) => ProjectStatus::Degraded,
            };

            if new_status != proj.status {
                self.metadata_store
                    .update_project_status(&proj.id, new_status)?;
                proj.status = new_status;
            }
            updated.push(proj);
        }

        Ok(updated)
    }

    pub fn metadata_store(&self) -> Arc<MetadataStore> {
        self.metadata_store.clone()
    }
}
