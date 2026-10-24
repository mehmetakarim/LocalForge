use localforge_core::{LocalForgeError, ProjectId, ProjectStatus, Result, VolumeId};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub loopback_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub previous_version: u32,
    pub target_version: u32,
    pub applied: bool,
    pub details: String,
}

pub struct MetadataStore {
    conn: Mutex<Connection>,
}

impl std::fmt::Debug for MetadataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataStore").finish()
    }
}

fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = match conn.prepare(&format!("PRAGMA table_info({table})")) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
        return false;
    };
    for col in rows.flatten() {
        if col.eq_ignore_ascii_case(column) {
            return true;
        }
    }
    false
}

fn has_unique_index(conn: &Connection, table: &str, index_name: &str) -> bool {
    let mut stmt = match conn.prepare(&format!("PRAGMA index_list({table})")) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let Ok(rows) = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let unique: i64 = row.get(2)?;
        Ok((name, unique == 1))
    }) else {
        return false;
    };
    for item in rows.flatten() {
        if item.0.eq_ignore_ascii_case(index_name) && item.1 {
            return true;
        }
    }
    false
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
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Migration init failed: {e}")))?;

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

        let _ = conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, description, applied_at)
             VALUES (1, 'Baseline projects table', datetime('now'));",
            [],
        );

        Ok(())
    }

    /// Returns current recorded schema version.
    pub fn get_schema_version(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let version: u32 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);

        Ok(version)
    }

    /// Applies Version 2 migration adding nullable `loopback_port` with unique partial index.
    /// Transactional, idempotent, rejects destructive changes, preserves existing project records.
    pub fn migrate_to_v2_loopback_ports(&self) -> Result<MigrationReport> {
        let mut conn = self.conn.lock().unwrap();
        let current_version = self_get_schema_version_locked(&conn)?;

        let col_exists = has_column(&conn, "projects", "loopback_port");
        let idx_exists = has_unique_index(&conn, "projects", "idx_projects_loopback_port");

        if col_exists && idx_exists {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO schema_migrations (version, description, applied_at)
                 VALUES (2, 'Add loopback_port with unique index', datetime('now'));",
                [],
            );
            return Ok(MigrationReport {
                previous_version: current_version,
                target_version: 2,
                applied: false,
                details: "Column loopback_port and unique index already exist (idempotent no-op)."
                    .into(),
            });
        }

        // If column exists but index doesn't (or partial migration), verify data integrity before indexing
        if col_exists && !idx_exists {
            let mut check_stmt = conn.prepare(
                "SELECT loopback_port, COUNT(*) FROM projects WHERE loopback_port IS NOT NULL GROUP BY loopback_port HAVING COUNT(*) > 1"
            ).map_err(|e| LocalForgeError::DatabaseError(format!("Failed to check for duplicate ports: {e}")))?;
            let duplicate_exists = check_stmt.exists([]).unwrap_or(false);
            if duplicate_exists {
                return Err(LocalForgeError::DatabaseError(
                    "Cannot apply unique index idx_projects_loopback_port: duplicate loopback_port values exist in projects table (fail-closed schema conflict).".into()
                ));
            }
        }

        let tx = conn.transaction().map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to begin migration transaction: {e}"))
        })?;

        if !col_exists {
            tx.execute("ALTER TABLE projects ADD COLUMN loopback_port INTEGER;", [])
                .map_err(|e| {
                    LocalForgeError::DatabaseError(format!(
                        "Failed to add loopback_port column: {e}"
                    ))
                })?;
        }

        if !idx_exists {
            tx.execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_loopback_port
                 ON projects(loopback_port)
                 WHERE loopback_port IS NOT NULL;",
                [],
            )
            .map_err(|e| {
                LocalForgeError::DatabaseError(format!(
                    "Failed to create unique index on loopback_port: {e}"
                ))
            })?;
        }

        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, description, applied_at)
             VALUES (2, 'Add loopback_port with unique index', datetime('now'));",
            [],
        )
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to record migration version 2: {e}"))
        })?;

        tx.commit().map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to commit migration transaction: {e}"))
        })?;

        Ok(MigrationReport {
            previous_version: current_version,
            target_version: 2,
            applied: true,
            details:
                "Schema successfully upgraded to version 2 (loopback_port and unique index added)."
                    .into(),
        })
    }

    /// Rollback V2 migration for testing recovery and downgrade procedures.
    pub fn rollback_v2_migration_for_test(&self) -> Result<MigrationReport> {
        let mut conn = self.conn.lock().unwrap();
        let current_version = self_get_schema_version_locked(&conn)?;

        if !has_column(&conn, "projects", "loopback_port") {
            return Ok(MigrationReport {
                previous_version: current_version,
                target_version: 1,
                applied: false,
                details: "Table does not have loopback_port column; already V1.".into(),
            });
        }

        let tx = conn
            .transaction()
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        // In SQLite, dropping a column or recreating table without the column
        tx.execute_batch(
            "CREATE TABLE projects_v1_backup (
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
            );
            INSERT INTO projects_v1_backup
            SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at
            FROM projects;
            DROP TABLE projects;
            ALTER TABLE projects_v1_backup RENAME TO projects;
            DELETE FROM schema_migrations WHERE version >= 2;",
        )
        .map_err(|e| LocalForgeError::DatabaseError(format!("Failed to rollback V2 migration: {e}")))?;

        tx.commit()
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        Ok(MigrationReport {
            previous_version: current_version,
            target_version: 1,
            applied: true,
            details: "Rolled back to schema version 1 (loopback_port removed, existing records preserved).".into(),
        })
    }

    pub fn insert_project(&self, project: &ProjectRecord) -> Result<()> {
        if let Some(port) = project.loopback_port {
            if !(18081..=18999).contains(&port) {
                return Err(LocalForgeError::Validation(format!(
                    "Port {port} is outside permitted LocalForge loopback range (18081..=18999)."
                )));
            }
        }

        let conn = self.conn.lock().unwrap();
        if has_column(&conn, "projects", "loopback_port") {
            conn.execute(
                "INSERT INTO projects (id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, loopback_port)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    project.loopback_port.map(|p| p as i64),
                ],
            )
            .map_err(|e| LocalForgeError::DatabaseError(format!("Failed to insert project: {e}")))?;
        } else {
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
        }

        Ok(())
    }

    pub fn get_project(&self, id: &ProjectId) -> Result<Option<ProjectRecord>> {
        let conn = self.conn.lock().unwrap();
        let has_port = has_column(&conn, "projects", "loopback_port");

        let sql = if has_port {
            "SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, loopback_port FROM projects WHERE id = ?1"
        } else {
            "SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, NULL as loopback_port FROM projects WHERE id = ?1"
        };

        let mut stmt = conn
            .prepare(sql)
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

            let loopback_port: Option<u16> = row
                .get::<_, Option<i64>>(10)
                .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?
                .and_then(|p| u16::try_from(p).ok());

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
                loopback_port,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let conn = self.conn.lock().unwrap();
        let has_port = has_column(&conn, "projects", "loopback_port");

        let sql = if has_port {
            "SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, loopback_port FROM projects ORDER BY name ASC"
        } else {
            "SELECT id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, NULL as loopback_port FROM projects ORDER BY name ASC"
        };

        let mut stmt = conn
            .prepare(sql)
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
                let loopback_port: Option<u16> = row
                    .get::<_, Option<i64>>(10)?
                    .and_then(|p| u16::try_from(p).ok());

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
                    loopback_port,
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

    pub fn update_project_php_version(&self, id: &ProjectId, php_version: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE projects SET php_version = ?1, updated_at = ?2 WHERE id = ?3",
            params![php_version, now, id.0],
        )
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!("Failed to update php_version: {e}"))
        })?;

        Ok(())
    }

    /// Persists or clears a project's dedicated loopback port.
    /// Strictly validates that port falls within permitted unprivileged range (18081..=18999),
    /// and enforces database uniqueness.
    pub fn update_project_loopback_port(&self, id: &ProjectId, port: Option<u16>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if !has_column(&conn, "projects", "loopback_port") {
            return Err(LocalForgeError::DatabaseError(
                "Database schema does not contain loopback_port column. Run migrate_to_v2_loopback_ports first.".into(),
            ));
        }

        if let Some(p) = port {
            if !(18081..=18999).contains(&p) {
                return Err(LocalForgeError::Validation(format!(
                    "Port {p} is outside permitted LocalForge loopback range (18081..=18999)."
                )));
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE projects SET loopback_port = ?1, updated_at = ?2 WHERE id = ?3",
            params![port.map(|p| p as i64), now, id.0],
        )
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") || err_str.contains("idx_projects_loopback_port") {
                LocalForgeError::DatabaseError(format!(
                    "Port conflict: port {:?} is already assigned to another project in the database.",
                    port
                ))
            } else {
                LocalForgeError::DatabaseError(format!("Failed to update loopback_port: {e}"))
            }
        })?;

        Ok(())
    }

    /// Finds the highest currently allocated port within a specific range in SQLite.
    pub fn find_highest_allocated_port(
        &self,
        range_start: u16,
        range_end: u16,
    ) -> Result<Option<u16>> {
        let conn = self.conn.lock().unwrap();
        if !has_column(&conn, "projects", "loopback_port") {
            return Ok(None);
        }

        let mut stmt = conn
            .prepare("SELECT MAX(loopback_port) FROM projects WHERE loopback_port >= ?1 AND loopback_port <= ?2")
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let max_port: Option<i64> = stmt
            .query_row(params![range_start as i64, range_end as i64], |row| {
                row.get(0)
            })
            .unwrap_or(None);

        Ok(max_port.and_then(|p| u16::try_from(p).ok()))
    }

    /// Checks if a port is recorded as allocated in the database.
    pub fn is_port_allocated_in_db(&self, port: u16) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        if !has_column(&conn, "projects", "loopback_port") {
            return Ok(false);
        }

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM projects WHERE loopback_port = ?1")
            .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let count: i64 = stmt
            .query_row(params![port as i64], |row| row.get(0))
            .unwrap_or(0);

        Ok(count > 0)
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

fn self_get_schema_version_locked(conn: &Connection) -> Result<u32> {
    let mut stmt = conn
        .prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
        .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

    let version: u32 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);

    Ok(version)
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
            loopback_port: None,
        };

        store.insert_project(&proj).unwrap();

        let retrieved = store.get_project(&proj.id).unwrap().expect("Project found");
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(retrieved.status, ProjectStatus::Stopped);
        assert_eq!(retrieved.loopback_port, None);

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

    #[test]
    fn test_v2_migration_and_port_constraints() {
        let store = MetadataStore::open_in_memory().unwrap();
        assert_eq!(store.get_schema_version().unwrap(), 1);

        // Insert project in V1 schema
        let proj1 = ProjectRecord {
            id: ProjectId::from_string("proj_1"),
            name: "Project One".into(),
            path: PathBuf::from("/Volumes/Drive/p1"),
            volume_uuid: VolumeId::new("V1"),
            document_root: "".into(),
            php_version: "8.4".into(),
            local_domain: "one.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-22T00:00:00Z".into(),
            updated_at: "2026-10-22T00:00:00Z".into(),
            loopback_port: None,
        };
        store.insert_project(&proj1).unwrap();

        // Migrate to V2
        let report = store.migrate_to_v2_loopback_ports().unwrap();
        assert!(report.applied);
        assert_eq!(store.get_schema_version().unwrap(), 2);

        // Verify existing project record preserved
        let fetched1 = store.get_project(&proj1.id).unwrap().unwrap();
        assert_eq!(fetched1.name, "Project One");
        assert_eq!(fetched1.loopback_port, None);

        // Idempotency check: running migration again does not fail
        let report2 = store.migrate_to_v2_loopback_ports().unwrap();
        assert!(!report2.applied);
        assert_eq!(store.get_schema_version().unwrap(), 2);

        // Assign valid port
        store
            .update_project_loopback_port(&proj1.id, Some(18081))
            .unwrap();
        let fetched_with_port = store.get_project(&proj1.id).unwrap().unwrap();
        assert_eq!(fetched_with_port.loopback_port, Some(18081));
        assert!(store.is_port_allocated_in_db(18081).unwrap());

        // Rejection of ports outside range (18081..=18999)
        assert!(store
            .update_project_loopback_port(&proj1.id, Some(80))
            .is_err());
        assert!(store
            .update_project_loopback_port(&proj1.id, Some(8080))
            .is_err());
        assert!(store
            .update_project_loopback_port(&proj1.id, Some(18080))
            .is_err());
        assert!(store
            .update_project_loopback_port(&proj1.id, Some(19000))
            .is_err());

        // Conflict check: second project cannot take port 18081
        let proj2 = ProjectRecord {
            id: ProjectId::from_string("proj_2"),
            name: "Project Two".into(),
            path: PathBuf::from("/Volumes/Drive/p2"),
            volume_uuid: VolumeId::new("V2"),
            document_root: "".into(),
            php_version: "8.5".into(),
            local_domain: "two.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-22T00:00:00Z".into(),
            updated_at: "2026-10-22T00:00:00Z".into(),
            loopback_port: None,
        };
        store.insert_project(&proj2).unwrap();

        let conflict_res = store.update_project_loopback_port(&proj2.id, Some(18081));
        assert!(conflict_res.is_err());

        // Assign non-conflicting port
        store
            .update_project_loopback_port(&proj2.id, Some(18082))
            .unwrap();
        assert_eq!(
            store.find_highest_allocated_port(18080, 18999).unwrap(),
            Some(18082)
        );

        // Test Rollback to V1
        let rb_report = store.rollback_v2_migration_for_test().unwrap();
        assert!(rb_report.applied);
        assert_eq!(store.get_schema_version().unwrap(), 1);

        let after_rb = store.get_project(&proj1.id).unwrap().unwrap();
        assert_eq!(after_rb.name, "Project One");
        assert_eq!(after_rb.loopback_port, None);
    }

    #[test]
    fn test_interrupted_migration_recovery_and_duplicate_detection() {
        let store = MetadataStore::open_in_memory().unwrap();

        // Simulate interrupted migration: column added, but index creation never executed
        {
            let conn = store.conn.lock().unwrap();
            conn.execute("ALTER TABLE projects ADD COLUMN loopback_port INTEGER;", [])
                .unwrap();
            assert!(has_column(&conn, "projects", "loopback_port"));
            assert!(!has_unique_index(
                &conn,
                "projects",
                "idx_projects_loopback_port"
            ));
        }

        // Running migration should recover cleanly by creating the missing unique index
        let report = store.migrate_to_v2_loopback_ports().unwrap();
        assert!(report.applied);

        {
            let conn = store.conn.lock().unwrap();
            assert!(has_unique_index(
                &conn,
                "projects",
                "idx_projects_loopback_port"
            ));
        }

        // Test fail-closed behavior when duplicates exist before indexing
        let store2 = MetadataStore::open_in_memory().unwrap();
        {
            let conn = store2.conn.lock().unwrap();
            conn.execute("ALTER TABLE projects ADD COLUMN loopback_port INTEGER;", [])
                .unwrap();
            // Insert two rows with identical loopback_port
            conn.execute(
                "INSERT INTO projects (id, name, path, volume_uuid, document_root, php_version, local_domain, status, created_at, updated_at, loopback_port)
                 VALUES ('p1', 'P1', '/path1', 'v1', '', '8.4', 'p1.test', 'stopped', 'now', 'now', 18081),
                        ('p2', 'P2', '/path2', 'v2', '', '8.4', 'p2.test', 'stopped', 'now', 'now', 18081)",
                [],
            ).unwrap();
        }

        // Migration must fail closed rather than corrupting or discarding data
        let fail_res = store2.migrate_to_v2_loopback_ports();
        assert!(fail_res.is_err());
    }
}
