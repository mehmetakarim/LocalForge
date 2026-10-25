use localforge_core::{LocalForgeError, Result};
use rusqlite::{Connection, OpenFlags, Row};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tempfile::NamedTempFile;

/// Computes SHA-256 hex digest of a file safely without loading entire contents into memory.
pub fn compute_file_sha256(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "Failed to open file '{}' for SHA-256 computation: {e}",
            path.display()
        ))
    })?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer).map_err(|e| {
            LocalForgeError::DatabaseError(format!("Read error during SHA-256 computation: {e}"))
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let mut s = String::with_capacity(64);
    for byte in digest {
        s.push_str(&format!("{:02x}", byte));
    }
    Ok(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshotRecord {
    pub id: String,
    pub name: String,
    pub php_version: String,
    pub local_domain: String,
    pub loopback_port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct WalSnapshotAudit {
    pub database_sha256: String,
    pub integrity_status: String,
    pub has_loopback_port_column: bool,
    pub schema_version: u32,
    pub projects: Vec<ProjectSnapshotRecord>,
}

/// Takes a consistent, WAL-aware read-only snapshot of an active SQLite database using
/// the official SQLite Online Backup API.
///
/// Safety guarantees:
/// 1. Does NOT use `immutable=1`, ensuring all uncheckpointed WAL frames are captured.
/// 2. Live database connection is opened strictly with `SQLITE_OPEN_READ_ONLY`.
/// 3. Live connection is dropped immediately upon completion of backup.
/// 4. Live database, WAL, and SHM files incur zero mutations.
/// 5. Validates integrity of the resulting snapshot before returning.
pub fn create_wal_safe_snapshot(live_db_path: &Path, snapshot_dest_path: &Path) -> Result<()> {
    // 1. Verify existence and read permissions
    if !live_db_path.exists() {
        return Err(LocalForgeError::DatabaseError(format!(
            "UnreadableLiveDatabase: database file '{}' does not exist",
            live_db_path.display()
        )));
    }

    // 2. Open live connection in pure read-only mode (never immutable=1!)
    let live_uri = format!("file:{}", live_db_path.display());
    let live_conn = Connection::open_with_flags(
        &live_uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "UnreadableLiveDatabase: failed to open live database in read-only mode: {e}"
        ))
    })?;

    // 3. Open snapshot destination in create/write mode
    let mut dest_conn = Connection::open(snapshot_dest_path).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "InconsistentSnapshot: failed to open destination snapshot file '{}': {e}",
            snapshot_dest_path.display()
        ))
    })?;

    // 4. Run SQLite Backup API to completion
    let backup = rusqlite::backup::Backup::new(&live_conn, &mut dest_conn).map_err(|e| {
        LocalForgeError::DatabaseError(format!(
            "InconsistentSnapshot: failed to initialize SQLite backup: {e}"
        ))
    })?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(10), None)
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "InconsistentSnapshot: backup step failed or interrupted: {e}"
            ))
        })?;

    // 5. Explicitly drop backup and live connection immediately — zero lingering handles
    drop(backup);
    drop(live_conn);

    // 6. Verify integrity of destination snapshot
    let integrity: String = dest_conn
        .query_row("PRAGMA integrity_check;", [], |r: &Row| r.get(0))
        .map_err(|e| {
            LocalForgeError::DatabaseError(format!(
                "CorruptSnapshot: PRAGMA integrity_check query failed on snapshot: {e}"
            ))
        })?;

    if integrity != "ok" {
        return Err(LocalForgeError::DatabaseError(format!(
            "CorruptSnapshot: snapshot integrity check failed: {integrity}"
        )));
    }

    Ok(())
}

/// Executes a WAL-safe, read-only preflight audit against an active SQLite database.
/// The snapshot is created in a secure temporary file and cleaned up upon drop.
pub fn inspect_wal_database_preflight(live_db_path: &Path) -> Result<WalSnapshotAudit> {
    let temp_snapshot = NamedTempFile::new().map_err(|e| {
        LocalForgeError::DatabaseError(format!("Failed to create temporary snapshot file: {e}"))
    })?;

    create_wal_safe_snapshot(live_db_path, temp_snapshot.path())?;

    let sha256 = compute_file_sha256(temp_snapshot.path())?;

    let snap_conn =
        Connection::open_with_flags(temp_snapshot.path(), OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                LocalForgeError::DatabaseError(format!(
                    "Failed to open verified snapshot for inspection: {e}"
                ))
            })?;

    let integrity: String = snap_conn
        .query_row("PRAGMA integrity_check;", [], |r: &Row| r.get(0))
        .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

    let has_port_col: i64 = snap_conn
        .prepare("SELECT count(*) FROM pragma_table_info('projects') WHERE name = 'loopback_port';")
        .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?
        .query_row([], |r: &Row| r.get(0))
        .unwrap_or(0);

    let schema_ver: u32 = snap_conn
        .query_row("PRAGMA user_version;", [], |r: &Row| r.get(0))
        .unwrap_or(0);

    let mut stmt = snap_conn
        .prepare("SELECT id, name, php_version, local_domain FROM projects ORDER BY name ASC;")
        .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row: &Row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

    let mut projects = Vec::new();
    for r in rows {
        let (id, name, php_version, local_domain) =
            r.map_err(|e| LocalForgeError::DatabaseError(e.to_string()))?;

        let port: Option<u16> = if has_port_col > 0 {
            snap_conn
                .query_row(
                    "SELECT loopback_port FROM projects WHERE id = ?1",
                    [&id],
                    |row: &Row| row.get(0),
                )
                .ok()
        } else {
            None
        };

        projects.push(ProjectSnapshotRecord {
            id,
            name,
            php_version,
            local_domain,
            loopback_port: port,
        });
    }

    Ok(WalSnapshotAudit {
        database_sha256: sha256,
        integrity_status: integrity,
        has_loopback_port_column: has_port_col > 0,
        schema_version: schema_ver,
        projects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wal_uncheckpointed_data_captured_in_snapshot_without_live_mutation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_wal.db");

        // 1. Initialize SQLite database in WAL mode and hold writer open
        let live_writer = Connection::open(&db_path).unwrap();
        live_writer
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE test_data (id INTEGER PRIMARY KEY, msg TEXT);
                 INSERT INTO test_data (msg) VALUES ('committed_frame_1');",
            )
            .unwrap();

        // Verify WAL file exists
        let wal_path = dir.path().join("test_wal.db-wal");
        assert!(wal_path.exists(), "WAL file should exist after WAL write");

        // Record WAL metadata before snapshot
        let wal_meta_before = std::fs::metadata(&wal_path).unwrap();

        // 2. Take WAL-safe snapshot
        let snapshot_file = NamedTempFile::new().unwrap();
        create_wal_safe_snapshot(&db_path, snapshot_file.path()).unwrap();

        // Verify live WAL was NOT modified
        let wal_meta_after = std::fs::metadata(&wal_path).unwrap();
        assert_eq!(
            wal_meta_before.modified().unwrap(),
            wal_meta_after.modified().unwrap(),
            "Live WAL file must not be modified during snapshot"
        );

        // 3. Inspect snapshot and verify it captured the uncheckpointed WAL row
        let snap_conn =
            Connection::open_with_flags(snapshot_file.path(), OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();

        let count: i64 = snap_conn
            .query_row("SELECT count(*) FROM test_data;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let msg: String = snap_conn
            .query_row("SELECT msg FROM test_data WHERE id = 1;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(msg, "committed_frame_1");
    }

    #[test]
    fn test_unreadable_live_database_error() {
        let dir = tempdir().unwrap();
        let non_existent = dir.path().join("does_not_exist.db");
        let dest = dir.path().join("dest.db");

        let err = create_wal_safe_snapshot(&non_existent, &dest).unwrap_err();
        assert!(err.to_string().contains("UnreadableLiveDatabase"));
    }
}
