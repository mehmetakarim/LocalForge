use localforge_core::{LocalForgeError, ProjectId, ProjectStatus, Result, VolumeId};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ProjectRecord {
    pub id: ProjectId,
    pub name: String,
    pub path: PathBuf,
    pub volume_uuid: VolumeId,
    pub document_root: String,
    pub php_version: String,
    pub local_domain: String,
    pub status: ProjectStatus,
    pub created_at: String,
    pub updated_at: String,
}

pub struct MetadataStore {
    conn: Mutex<Connection>,
}

impl std::fmt::Debug for MetadataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataStore").finish()
    }
}

impl MetadataStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LocalForgeError::DatabaseError(format!("Failed to create database directory: {e}"))
            })?;
        }

        let conn = Connection::open(db_path).map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to open SQLite db: {e}"))
        })?;

        // Enforce WAL mode and foreign keys
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Failed to set pragmas: {e}")))?;

        let store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to open in-memory db: {e}"))
        })?;

        let store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                volume_uuid TEXT NOT NULL,
                document_root TEXT NOT NULL,
                php_version TEXT NOT NULL,
                local_domain TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Migration failed: {e}")))?;

        Ok(())
    }

    pub fn insert_project(&self, project: &ProjectRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                project.id.0,
                project.name,
                project.path.to_string_lossy().to_string(),
                project.volume_uuid.0,
                project.document_root,
                project.php_version,
                project.local_domain,
                project.status.to_string(),
                project.created_at,
                project.updated_at,
            ],
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Failed to insert project: {e}")))?;

        Ok(())
    }

    pub fn get_project(&self, id: &ProjectId) -> Result<Option<ProjectRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at FROM projects WHERE id = ?1")
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let mut rows = stmt
            .query(params![id.0])
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?
        {
            let status_str: String = row
                .get(7)
                .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;
            let status = match status_str.as_str() {
                "running" => ProjectStatus::Running,
                "starting" => ProjectStatus::Starting,
                "stopping" => ProjectStatus::Stopping,
                "degraded" => ProjectStatus::Degraded,
                "failed" => ProjectStatus::Failed,
                "disconnected" => ProjectStatus::Disconnected,
                "unknown" => ProjectStatus::Unknown,
                _ => ProjectStatus::Stopped,
            };

            let path_str: String = row
                .get(2)
                .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

            Ok(Some(ProjectRecord {
                id: ProjectId::from_string(row.get::<_, String>(0).unwrap()),
                name: row.get(1).unwrap(),
                path: PathBuf::from(path_str),
                volume_uuid: VolumeId::new(row.get::<_, String>(3).unwrap()),
                document_root: row.get(4).unwrap(),
                php_version: row.get(5).unwrap(),
                local_domain: row.get(6).unwrap(),
                status,
                created_at: row.get(8).unwrap(),
                updated_at: row.get(9).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at FROM projects ORDER BY name ASC")
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let status_str: String = row.get(7)?;
                let status = match status_str.as_str() {
                    "running" => ProjectStatus::Running,
                    "starting" => ProjectStatus::Starting,
                    "stopping" => ProjectStatus::Stopping,
                    "degraded" => ProjectStatus::Degraded,
                    "failed" => ProjectStatus::Failed,
                    "disconnected" => ProjectStatus::Disconnected,
                    "unknown" => ProjectStatus::Unknown,
                    _ => ProjectStatus::Stopped,
                };
                let path_str: String = row.get(2)?;

                Ok(ProjectRecord {
                    id: ProjectId::from_string(row.get::<_, String>(0)?),
                    name: row.get(1)?,
                    path: PathBuf::from(path_str),
                    volume_uuid: VolumeId::new(row.get::<_, String>(3)?),
                    document_root: row.get(4)?,
                    php_version: row.get(5)?,
                    local_domain: row.get(6)?,
                    status,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?);
        }

        Ok(list)
    }

    pub fn update_project_status(&self, id: &ProjectId, status: ProjectStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE projects SET status = ?1 WHERE id = ?2",
            params![status.to_string(), id.0],
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Failed to update status: {e}")))?;

        Ok(())
    }

    pub fn delete_project(&self, id: &ProjectId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id.0])
            .map_err(|e| {
                LocalForgeError::DatabaseError(format!("Failed to delete project: {e}"))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_crud() {
        let store = MetadataStore::open_in_memory().unwrap();
        let proj = ProjectRecord {
            id: ProjectId::from_string("proj_test1"),
            name: "Test Project".into(),
            path: PathBuf::from("/Volumes/Mac Harici Disk/Projects/test"),
            volume_uuid: VolumeId::new("VOL-1234"),
            document_root: "public".into(),
            php_version: "8.2".into(),
            local_domain: "test.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-18T00:00:00Z".into(),
            updated_at: "2026-10-18T00:00:00Z".into(),
        };

        store.insert_project(&proj).unwrap();

        let retrieved = store.get_project(&proj.id).unwrap().expect("Project found");
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(retrieved.status, ProjectStatus::Stopped);

        store
            .update_project_status(&proj.id, ProjectStatus::Running)
            .unwrap();
        let updated = store.get_project(&proj.id).unwrap().unwrap();
        assert_eq!(updated.status, ProjectStatus::Running);

        let list = store.list_projects().unwrap();
        assert_eq!(list.len(), 1);

        store.delete_project(&proj.id).unwrap();
        let after_delete = store.get_project(&proj.id).unwrap();
        assert!(after_delete.is_none());
    }
}
