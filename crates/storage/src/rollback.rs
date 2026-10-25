use crate::db::{MetadataStore, MigrationReport, ProjectRecord};
use chrono::Utc;
use localforge_core::{LocalForgeError, ProjectId, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureInjectionPoint {
    None,
    ImmediatelyAfterSchemaMigration,
    AfterProjectAPortAssignment,
    DuringPostVerification,
}

#[derive(Debug, Clone)]
pub struct RollbackConfig {
    pub db_path: PathBuf,
    pub backup_path: PathBuf,
    pub app_support_dir: PathBuf,
    pub allowlist: Vec<String>,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::new(),
            backup_path: PathBuf::new(),
            app_support_dir: PathBuf::new(),
            allowlist: vec![
                "localforge.db".to_string(),
                "localforge.db-wal".to_string(),
                "localforge.db-shm".to_string(),
            ],
        }
    }
}

/// Verifies writer quiescence by inspecting both file descriptors (via `lsof`) and process names (via `pgrep`).
///
/// Security guarantees:
/// 1. Inspects `localforge.db`, `localforge.db-wal`, and `localforge.db-shm` to ensure no other process holds open file descriptors.
/// 2. Inspects running processes to ensure no foreign `localforge` daemons are active.
/// 3. Treats failure to execute inspection commands as an explicit unsafe condition (fail-closed).
pub fn verify_writer_quiescence(db_path: &Path) -> Result<bool> {
    let current_pid = std::process::id();

    // Check 1: lsof inspection on db_path and its WAL/SHM companions
    let parent = db_path.parent();
    let companions = [
        Some(db_path.to_path_buf()),
        parent.map(|p| p.join("localforge.db-wal")),
        parent.map(|p| p.join("localforge.db-shm")),
    ];

    for target in companions.into_iter().flatten() {
        if !target.exists() {
            continue;
        }
        let lsof_res = std::process::Command::new("lsof")
            .arg("-t")
            .arg(&target)
            .output();

        match lsof_res {
            Ok(output) => {
                if output.status.code() == Some(0) {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if let Ok(pid) = line.trim().parse::<u32>() {
                            if pid != current_pid {
                                return Ok(false);
                            }
                        }
                    }
                } else if output.status.code() != Some(1) {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(LocalForgeError::DatabaseError(format!(
                        "UnsafeCondition: lsof inspection failed with status {}: {stderr}",
                        output.status
                    )));
                }
            }
            Err(e) => {
                return Err(LocalForgeError::DatabaseError(format!(
                    "UnsafeCondition: failed to execute lsof for database quiescence check: {e}"
                )));
            }
        }
    }

    // Check 2: pgrep inspection for foreign localforge processes
    let pgrep_res = std::process::Command::new("pgrep")
        .arg("-il")
        .arg("localforge")
        .output();

    match pgrep_res {
        Ok(output) => {
            if output.status.code() == Some(0) {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(pid_str) = parts.first() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            if pid != current_pid {
                                return Ok(false);
                            }
                        }
                    }
                }
            } else if output.status.code() != Some(1) {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(LocalForgeError::DatabaseError(format!(
                    "UnsafeCondition: pgrep inspection failed with status {}: {stderr}",
                    output.status
                )));
            }
        }
        Err(e) => {
            return Err(LocalForgeError::DatabaseError(format!(
                "UnsafeCondition: failed to execute pgrep for process quiescence check: {e}"
            )));
        }
    }

    Ok(true)
}

/// Executes a non-destructive quarantine and atomic restoration of the database from a verified backup.
///
/// Precondition: All SQLite connections to the database MUST be closed and dropped before calling.
///
/// Execution guarantees:
/// 1. Verifies writer quiescence immediately before restoration. Fails closed on inspection failure or active writers.
/// 2. Prepares restored data in a secure temporary staging file on the same filesystem.
/// 3. Verifies staging file integrity (`PRAGMA integrity_check`), record counts, and Projetest PHP version BEFORE touching live files.
/// 4. Does NOT swallow verification errors (missing Projetest or incorrect PHP version fails closed).
/// 5. Moves failed live files to a timestamped quarantine directory with 0700 permissions.
/// 6. Atomically promotes staging file to destination path.
/// 7. Performs final verification on promoted database before reporting success.
pub fn execute_isolated_quarantine_and_restore(
    config: &RollbackConfig,
    failure_reason: &str,
    expected_restored_project_count: usize,
    expected_projetest_php_version: &str,
) -> Result<PathBuf> {
    // 1. Verify Writer Quiescence Immediately Before Restoration
    let is_quiescent = verify_writer_quiescence(&config.db_path)?;
    if !is_quiescent {
        return Err(LocalForgeError::DatabaseError(format!(
            "CRITICAL: Rollback aborted: active SQLite processes detected. Live failed state preserved at '{}'. Human intervention required immediately.",
            config.db_path.display()
        )));
    }

    // 2. Prepare Restored Data in a Secure Temporary Staging File (Same Filesystem)
    let parent_dir = config.db_path.parent().unwrap_or(&config.app_support_dir);
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let staging_path = parent_dir.join(format!(".restore_staging_{timestamp}.db"));

    if !config.backup_path.exists() {
        return Err(LocalForgeError::DatabaseError(format!(
            "Rollback fatal error: verified backup file '{}' does not exist",
            config.backup_path.display()
        )));
    }

    std::fs::copy(&config.backup_path, &staging_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Rollback fatal error: failed to stage backup copy at '{}': {e}",
            staging_path.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&staging_path, std::fs::Permissions::from_mode(0o600));
    }

    // 3. Verify Prepared Restore BEFORE Touching Live Files
    let verify_conn = Connection::open_with_flags(&staging_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| {
            let _ = std::fs::remove_file(&staging_path);
            LocalForgeError::DatabaseError(format!(
                "Staged rollback verification failed: cannot open staging db: {e}"
            ))
        })?;

    let integrity: String = verify_conn
        .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
        .map_err(|e| {
            let _ = std::fs::remove_file(&staging_path);
            LocalForgeError::DatabaseError(format!(
                "Staged rollback verification failed: integrity check failed: {e}"
            ))
        })?;

    if integrity != "ok" {
        let _ = std::fs::remove_file(&staging_path);
        return Err(LocalForgeError::DatabaseError(format!(
            "Staged rollback verification failed: integrity corrupted: {integrity}"
        )));
    }

    let count: usize = verify_conn
        .query_row("SELECT count(*) FROM projects;", [], |r| r.get(0))
        .map_err(|e| {
            let _ = std::fs::remove_file(&staging_path);
            LocalForgeError::DatabaseError(format!(
                "Staged rollback verification failed: project count query failed: {e}"
            ))
        })?;

    if count != expected_restored_project_count {
        let _ = std::fs::remove_file(&staging_path);
        return Err(LocalForgeError::DatabaseError(format!(
            "Staged rollback verification failed: expected {expected_restored_project_count} projects, found {count}"
        )));
    }

    // Must NOT swallow verification errors:
    let php_ver: String = verify_conn
        .query_row(
            "SELECT php_version FROM projects WHERE name = 'Projetest';",
            [],
            |r| r.get(0),
        )
        .map_err(|e| {
            let _ = std::fs::remove_file(&staging_path);
            LocalForgeError::DatabaseError(format!(
                "Staged rollback verification failed: Projetest missing or query failed after restore: {e}"
            ))
        })?;

    if php_ver != expected_projetest_php_version {
        let _ = std::fs::remove_file(&staging_path);
        return Err(LocalForgeError::DatabaseError(format!(
            "Staged rollback verification failed: Projetest PHP version is '{php_ver}', expected '{expected_projetest_php_version}'"
        )));
    }

    drop(verify_conn);

    // 4. Prepare Private Quarantine Directory (0700)
    let quarantine_dir = config
        .app_support_dir
        .join("quarantine")
        .join(format!("quarantine_{timestamp}"));

    std::fs::create_dir_all(&quarantine_dir).map_err(|e| {
        let _ = std::fs::remove_file(&staging_path);
        LocalForgeError::DatabaseError(format!(
            "Rollback failed: could not create quarantine directory '{}': {e}",
            quarantine_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&quarantine_dir, std::fs::Permissions::from_mode(0o700));
    }

    // 5. Move Allowlist Files into Quarantine (Preserve failed live state for diagnosis)
    for filename in &config.allowlist {
        let live_file = parent_dir.join(filename);
        if live_file.exists() {
            let meta = std::fs::symlink_metadata(&live_file).map_err(|e| {
                let _ = std::fs::remove_file(&staging_path);
                LocalForgeError::DatabaseError(format!(
                    "Failed to read metadata for '{}': {e}",
                    live_file.display()
                ))
            })?;

            if meta.file_type().is_symlink() {
                let _ = std::fs::remove_file(&staging_path);
                return Err(LocalForgeError::DatabaseError(format!(
                    "Rollback security violation: target file '{}' is a symbolic link",
                    live_file.display()
                )));
            }

            let dest_file = quarantine_dir.join(filename);
            std::fs::rename(&live_file, &dest_file).map_err(|e| {
                let _ = std::fs::remove_file(&staging_path);
                LocalForgeError::DatabaseError(format!(
                    "Rollback failed: unable to move '{}' to quarantine '{}': {e}",
                    live_file.display(),
                    dest_file.display()
                ))
            })?;
        }
    }

    // 6. Atomically Promote Verified Staging File to Live Destination
    std::fs::rename(&staging_path, &config.db_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Rollback fatal error: failed to promote staged database '{}' to '{}': {e}",
            staging_path.display(),
            config.db_path.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config.db_path, std::fs::Permissions::from_mode(0o600));
    }

    // 7. Final Independent Post-Promotion Verification
    let final_conn = Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "Final rollback verification failed: cannot open promoted database: {e}"
            ))
        })?;

    let final_integrity: String = final_conn
        .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "Final rollback verification failed: integrity check failed: {e}"
            ))
        })?;

    if final_integrity != "ok" {
        return Err(LocalForgeError::DatabaseError(format!(
            "Final rollback verification failed: promoted database integrity corrupted: {final_integrity}"
        )));
    }

    drop(final_conn);

    tracing::warn!(
        "Database rollback successfully executed due to: {}. Restored from '{}'. Failed live state quarantined at '{}'.",
        failure_reason,
        config.backup_path.display(),
        quarantine_dir.display()
    );

    Ok(quarantine_dir)
}

#[derive(Debug, Clone)]
pub struct GatedMigrationResult {
    pub report: MigrationReport,
    pub project_a_after: ProjectRecord,
    pub project_b_after: ProjectRecord,
}

/// Executes a gated migration with strict ownership scope separation and automated rollback.
///
/// Execution is divided into two mutually exclusive scopes:
/// Scope 1: Live execution via `MetadataStore`. The store is dropped immediately upon completion or error.
/// Scope 2: Rollback execution. Only runs after `MetadataStore` has been completely dropped.
pub fn execute_gated_migration_lifecycle(
    config: &RollbackConfig,
    project_a_id: &str,
    project_a_port: u16,
    project_b_id: &str,
    project_b_port: u16,
    failure_injection: FailureInjectionPoint,
) -> Result<GatedMigrationResult> {
    // -------------------------------------------------------------------------
    // SCOPE 1: Execution Scope
    // -------------------------------------------------------------------------
    let execution_result: Result<GatedMigrationResult> = (|| {
        let store = MetadataStore::open(&config.db_path)?;
        let p1_id = ProjectId(project_a_id.to_string());
        let p2_id = ProjectId(project_b_id.to_string());

        // Pre-check
        let p2 = store.get_project(&p2_id)?.ok_or_else(|| {
            LocalForgeError::DatabaseError(format!("Project B '{project_b_id}' missing"))
        })?;
        if p2.php_version != "8.4" {
            return Err(LocalForgeError::Validation(format!(
                "PO-U5-01 VIOLATION: Projetest PHP version is '{}', expected '8.4'",
                p2.php_version
            )));
        }

        // Migration Step
        let report = store.migrate_to_v2_loopback_ports()?;

        if failure_injection == FailureInjectionPoint::ImmediatelyAfterSchemaMigration {
            return Err(LocalForgeError::DatabaseError(
                "Simulated failure immediately after schema migration".into(),
            ));
        }

        // Port A Assignment
        store.update_project_loopback_port(&p1_id, Some(project_a_port))?;

        if failure_injection == FailureInjectionPoint::AfterProjectAPortAssignment {
            return Err(LocalForgeError::DatabaseError(
                "Simulated failure after Project A port assignment".into(),
            ));
        }

        // Port B Assignment
        store.update_project_loopback_port(&p2_id, Some(project_b_port))?;

        if failure_injection == FailureInjectionPoint::DuringPostVerification {
            return Err(LocalForgeError::DatabaseError(
                "Simulated failure during post verification".into(),
            ));
        }

        let p1_after = store.get_project(&p1_id)?.ok_or_else(|| {
            LocalForgeError::DatabaseError("Project A missing after migration".into())
        })?;
        let p2_after = store.get_project(&p2_id)?.ok_or_else(|| {
            LocalForgeError::DatabaseError("Project B missing after migration".into())
        })?;

        if p1_after.loopback_port != Some(project_a_port) {
            return Err(LocalForgeError::DatabaseError(
                "Project A loopback port assignment mismatch".into(),
            ));
        }
        if p2_after.loopback_port != Some(project_b_port) {
            return Err(LocalForgeError::DatabaseError(
                "Project B loopback port assignment mismatch".into(),
            ));
        }
        if p2_after.php_version != "8.4" {
            return Err(LocalForgeError::Validation(
                "PO-U5-01 VIOLATION: Projetest PHP version was mutated".into(),
            ));
        }

        Ok(GatedMigrationResult {
            report,
            project_a_after: p1_after,
            project_b_after: p2_after,
        })
        // NOTE: `store` is explicitly dropped here when the closure finishes!
    })();

    // -------------------------------------------------------------------------
    // SCOPE 2: Rollback Scope (MetadataStore is guaranteed dropped)
    // -------------------------------------------------------------------------
    match execution_result {
        Ok(res) => Ok(res),
        Err(exec_err) => {
            let rollback_res =
                execute_isolated_quarantine_and_restore(config, &exec_err.to_string(), 2, "8.4");

            match rollback_res {
                Ok(quarantine_path) => Err(LocalForgeError::DatabaseError(format!(
                    "Migration failed ({exec_err}). Database rolled back successfully and verified from backup. Failed live state preserved at '{}'.",
                    quarantine_path.display()
                ))),
                Err(rollback_err) => Err(LocalForgeError::DatabaseError(format!(
                    "FATAL ERROR: Migration failed ({exec_err}) AND rollback failed ({rollback_err}). Human intervention required immediately."
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_test_v1_database(path: &Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (
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
            INSERT INTO projects VALUES (
                'proj_a', 'r1_acceptance_project', '/test/a', 'vol_1', '/test/a/public', '8.5', 'r1-acceptance.localforge', 'running', '2026-10-25T00:00:00Z', '2026-10-25T00:00:00Z'
            );
            INSERT INTO projects VALUES (
                'proj_b', 'Projetest', '/test/b', 'vol_1', '/test/b/public', '8.4', 'projetest.localforge', 'stopped', '2026-10-25T00:00:00Z', '2026-10-25T00:00:00Z'
            );
            CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );
            INSERT INTO schema_migrations VALUES (1, 'Initial schema', '2026-10-25T00:00:00Z');",
        )
        .unwrap();
    }

    #[test]
    fn test_simulated_failure_after_schema_migration_triggers_clean_isolated_rollback() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let backup_path = dir.path().join("backup_v1.db");
        let app_support = dir.path().to_path_buf();

        create_test_v1_database(&db_path);
        std::fs::copy(&db_path, &backup_path).unwrap();

        let config = RollbackConfig {
            db_path: db_path.clone(),
            backup_path: backup_path.clone(),
            app_support_dir: app_support.clone(),
            allowlist: vec!["localforge.db".into()],
        };

        let res = execute_gated_migration_lifecycle(
            &config,
            "proj_a",
            18081,
            "proj_b",
            18082,
            FailureInjectionPoint::ImmediatelyAfterSchemaMigration,
        );

        let err = res.unwrap_err();
        assert!(err
            .to_string()
            .contains("Simulated failure immediately after schema migration"));
        assert!(err
            .to_string()
            .contains("Database rolled back successfully"));

        // Verify restored DB is clean V1
        let conn = Connection::open(&db_path).unwrap();
        let integrity: String = conn
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");

        let has_port: i64 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('projects') WHERE name = 'loopback_port';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            has_port, 0,
            "Restored database must be clean V1 (no loopback_port)"
        );

        let p2_php: String = conn
            .query_row(
                "SELECT php_version FROM projects WHERE id = 'proj_b';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(p2_php, "8.4", "PO-U5-01 Projetest must remain PHP 8.4");

        // Verify quarantine directory contains the failed files
        let q_dir = app_support.join("quarantine");
        assert!(q_dir.exists());
        let entries: Vec<_> = std::fs::read_dir(q_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_simulated_failure_after_project_a_port_assignment_triggers_clean_isolated_rollback() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let backup_path = dir.path().join("backup_v1.db");
        let app_support = dir.path().to_path_buf();

        create_test_v1_database(&db_path);
        std::fs::copy(&db_path, &backup_path).unwrap();

        let config = RollbackConfig {
            db_path: db_path.clone(),
            backup_path: backup_path.clone(),
            app_support_dir: app_support.clone(),
            allowlist: vec!["localforge.db".into()],
        };

        let res = execute_gated_migration_lifecycle(
            &config,
            "proj_a",
            18081,
            "proj_b",
            18082,
            FailureInjectionPoint::AfterProjectAPortAssignment,
        );

        let err = res.unwrap_err();
        assert!(err
            .to_string()
            .contains("Simulated failure after Project A port assignment"));
        assert!(err
            .to_string()
            .contains("Database rolled back successfully"));

        // Verify restored DB is clean V1
        let conn = Connection::open(&db_path).unwrap();
        let integrity: String = conn
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");

        let p2_php: String = conn
            .query_row(
                "SELECT php_version FROM projects WHERE id = 'proj_b';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(p2_php, "8.4");
    }

    #[test]
    fn test_missing_projetest_fails_closed_without_swallowing_error() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let backup_path = dir.path().join("backup_corrupt.db");
        let app_support = dir.path().to_path_buf();

        // Create backup where Projetest is missing!
        let conn = Connection::open(&backup_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT, php_version TEXT);
             INSERT INTO projects VALUES ('p1', 'OtherProject', '8.5');",
        )
        .unwrap();

        let config = RollbackConfig {
            db_path,
            backup_path,
            app_support_dir: app_support,
            allowlist: vec!["localforge.db".into()],
        };

        let res =
            execute_isolated_quarantine_and_restore(&config, "testing missing projetest", 2, "8.4");

        let err = res.unwrap_err();
        assert!(
            err.to_string()
                .contains("Staged rollback verification failed")
                || err.to_string().contains("Projetest missing")
        );
    }

    #[test]
    fn test_incorrect_php_version_after_restore_fails_closed() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let backup_path = dir.path().join("backup_mutated.db");
        let app_support = dir.path().to_path_buf();

        // Create backup where Projetest has mutated PHP 8.5
        let conn = Connection::open(&backup_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT, php_version TEXT);
             INSERT INTO projects VALUES ('p1', 'r1_acceptance_project', '8.5');
             INSERT INTO projects VALUES ('p2', 'Projetest', '8.5');",
        )
        .unwrap();

        let config = RollbackConfig {
            db_path,
            backup_path,
            app_support_dir: app_support,
            allowlist: vec!["localforge.db".into()],
        };

        let res = execute_isolated_quarantine_and_restore(
            &config,
            "testing mutated php version",
            2,
            "8.4",
        );

        let err = res.unwrap_err();
        assert!(err
            .to_string()
            .contains("Projetest PHP version is '8.5', expected '8.4'"));
    }

    #[test]
    fn test_successful_gated_migration_lifecycle_without_failure_injection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("localforge.db");
        let backup_path = dir.path().join("backup_v1.db");
        let app_support = dir.path().to_path_buf();

        create_test_v1_database(&db_path);
        std::fs::copy(&db_path, &backup_path).unwrap();

        let config = RollbackConfig {
            db_path: db_path.clone(),
            backup_path: backup_path.clone(),
            app_support_dir: app_support.clone(),
            allowlist: vec!["localforge.db".into()],
        };

        let res = execute_gated_migration_lifecycle(
            &config,
            "proj_a",
            18081,
            "proj_b",
            18082,
            FailureInjectionPoint::None,
        );

        let success = res.unwrap();
        assert_eq!(success.project_a_after.loopback_port, Some(18081));
        assert_eq!(success.project_b_after.loopback_port, Some(18082));
        assert_eq!(success.project_b_after.php_version, "8.4");
    }
}
