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
    pub service: Arc<ProjectService>,
    pub runtime_service: Arc<ProjectRuntimeService>,
    pub volume_manager: Arc<dyn VolumeManager>,
    pub metadata_store: Arc<MetadataStore>,
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
    let projs = state
        .metadata_store
        .list_projects()
        .map_err(|e| e.to_string())?;

    Ok(projs.into_iter().map(ProjectDto::from).collect())
}

#[tauri::command]
fn sync_projects(state: State<'_, AppState>) -> Result<Vec<ProjectDto>, String> {
    let _ = state.runtime_service.sync_all_projects_runtime_status();
    let projs = state
        .metadata_store
        .list_projects()
        .map_err(|e| e.to_string())?;

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
    let created = state
        .service
        .create_project(&name, Path::new(&parent_dir), &docroot, &php_version)
        .map_err(|e| e.to_string())?;

    Ok(ProjectDto::from(created))
}

#[tauri::command]
fn import_project(state: State<'_, AppState>, path: String) -> Result<ProjectDto, String> {
    let imported = state
        .service
        .import_project(Path::new(&path))
        .map_err(|e| e.to_string())?;

    Ok(ProjectDto::from(imported))
}

#[tauri::command]
fn start_project(state: State<'_, AppState>, id: String) -> Result<ProjectRuntimeStatus, String> {
    state
        .runtime_service
        .start_project(&ProjectId::from_string(id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_project(state: State<'_, AppState>, id: String) -> Result<ProjectRuntimeStatus, String> {
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

            state
                .metadata_store
                .update_project_status(&pid, project_status)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = PathBuf::from(&home).join("Library/Application Support/LocalForge");
    let _ = std::fs::create_dir_all(&base);
    let db_path = base.join("localforge.db");
    let runtime_base = base.join("runtime");

    let metadata_store = Arc::new(
        MetadataStore::open(&db_path).unwrap_or_else(|_| MetadataStore::open_in_memory().unwrap()),
    );
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    let service = Arc::new(ProjectService::new(
        metadata_store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
    ));

    let runtime_service = Arc::new(ProjectRuntimeService::create_default(
        runtime_base,
        metadata_store.clone(),
        volume_manager.clone(),
        volume_barrier,
        process_manager,
        8080,
    ));

    let state = AppState {
        service,
        runtime_service,
        volume_manager,
        metadata_store,
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
            update_project_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running LocalForge application");
}
