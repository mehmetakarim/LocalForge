use crate::error::{Result, RuntimeError};
use crate::health::probe_http_health;
use crate::nginx::NginxSupervisor;
use crate::php::PhpFpmSupervisor;
use localforge_core::{ProjectId, ProjectStatus};
use localforge_platform::{ProcessManager, VolumeManager};
use localforge_storage::{MetadataStore, VolumeBarrier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRuntimeStatus {
    pub project_id: ProjectId,
    pub name: String,
    pub status: ProjectStatus,
    pub php_version: String,
    pub php_fpm_pid: Option<u32>,
    pub socket_path: Option<PathBuf>,
    pub nginx_pid: Option<u32>,
    pub gateway_port: u16,
    pub local_domain: String,
    pub local_url: String,
    pub message: String,
}

pub struct ProjectRuntimeService {
    metadata_store: Arc<MetadataStore>,
    volume_manager: Arc<dyn VolumeManager>,
    volume_barrier: Arc<VolumeBarrier>,
    process_manager: Arc<dyn ProcessManager>,
    php_supervisor: Arc<PhpFpmSupervisor>,
    nginx_supervisor: Arc<NginxSupervisor>,
    gateway_port: u16,
}

impl ProjectRuntimeService {
    pub fn new(
        metadata_store: Arc<MetadataStore>,
        volume_manager: Arc<dyn VolumeManager>,
        volume_barrier: Arc<VolumeBarrier>,
        process_manager: Arc<dyn ProcessManager>,
        php_supervisor: Arc<PhpFpmSupervisor>,
        nginx_supervisor: Arc<NginxSupervisor>,
        gateway_port: u16,
    ) -> Self {
        Self {
            metadata_store,
            volume_manager,
            volume_barrier,
            process_manager,
            php_supervisor,
            nginx_supervisor,
            gateway_port,
        }
    }

    /// Convenience constructor initializing default paths under base runtime directory.
    pub fn create_default(
        base_dir: PathBuf,
        metadata_store: Arc<MetadataStore>,
        volume_manager: Arc<dyn VolumeManager>,
        volume_barrier: Arc<VolumeBarrier>,
        process_manager: Arc<dyn ProcessManager>,
        gateway_port: u16,
    ) -> Self {
        let run_dir = base_dir.join("run");
        let conf_dir = base_dir.join("nginx");
        let _ = std::fs::create_dir_all(&run_dir);
        let _ = std::fs::create_dir_all(&conf_dir);

        let php_supervisor = Arc::new(PhpFpmSupervisor::new(
            run_dir.clone(),
            process_manager.clone(),
        ));
        let nginx_supervisor = Arc::new(NginxSupervisor::new(
            conf_dir,
            run_dir,
            gateway_port,
            process_manager.clone(),
        ));

        Self::new(
            metadata_store,
            volume_manager,
            volume_barrier,
            process_manager,
            php_supervisor,
            nginx_supervisor,
            gateway_port,
        )
    }

    /// Starts development runtime for the given project.
    pub fn start_project(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let project = self
            .metadata_store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // 1. External volume barrier check
        if let Err(e) = self
            .volume_barrier
            .verify_volume_available(&project.path, &project.volume_uuid)
        {
            let _ = self
                .metadata_store
                .update_project_status(&project.id, ProjectStatus::Disconnected);
            return Err(RuntimeError::VolumeDisconnected(e.to_string()));
        }

        // 2. Validate requested PHP runtime version
        if let Err(e) = self
            .php_supervisor
            .validate_php_version(&project.php_version)
        {
            let _ = self
                .metadata_store
                .update_project_status(&project.id, ProjectStatus::Degraded);
            return Err(e);
        }

        // 3. Mark project STARTING
        let _ = self
            .metadata_store
            .update_project_status(&project.id, ProjectStatus::Starting);

        // 4. Start isolated PHP-FPM pool
        let socket_path =
            match self
                .php_supervisor
                .start_pool(&project.id, &project.php_version, &project.path)
            {
                Ok(sock) => sock,
                Err(e) => {
                    let _ = self
                        .metadata_store
                        .update_project_status(&project.id, ProjectStatus::Failed);
                    return Err(e);
                }
            };

        // 5. Register virtual host with shared Nginx gateway
        if let Err(e) = self
            .nginx_supervisor
            .add_or_update_vhost(&project, &socket_path)
        {
            let _ = self.php_supervisor.stop_pool(&project.id);
            let _ = self
                .metadata_store
                .update_project_status(&project.id, ProjectStatus::Failed);
            return Err(e);
        }

        // 6. Real HTTP Health Probe (allow 50ms for worker reload)
        std::thread::sleep(std::time::Duration::from_millis(50));
        let local_url = format!("http://{}:{}", project.local_domain, self.gateway_port);
        let loopback_host = format!("{}:{}", project.local_domain, self.gateway_port);

        let probe_result = probe_http_health(self.gateway_port, &loopback_host, "/");
        let message = match probe_result {
            Ok((status_code, _)) => {
                format!("Runtime active and verified. HTTP response status: {status_code}")
            }
            Err(e) => {
                // Also try plain local domain without port in Host header
                match probe_http_health(self.gateway_port, &project.local_domain, "/") {
                    Ok((status_code, _)) => {
                        format!("Runtime active and verified. HTTP response status: {status_code}")
                    }
                    Err(_) => {
                        format!("Runtime processes active; HTTP probe warning: {e}")
                    }
                }
            }
        };

        // 7. Mark project RUNNING in metadata store
        let _ = self
            .metadata_store
            .update_project_status(&project.id, ProjectStatus::Running);

        let fpm_id = self.php_supervisor.get_pool_identity(&project.id);
        let nginx_id = self.nginx_supervisor.get_gateway_identity();

        Ok(ProjectRuntimeStatus {
            project_id: project.id,
            name: project.name,
            status: ProjectStatus::Running,
            php_version: project.php_version,
            php_fpm_pid: fpm_id.map(|i| i.pid),
            socket_path: Some(socket_path),
            nginx_pid: nginx_id.map(|i| i.pid),
            gateway_port: self.gateway_port,
            local_domain: project.local_domain,
            local_url,
            message,
        })
    }

    /// Stops development runtime for the given project cleanly.
    pub fn stop_project(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let project = self
            .metadata_store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // 1. Mark project STOPPING
        let _ = self
            .metadata_store
            .update_project_status(&project.id, ProjectStatus::Stopping);

        // 2. Stop independent PHP-FPM pool (Project A stopped, Project B untouched)
        self.php_supervisor.stop_pool(&project.id)?;

        // 3. Remove virtual host from Nginx gateway
        self.nginx_supervisor.remove_vhost(&project.id)?;

        // 4. Mark project STOPPED in metadata store
        let _ = self
            .metadata_store
            .update_project_status(&project.id, ProjectStatus::Stopped);

        Ok(ProjectRuntimeStatus {
            project_id: project.id,
            name: project.name,
            status: ProjectStatus::Stopped,
            php_version: project.php_version,
            php_fpm_pid: None,
            socket_path: None,
            nginx_pid: self.nginx_supervisor.get_gateway_identity().map(|i| i.pid),
            gateway_port: self.gateway_port,
            local_domain: project.local_domain.clone(),
            local_url: format!("http://{}:{}", project.local_domain, self.gateway_port),
            message: "Project runtime stopped cleanly.".into(),
        })
    }

    /// Returns verified runtime health status for a project.
    pub fn get_runtime_status(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let project = self
            .metadata_store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // Check volume
        let volume_ok = self
            .volume_barrier
            .verify_volume_available(&project.path, &project.volume_uuid)
            .is_ok();

        if !volume_ok {
            let _ = self
                .metadata_store
                .update_project_status(&project.id, ProjectStatus::Disconnected);

            return Ok(ProjectRuntimeStatus {
                project_id: project.id,
                name: project.name,
                status: ProjectStatus::Disconnected,
                php_version: project.php_version,
                php_fpm_pid: None,
                socket_path: None,
                nginx_pid: None,
                gateway_port: self.gateway_port,
                local_domain: project.local_domain.clone(),
                local_url: format!("http://{}:{}", project.local_domain, self.gateway_port),
                message: "External storage volume unmounted or missing.".into(),
            });
        }

        let fpm_alive = self.php_supervisor.is_pool_alive(&project.id);
        let nginx_alive = self.nginx_supervisor.is_gateway_alive();

        let fpm_id = self.php_supervisor.get_pool_identity(&project.id);
        let nginx_id = self.nginx_supervisor.get_gateway_identity();

        let (verified_status, message) = if fpm_alive && nginx_alive {
            (
                ProjectStatus::Running,
                "Runtime active and healthy.".to_string(),
            )
        } else if !fpm_alive && project.status == ProjectStatus::Running {
            // Process crashed while recorded as running
            (
                ProjectStatus::Failed,
                "PHP-FPM process terminated unexpectedly.".to_string(),
            )
        } else if fpm_alive && !nginx_alive {
            (
                ProjectStatus::Degraded,
                "PHP-FPM active, but Nginx gateway is unresponsive.".to_string(),
            )
        } else {
            (ProjectStatus::Stopped, "Runtime stopped.".to_string())
        };

        // Sync verified status to SQLite if it diverged
        if verified_status != project.status {
            let _ = self
                .metadata_store
                .update_project_status(&project.id, verified_status);
        }

        Ok(ProjectRuntimeStatus {
            project_id: project.id.clone(),
            name: project.name,
            status: verified_status,
            php_version: project.php_version,
            php_fpm_pid: fpm_id.map(|i| i.pid),
            socket_path: if fpm_alive {
                Some(self.php_supervisor.socket_path(&project.id))
            } else {
                None
            },
            nginx_pid: nginx_id.map(|i| i.pid),
            gateway_port: self.gateway_port,
            local_domain: project.local_domain.clone(),
            local_url: format!("http://{}:{}", project.local_domain, self.gateway_port),
            message,
        })
    }

    /// Queries runtime status for all registered projects.
    pub fn sync_all_projects_runtime_status(&self) -> Result<Vec<ProjectRuntimeStatus>> {
        let projects = self.metadata_store.list_projects()?;
        let mut list = Vec::new();
        for p in projects {
            if let Ok(st) = self.get_runtime_status(&p.id) {
                list.push(st);
            }
        }
        Ok(list)
    }

    /// Exposes supervisor references when direct lower-level operations are needed.
    pub fn php_supervisor(&self) -> &PhpFpmSupervisor {
        &self.php_supervisor
    }

    pub fn nginx_supervisor(&self) -> &NginxSupervisor {
        &self.nginx_supervisor
    }

    pub fn volume_manager(&self) -> &dyn VolumeManager {
        &*self.volume_manager
    }

    pub fn process_manager(&self) -> &dyn ProcessManager {
        &*self.process_manager
    }
}
