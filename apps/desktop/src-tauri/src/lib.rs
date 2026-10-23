use localforge_core::{ProjectId, ProjectStatus, VolumeInfo};
use localforge_platform::{MacOSProcessManager, MacOSVolumeManager, VolumeManager};
use localforge_project_manager::ProjectService;
use localforge_runtime::{ProjectRuntimeService, ProjectRuntimeStatus};
use localforge_storage::{MetadataStore, ProjectRecord, VolumeBarrier};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub volume_uuid: String,
    pub document_root: String,
    pub php_version: String,
    pub local_domain: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub loopback_port: Option<u16>,
}

impl From<ProjectRecord> for ProjectDto {
    fn from(r: ProjectRecord) -> Self {
        Self {
            id: r.id.0,
            name: r.name,
            path: r.path.to_string_lossy().to_string(),
            volume_uuid: r.volume_uuid.0,
            document_root: r.document_root,
            php_version: r.php_version,
            local_domain: r.local_domain,
            status: r.status.to_string(),
            created_at: r.created_at,
            updated_at: r.updated_at,
            loopback_port: r.loopback_port,
        }
    }
}

pub struct AppState {
    pub db_path: PathBuf,
    pub metadata_store: std::sync::RwLock<Result<Arc<MetadataStore>, String>>,
    pub service: std::sync::RwLock<Result<Arc<ProjectService>, String>>,
    pub runtime_service: Arc<ProjectRuntimeService>,
    pub volume_manager: Arc<dyn VolumeManager>,
    pub volume_barrier: Arc<VolumeBarrier>,
}

impl AppState {
    pub fn get_metadata_store(&self) -> Result<Arc<MetadataStore>, String> {
        let read = self.metadata_store.read().unwrap();
        match &*read {
            Ok(store) => Ok(store.clone()),
            Err(orig_err) => {
                let err_msg = orig_err.clone();
                drop(read);
                // Attempt to recover by reopening persistent SQLite database
                match MetadataStore::open(&self.db_path) {
                    Ok(recovered_store) => {
                        let store_arc = Arc::new(recovered_store);
                        let mut write = self.metadata_store.write().unwrap();
                        *write = Ok(store_arc.clone());
                        let mut svc_write = self.service.write().unwrap();
                        *svc_write = Ok(Arc::new(ProjectService::new(
                            store_arc.clone(),
                            self.volume_manager.clone(),
                            self.volume_barrier.clone(),
                        )));
                        self.runtime_service.reconnect_metadata_store(store_arc.clone());
                        Ok(store_arc)
                    }
                    Err(retry_err) => Err(format!(
                        "Kalıcı SQLite veritabanı açılamadı ({}). Yeniden deneme hatası: {}. İlk hata: {}. Mevcut kullanıcı verilerini korumak amacıyla geçici belleğe geçiş engellendi.",
                        self.db_path.display(),
                        retry_err,
                        err_msg
                    )),
                }
            }
        }
    }

    pub fn get_service(&self) -> Result<Arc<ProjectService>, String> {
        let _ = self.get_metadata_store()?;
        let read = self.service.read().unwrap();
        match &*read {
            Ok(s) => Ok(s.clone()),
            Err(e) => Err(e.clone()),
        }
    }
}

#[tauri::command]
fn get_volumes(state: State<'_, AppState>) -> Result<Vec<VolumeInfo>, String> {
    state
        .volume_manager
        .list_volumes()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_projects(state: State<'_, AppState>) -> Result<Vec<ProjectDto>, String> {
    let store = state.get_metadata_store()?;
    let projs = store
        .list_projects()
        .map_err(|e| format!("Failed to read project metadata: {e}"))?;

    Ok(projs.into_iter().map(ProjectDto::from).collect())
}

#[tauri::command]
fn sync_projects(state: State<'_, AppState>) -> Result<Vec<ProjectDto>, String> {
    let store = state.get_metadata_store()?;
    state
        .runtime_service
        .sync_all_projects_runtime_status()
        .map_err(|e| format!("Synchronization failed: {e}"))?;

    let projs = store
        .list_projects()
        .map_err(|e| format!("Failed to list projects: {e}"))?;

    Ok(projs.into_iter().map(ProjectDto::from).collect())
}

#[tauri::command]
fn create_project(
    state: State<'_, AppState>,
    name: String,
    parent_dir: String,
    docroot: String,
    php_version: String,
) -> Result<ProjectDto, String> {
    let service = state.get_service()?;
    let created = service
        .create_project(&name, Path::new(&parent_dir), &docroot, &php_version)
        .map_err(|e| e.to_string())?;

    Ok(ProjectDto::from(created))
}

#[tauri::command]
fn import_project(state: State<'_, AppState>, path: String) -> Result<ProjectDto, String> {
    let service = state.get_service()?;
    let imported = service
        .import_project(Path::new(&path))
        .map_err(|e| e.to_string())?;

    Ok(ProjectDto::from(imported))
}

#[tauri::command]
fn start_project(state: State<'_, AppState>, id: String) -> Result<ProjectRuntimeStatus, String> {
    let _ = state.get_metadata_store()?;
    state
        .runtime_service
        .start_project(&ProjectId::from_string(id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_project(state: State<'_, AppState>, id: String) -> Result<ProjectRuntimeStatus, String> {
    let _ = state.get_metadata_store()?;
    state
        .runtime_service
        .stop_project(&ProjectId::from_string(id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_project_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
) -> Result<(), String> {
    let store = state.get_metadata_store()?;
    let pid = ProjectId::from_string(id);
    match status.as_str() {
        "running" => {
            state
                .runtime_service
                .start_project(&pid)
                .map_err(|e| e.to_string())?;
        }
        "stopped" => {
            state
                .runtime_service
                .stop_project(&pid)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            let project_status = match status.as_str() {
                "disconnected" => ProjectStatus::Disconnected,
                "degraded" => ProjectStatus::Degraded,
                "failed" => ProjectStatus::Failed,
                _ => ProjectStatus::Stopped,
            };

            store
                .update_project_status(&pid, project_status)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct RuntimeOverviewDto {
    pub php_versions: Vec<String>,
    pub php_details: Vec<localforge_runtime::PhpVersionDetail>,
    pub nginx_port: u16,
    pub nginx_running: bool,
    pub nginx_port_conflict: bool,
    pub mariadb_port: u16,
    pub mariadb_socket: String,
    pub mariadb_alive: bool,
    pub mariadb_version: Option<String>,
    pub mariadb_latency_ms: Option<u64>,
    pub mariadb_datadir_initialized: bool,
    pub php_fpm_running: Option<bool>,
}

#[tauri::command]
fn get_runtime_overview(state: State<'_, AppState>) -> Result<RuntimeOverviewDto, String> {
    let php_details = localforge_runtime::PhpFpmSupervisor::discover_installed_details();
    let php_versions = localforge_runtime::PhpFpmSupervisor::discover_installed_versions();
    let nginx_port = 8080;
    let gateway_health = state
        .runtime_service
        .nginx_supervisor()
        .check_gateway_health();
    let nginx_running = matches!(
        gateway_health,
        localforge_runtime::GatewayHealth::Running { .. }
    );
    let nginx_port_conflict = matches!(
        gateway_health,
        localforge_runtime::GatewayHealth::PortConflict { .. }
    );
    let db_status = state.runtime_service.get_database_status().ok();
    let mariadb_supervisor = state.runtime_service.mariadb_supervisor();
    let mariadb_datadir_initialized = mariadb_supervisor.is_datadir_initialized();

    let store = state.get_metadata_store()?;
    let projs = store
        .list_projects()
        .map_err(|e| format!("Failed to read project metadata: {e}"))?;

    let running_projs: Vec<_> = projs
        .into_iter()
        .filter(|p| p.status == localforge_core::ProjectStatus::Running)
        .collect();

    let php_fpm_running = if running_projs.is_empty() {
        false
    } else {
        running_projs
            .iter()
            .all(|p| state.runtime_service.php_supervisor().is_pool_alive(&p.id))
    };

    Ok(RuntimeOverviewDto {
        php_versions,
        php_details,
        nginx_port,
        nginx_running,
        nginx_port_conflict,
        mariadb_port: db_status.as_ref().map(|s| s.port).unwrap_or(3306),
        mariadb_socket: db_status
            .as_ref()
            .map(|s| s.socket_path.display().to_string())
            .unwrap_or_default(),
        mariadb_alive: db_status.as_ref().map(|s| s.is_alive).unwrap_or(false),
        mariadb_version: db_status.as_ref().and_then(|s| s.version.clone()),
        mariadb_latency_ms: db_status.as_ref().and_then(|s| s.latency_ms),
        mariadb_datadir_initialized,
        php_fpm_running: Some(php_fpm_running),
    })
}

#[tauri::command]
fn update_project_php_version(
    state: State<'_, AppState>,
    id: String,
    php_version: String,
) -> Result<ProjectDto, String> {
    let pid = ProjectId::from_string(id);
    let store = state.get_metadata_store()?;
    let _existing_proj = store
        .get_project(&pid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proje bulunamadı: {}", pid.0))?;

    // Strict validation: Validate against installed PHP-FPM binaries before persisting
    state
        .runtime_service
        .php_supervisor()
        .validate_php_version(&php_version)
        .map_err(|e| format!("PHP sürümü doğrulanamadı: {e}"))?;

    // Persist only after verified validation
    store
        .update_project_php_version(&pid, &php_version)
        .map_err(|e| format!("Veritabanı güncellenemedi: {e}"))?;

    let updated_proj = store
        .get_project(&pid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proje bulunamadı: {}", pid.0))?;

    Ok(ProjectDto::from(updated_proj))
}

#[tauri::command]
fn start_service(
    state: State<'_, AppState>,
    service: String,
) -> Result<RuntimeOverviewDto, String> {
    match service.as_str() {
        "mariadb" => {
            let supervisor = state.runtime_service.mariadb_supervisor();
            if !supervisor.is_datadir_initialized() {
                return Err(
                    "MariaDB veri dizini başlatılmamış. Güvenlik politikası ve Product Owner direktifi gereği otomatik disk ilklendirme durdurulmuştur. Lütfen yetkilendirme talep ediniz."
                        .into(),
                );
            }
            state
                .runtime_service
                .ensure_database_running()
                .map_err(|e| format!("MariaDB başlatılamadı: {e}"))?;
            get_runtime_overview(state)
        }
        _ => Err(format!("Bilinmeyen veya desteklenmeyen servis: {service}")),
    }
}

#[tauri::command]
fn stop_service(state: State<'_, AppState>, service: String) -> Result<RuntimeOverviewDto, String> {
    match service.as_str() {
        "mariadb" => {
            let supervisor = state.runtime_service.mariadb_supervisor();
            supervisor
                .stop()
                .map_err(|e| format!("MariaDB durdurulamadı: {e}"))?;
            get_runtime_overview(state)
        }
        _ => Err(format!("Bilinmeyen veya desteklenmeyen servis: {service}")),
    }
}

#[tauri::command]
fn get_project_routing(
    state: State<'_, AppState>,
    id: String,
) -> Result<localforge_core::ProjectRoutingProfile, String> {
    let pid = ProjectId::from_string(id);
    let mut profile = state
        .runtime_service
        .get_project_routing_profile(&pid)
        .map_err(|e| e.to_string())?;

    // Check if preferred domain resolves on the host
    use std::net::ToSocketAddrs;
    let target = format!("{}:80", profile.preferred_domain);
    let resolves = target
        .to_socket_addrs()
        .map(|mut addrs| addrs.any(|a| a.ip().is_loopback()))
        .unwrap_or(false);

    if resolves {
        profile.resolution_status = localforge_core::DomainResolutionStatus::Resolved;
        profile.effective_url = format!("http://{}", profile.preferred_domain);
        profile.scenario = "Scenario A: Branded Local Domain Resolved".into();
        profile.guidance_tr = format!(
            "http://{} yerel adı başarıyla çözümlendi ve ağ geçidine yönlendirildi.",
            profile.preferred_domain
        );
    }

    Ok(profile)
}

#[tauri::command]
fn check_domain_resolution(domain: String) -> Result<bool, String> {
    use std::net::ToSocketAddrs;
    let target = format!("{}:80", domain.trim());
    match target.to_socket_addrs() {
        Ok(mut addrs) => Ok(addrs.any(|a| a.ip().is_loopback())),
        Err(_) => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let base = localforge_runtime::resolve_application_support_dir()
        .unwrap_or_else(|_| PathBuf::from("Library/Application Support/LocalForge"));
    let _ = std::fs::create_dir_all(&base);
    let db_path = base.join("localforge.db");
    let runtime_base = base.join("runtime");

    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    // Persistent storage initialization — Fail closed with recoverable error, NEVER silently fallback to in-memory!
    let (initial_store, initial_service, runtime_service) = match MetadataStore::open(&db_path) {
        Ok(store) => {
            let store_arc = Arc::new(store);
            let s = Arc::new(ProjectService::new(
                store_arc.clone(),
                volume_manager.clone(),
                volume_barrier.clone(),
            ));
            let rs = Arc::new(ProjectRuntimeService::create_default(
                runtime_base,
                store_arc.clone(),
                volume_manager.clone(),
                volume_barrier.clone(),
                process_manager.clone(),
                8080,
            ));
            (Ok(store_arc), Ok(s), rs)
        }
        Err(e) => {
            let err_msg = format!(
                "Kalıcı SQLite veritabanı açılamadı ({}): {}. Mevcut verileri korumak amacıyla geçici belleğe geçiş engellendi.",
                db_path.display(),
                e
            );
            // Create an isolated in-memory placeholder solely for runtime supervisor instantiation so Tauri can boot and report the error to UI
            let mem_store = Arc::new(MetadataStore::open_in_memory().unwrap());
            let rs = Arc::new(ProjectRuntimeService::create_default(
                runtime_base,
                mem_store,
                volume_manager.clone(),
                volume_barrier.clone(),
                process_manager.clone(),
                8080,
            ));
            (Err(err_msg.clone()), Err(err_msg), rs)
        }
    };

    let state = AppState {
        db_path,
        metadata_store: std::sync::RwLock::new(initial_store),
        service: std::sync::RwLock::new(initial_service),
        runtime_service,
        volume_manager,
        volume_barrier,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_volumes,
            get_projects,
            sync_projects,
            create_project,
            import_project,
            start_project,
            stop_project,
            update_project_status,
            update_project_php_version,
            start_service,
            stop_service,
            get_runtime_overview,
            check_domain_resolution,
            get_project_routing
        ])
        .run(tauri::generate_context!())
        .expect("error while running LocalForge application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use localforge_core::{ProjectId, ProjectStatus, VolumeId};
    use localforge_platform::{MacOSProcessManager, MacOSVolumeManager};
    use localforge_storage::ProjectRecord;
    use tempfile::tempdir;

    #[test]
    fn test_app_state_recovery_guard_and_reconnection() {
        let dir = tempdir().unwrap();
        let blocker = dir.path().join("blocker_file");
        std::fs::write(&blocker, b"not a dir").unwrap();
        let invalid_db_path = blocker.join("sub").join("localforge.db");

        let volume_manager = Arc::new(MacOSVolumeManager::new());
        let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
        let process_manager = Arc::new(MacOSProcessManager::new());
        let runtime_base = dir.path().join("runtime");

        // 1. Simulate startup when persistent database fails to open
        let err_msg = "Kalıcı SQLite veritabanı açılamadı (test error)".to_string();
        let mem_store = Arc::new(MetadataStore::open_in_memory().unwrap());
        let rs = Arc::new(ProjectRuntimeService::create_default(
            runtime_base.clone(),
            mem_store.clone(),
            volume_manager.clone(),
            volume_barrier.clone(),
            process_manager.clone(),
            18080,
        ));

        let mut state = AppState {
            db_path: invalid_db_path,
            metadata_store: std::sync::RwLock::new(Err(err_msg.clone())),
            service: std::sync::RwLock::new(Err(err_msg.clone())),
            runtime_service: rs.clone(),
            volume_manager: volume_manager.clone(),
            volume_barrier: volume_barrier.clone(),
        };

        // 2. Verify all operations fail closed while in error state:
        // Recovery error remains visible and no operations touch in-memory placeholder
        let store_res = state.get_metadata_store();
        assert!(store_res.is_err());
        assert!(store_res
            .unwrap_err()
            .contains("Kalıcı SQLite veritabanı açılamadı"));

        let svc_res = state.get_service();
        assert!(svc_res.is_err());

        // In-memory placeholder must have 0 projects
        assert_eq!(mem_store.list_projects().unwrap().len(), 0);

        // 3. Prepare real on-disk persistent database with an existing project
        let valid_db_path = dir.path().join("valid_localforge.db");
        let persistent_store = MetadataStore::open(&valid_db_path).unwrap();
        let existing_project = ProjectRecord {
            id: ProjectId::from_string("proj_persisted"),
            name: "Persistent Project".into(),
            path: dir.path().join("persisted_site"),
            volume_uuid: VolumeId::new("UUID-PERSIST"),
            document_root: "".into(),
            php_version: "8.4".into(),
            local_domain: "persisted.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-20T00:00:00Z".into(),
            updated_at: "2026-10-20T00:00:00Z".into(),
            loopback_port: None,
        };
        persistent_store.insert_project(&existing_project).unwrap();
        drop(persistent_store);

        // 4. Update state db_path to the valid database to simulate recovery (e.g. disk remounted / repaired)
        state.db_path = valid_db_path.clone();

        // 5. Trigger recovery via get_metadata_store()
        let recovered_store = state.get_metadata_store().expect("Recovery must succeed");
        let listed = recovered_store.list_projects().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id.0, "proj_persisted");

        // 6. Verify existing on-disk data was NEVER overwritten or lost
        assert_eq!(listed[0].name, "Persistent Project");

        // 7. Verify all services reconnected to recovered persistent store (NO stale references!)
        let active_svc = state.get_service().expect("Service must be reconnected");
        assert!(Arc::ptr_eq(&recovered_store, &active_svc.metadata_store()));

        // Runtime service must now be reconnected to recovered store
        let rs_store = state.runtime_service.metadata_store();
        assert!(Arc::ptr_eq(&recovered_store, &rs_store));

        // Temporary in-memory store remains completely empty and unpolluted
        assert_eq!(mem_store.list_projects().unwrap().len(), 0);
    }

    #[test]
    fn test_check_domain_resolution_behavior() {
        // localhost or 127.0.0.1 must resolve to loopback
        assert!(check_domain_resolution("localhost".into()).unwrap());
        assert!(check_domain_resolution("127.0.0.1".into()).unwrap());

        // An unresolvable/unconfigured domain must fail closed (Ok(false))
        assert!(
            !check_domain_resolution("definitely-unconfigured-test-domain-999.test".into())
                .unwrap()
        );
    }
}
