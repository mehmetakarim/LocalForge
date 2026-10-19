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
    pub nginx_port: u16,
    pub nginx_running: bool,
    pub nginx_port_conflict: bool,
    pub mariadb_port: u16,
    pub mariadb_socket: String,
    pub mariadb_alive: bool,
    pub mariadb_version: Option<String>,
    pub mariadb_latency_ms: Option<u64>,
    pub php_fpm_running: Option<bool>,
}

#[tauri::command]
fn get_runtime_overview(state: State<'_, AppState>) -> Result<RuntimeOverviewDto, String> {
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
        php_fpm_running: Some(php_fpm_running),
    })
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
            get_runtime_overview
        ])
        .run(tauri::generate_context!())
        .expect("error while running LocalForge application");
}
