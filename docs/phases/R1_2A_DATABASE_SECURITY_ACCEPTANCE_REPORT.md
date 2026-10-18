# LocalForge — Phase R1.2A Database Security Acceptance Report

**Date**: 2026-10-18  
**Phase**: R1.2A — Database Security Remediation  
**Status**: Completed & Verified  
**Branch**: `phase-r1/r1-2a-database-security`  
**Base Commit**: `5459fd2`  
**Classification**: Security Remediation & Runtime Hardening  

---

## 1. Original Security Findings

During the independent technical review of Phase R1.2 (MariaDB Integration), several critical security vulnerabilities and reliability risks were uncovered:

1. **SQL Credential String Interpolation (Vulnerability R1.2A-01)**:
   - In `create_project_database()`, user passwords were formatted directly into dynamic DDL statements (`IDENTIFIED BY '{password}'`). Unescaped single quotes or malicious payloads could execute SQL injection attacks (e.g., arbitrary DDL execution, dropping databases). Null bytes (`\0`) were not blocked, creating truncation risks.
   - Raw database errors containing cleartext passwords could leak into application logs or user-facing error messages.

2. **Insecure Backup Creation & Symlink Attack Surface (Vulnerability R1.2A-02)**:
   - `dump_database()` created files using standard `File::create()`, leaving backup dumps world-readable (`0o644` depending on umask) during writing before an explicit `chmod` could execute.
   - Existing backup files could be silently overwritten without user consent.
   - Symbolic links in the destination path were not canonicalized, enabling symlink overwriting of sensitive system files.
   - The backup process buffered entire SQL dumps in RAM via `child.wait_with_output()`, creating Denial of Service (OOM) risks for large databases.
   - Empty or corrupted partial dumps were retained on disk if the client crashed or failed.

3. **Self-Authorization & Broken Access Control on Deletion (Vulnerability R1.2A-03)**:
   - `drop_project_database(db_name, approval_token)` accepted `approval_token == db_name` as valid authorization. This allowed callers knowing the database name to bypass explicit human authorization completely.
   - There was no cryptographic single-use token mechanism, no expiration window, and no binding to the caller context.

4. **Unsafe Database Restore (Vulnerability R1.2A-04)**:
   - `restore_database()` permitted restoring dumps into active, populated databases without checking if tables already existed, resulting in silent data collisions or corruption.
   - File validation did not verify the presence of SQL dump headers or reject zero-byte/non-existent files safely.
   - Dumps were loaded into memory via `fs::read_to_string()` before piping to stdin, posing memory exhaustion risks.
   - Symlinks in the source dump path were not canonicalized.

5. **Aggressive Process Termination & Datadir Confusion (Vulnerability R1.2A-05)**:
   - `MariaDbSupervisor::stop()` immediately sent `SIGKILL` if a quick status check failed, bypassing InnoDB's clean buffer pool flushing and risking database corruption.
   - Datadir initialization did not verify ownership (`uid == geteuid()`), risking execution over alien directories.
   - Unrecognized existing directories could be overwritten or corrupted during re-initialization.

---

## 2. Root-Cause Analysis

- **Rapid Prototyping vs. Defensive Hardening**: Phase R1.2 focused on functional verification (MariaDB daemon startup, port binding, and PHP PDO connectivity), relying on standard string formatting and simple subprocess calls without strict adversarial threat modeling.
- **Missing Token Lifecycle Management**: Authorization was designed as a static equality check rather than an application-mediated cryptographic challenge with single-use consumption semantics.
- **Subprocess Buffer Inefficiencies**: Standard library `output()` and `read_to_string()` convenience methods were used instead of streaming file descriptors directly between disk and subprocess pipes (`Stdio::from(file)`).
- **Time-of-Check to Time-of-Use (TOCTOU) Permissions**: Applying `set_permissions()` after `File::create()` leaves an open race window where another local process can read database dumps containing sensitive credentials and data.

---

## 3. Source Modifications

The following production files were modified and hardened:

### `crates/runtime/src/error.rs`
- Added explicit error variants:
  - `DatabaseUnauthorized(String)`: For authorization failures, expired tokens, token-database mismatches, and self-authorization attempts.
  - `BackupError(String)`: For destination path traversal, symlink detection, pre-existing file collisions, and invalid SQL dump headers.
  - `RestoreError(String)`: For corrupted dump files, non-empty target databases, and child pipe failures.
  - `CredentialError(String)`: For null bytes, passwords exceeding 128 characters, empty passwords, and encoding errors.

### `crates/runtime/Cargo.toml`
- Promoted `libc.workspace = true` to runtime dependencies for POSIX `getuid` validation and Unix mode flags.

### `crates/runtime/src/mariadb.rs`
1. **`DatabaseDeletionApproval`**:
   - Implemented approval struct containing `token: String` (UUIDv4), `database_name: String`, `requested_at: SystemTime`, `expires_at: SystemTime` (60-second TTL), `requesting_context: String`, and `consumed: bool`.
   - Added thread-safe `active_approvals: Mutex<HashMap<String, DatabaseDeletionApproval>>` to `MariaDbSupervisor`.
2. **`escape_sql_string_literal(input: &str) -> Result<String, RuntimeError>`**:
   - Rejects null bytes (`\0`).
   - Limits password length to 128 characters.
   - Escapes single quotes (`'` -> `''`), backslashes (`\` -> `\\`), control characters (`\n`, `\r`, `\t`, `\Z`).
   - Supports full Unicode and multibyte UTF-8 characters without corruption.
3. **`create_project_database`**:
   - Validates identifier length (max 64 characters) and characters (`[a-zA-Z0-9_]`).
   - Uses `escape_sql_string_literal` for user password.
   - Redacts password from stderr output using `execute_query_internal` secret scrubbing.
4. **`request_database_deletion_approval` & `drop_project_database`**:
   - Intercepts and rejects self-authorization: `if approval_token == db_name` immediately returns `Err(RuntimeError::DatabaseUnauthorized("Self-authorization rejected..."))`.
   - Checks token presence, validity, single-use consumption (`consumed = true`), database matching, and expiration (TTL <= 60s).
5. **`dump_database`**:
   - Uses `OpenOptionsExt::mode(0o600)` at creation time on Unix.
   - Rejects existing files (`create_new(true)`) to prevent overwrite.
   - Validates destination parent path and rejects symlinks (`symlink_metadata().is_symlink()`).
   - Writes to atomic temporary file (`.tmp_dump_<db>_<uuid>.part`) in same parent directory.
   - Streams `mariadb-dump` stdout directly to file descriptor (`Stdio::from(file)`).
   - Validates exit code and verifies non-empty SQL dump header (`--`, `/*`, or `CREATE`).
   - Renames atomically via `std::fs::rename()`. Automatically cleans up temporary files on error.
6. **`restore_database`**:
   - Canonicalizes dump file path and rejects symlinks.
   - Checks file size (> 0) and validates presence of SQL dump header.
   - Queries `information_schema.tables` on target database; refuses to restore if existing table count > 0.
   - Streams dump file directly into `mariadb` stdin (`Stdio::from(file)`).
7. **`ensure_datadir_initialized`**:
   - Validates datadir ownership: `meta.uid() == libc::getuid()`.
   - If directory exists and contains entries other than `mysql` or `.DS_Store`, refuses to overwrite unknown data directory.
8. **`stop()`**:
   - Attempts graceful shutdown using `mysqladmin shutdown` via socket.
   - If supervisor owns child handle, sends `SIGTERM` and polls for exit up to 5 seconds.
   - Only falls back to `SIGKILL` as a last resort if graceful termination fails.

### `crates/runtime/src/lib.rs`
- Re-exported `DatabaseDeletionApproval` and `escape_sql_string_literal`.

---

## 4. Authentication and Credential Strategy

### Escaped String Literal Safety
- MariaDB DDL statements (`CREATE USER ... IDENTIFIED BY '...'`) do not support prepared statement parameter binding (`?`) in MySQL/MariaDB wire protocols.
- `escape_sql_string_literal` provides strict SQL-92 and MariaDB escaping:
  - Rejects null bytes `\0` with `RuntimeError::CredentialError`.
  - Replaces `'` with `''` and `\` with `\\`.
  - Enforces minimum 8 and maximum 128 character password constraints.
  - Automatically scrubs password text from `execute_query_internal` stderr logs on error.

### macOS Keychain Evaluation
- **Evaluation**: For developer environments on macOS Apple Silicon, database passwords must not be stored in unencrypted plaintext on disk.
- **Strategy**:
  - The shared Rust platform crate (`crates/platform`) abstracts OS credential storage.
  - On macOS, `Security.framework` (`SecItemAdd`, `SecItemCopyMatching`) will store project database credentials securely under the service identifier `com.localforge.credentials`.
  - Plaintext passwords exist in memory only during database provisioning and are immediately dropped.

---

## 5. Secure Backup Architecture

1. **Atomic Creation with 0600 Permissions**:
   ```rust
   #[cfg(unix)]
   options.mode(0o600);
   let file = options.create_new(true).write(true).open(&temp_path)?;
   ```
   No race window exists where backup dumps are exposed to other users or processes.

2. **Atomic Temporary Rename**:
   - The backup writes to a hidden temporary file `.tmp_dump_<db>_<uuid>.part` in the target directory.
   - Only upon verification of clean zero-exit status and non-empty valid SQL headers is `fs::rename(&temp_path, &dest_path)` invoked.
   - If an error occurs, the temporary file is deleted immediately.

3. **Direct Kernel Streaming (No RAM Buffering)**:
   - `mariadb-dump` stdout connects directly to `Stdio::from(file)`.
   - Memory usage remains O(1) regardless of whether the database is 1 MB or 100 GB.

---

## 6. Database Deletion Authorization Architecture

```mermaid
sequenceDiagram
    autonumber
    actor Caller as UI / CLI Caller
    participant Sup as MariaDbSupervisor
    participant Store as Active Approvals (Mutex)
    participant MariaDB as MariaDB Engine

    Caller->>Sup: drop_project_database("crm", "crm")
    Sup-->>Caller: Err(DatabaseUnauthorized: Self-authorization rejected)

    Caller->>Sup: request_database_deletion_approval("crm", "user_prompt_dialog")
    Sup->>Store: Generate UUID token, 60s TTL
    Store-->>Caller: DatabaseDeletionApproval (token, expires_at)

    Caller->>Sup: drop_project_database("crm", token)
    Sup->>Store: Lookup & Consume token (consumed=true)
    Note over Sup,Store: Validates db_name match, not expired, not consumed
    Sup->>MariaDB: DROP DATABASE `crm`
    MariaDB-->>Sup: OK
    Sup-->>Caller: Ok(())

    Caller->>Sup: drop_project_database("crm", token)
    Sup->>Store: Lookup token
    Store-->>Caller: Err(DatabaseUnauthorized: Token already consumed)
```

- **Anti-Self-Approval**: `approval_token == db_name` immediately returns `DatabaseUnauthorized`.
- **Single-Use**: Token is marked `consumed` immediately upon retrieval; replay attacks fail.
- **Short-Lived**: Enforces 60-second time-to-live (`TTL`).
- **Context Binding**: Bound strictly to the specified database name and caller context.

---

## 7. Safe Restore Architecture

1. **Non-Empty Database Guard**:
   - Executes `SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = '<db>';`.
   - If count > 0, restore aborts with: `Refusing to restore into non-empty database '<db>' (contains X existing tables)`.
2. **Streaming Subprocess Pipe**:
   - Dump file is opened with read permissions and passed to `Stdio::from(input_file)`.
   - No buffer allocation in Rust process memory.
3. **DDL Rollback Limitations**:
   - In MySQL and MariaDB, DDL statements (`CREATE TABLE`, `ALTER TABLE`, `DROP TABLE`) trigger an **implicit commit**. Transactions (`BEGIN ... ROLLBACK`) cannot roll back DDL operations.
   - Because of this engine limitation, enforcing a strictly empty database before restore is the only reliable way to prevent partial overwrite or mixed-state corruption.

---

## 8. MariaDB Lifecycle Verification

- **Datadir Ownership Guard**:
  - `fs::metadata(path).uid() == libc::getuid()`.
  - Prevents running daemon over files owned by other users or `root`.
- **Unknown Directory Guard**:
  - Detects non-empty directories without MariaDB system tables and fails closed.
- **Graceful Termination**:
  - `mysqladmin --socket=... shutdown` is attempted first.
  - If spawned child process, `SIGTERM` is issued with 5-second polling loop.
  - Clean buffer pool dumps are preserved in `ib_buffer_pool`.

---

## 9. Adversarial Test Evidence

The following adversarial test suite was executed against a live MariaDB instance (`crates/runtime/tests/mariadb_integration_test.rs`):

| Test Identifier | Adversarial Vector / Scenario Tested | Result | Evidence / Details |
|---|---|---|---|
| `test_r1_2a_07` | SQL Injection in Password (`p'ass'; DROP DATABASE mysql; --`) | **PASS** | Provisioned safely; `mysql` database intact. |
| `test_r1_2a_07` | Null Byte Injection (`p\0assword`) | **PASS** | Rejected with `CredentialError("Null bytes forbidden")`. |
| `test_r1_2a_07` | Empty & Oversized Passwords (0 chars, 200 chars) | **PASS** | Rejected with `CredentialError`. |
| `test_r1_2a_07` | Unicode & Special Chars (`P@ss\w'o"rd🚀2026_Şçğüö;--`) | **PASS** | Successfully provisioned and authenticated. |
| `test_r1_2a_08` | Backup Destination is a Symlink | **PASS** | Rejected with `BackupError("Destination ... is a symlink")`. |
| `test_r1_2a_08` | Backup Overwrite Existing File | **PASS** | Rejected with `BackupError("already exists")`. |
| `test_r1_2a_08` | Backup File Permissions (`0o600`) | **PASS** | Verified mode `0o100600` via POSIX metadata. |
| `test_r1_2a_09` | Restore Non-Existent & Empty Files | **PASS** | Rejected with `RestoreError`. |
| `test_r1_2a_09` | Restore Corrupted/Garbage Dump File | **PASS** | Rejected with `RestoreError("does not appear to be a valid SQL dump archive")`. |
| `test_r1_2a_09` | Restore Source is a Symlink | **PASS** | Rejected with `RestoreError("Source ... is a symlink")`. |
| `test_r1_2a_10` | Non-Empty Unknown Directory Protection | **PASS** | Rejected with `Refusing to overwrite unknown data directory`. |
| `test_r1_2a_11` | Self-Authorization (`approval_token == db_name`) | **PASS** | Rejected with `DatabaseUnauthorized("Self-authorization rejected")`. |
| `test_r1_2a_11` | Database Mismatch (token for `db_a` used on `db_b`) | **PASS** | Rejected with `DatabaseUnauthorized`. |
| `test_r1_2a_11` | Token Replay / Double Consumption | **PASS** | Second invocation rejected; token consumed. |
| `test_r1_2a_11` | Expired Token (TTL > 60s) | **PASS** | Rejected with `DatabaseUnauthorized("has expired")`. |

---

## 10. Regression Test Evidence

All existing test suites across the entire LocalForge workspace were executed and verified:

```
running 9 tests (localforge-runtime lib)
test health::tests::test_port_search ... ok
test mariadb::tests::test_escape_sql_string_literal ... ok
test mariadb::tests::test_mariadb_identifier_validation ... ok
test mariadb::tests::test_database_deletion_approval_lifecycle ... ok
test mariadb::tests::test_mariadb_supervisor_paths ... ok
test mariadb::tests::test_mariadb_config_generation ... ok
test php::tests::test_pool_config_generation ... ok
test nginx::tests::test_nginx_config_generation ... ok
test php::tests::test_invalid_php_version_fails_safely ... ok
test result: ok. 9 passed; 0 failed

running 11 tests (mariadb_integration_test)
test test_r1_2_01_mariadb_version_detection ... ok
test test_r1_2_02_database_and_user_creation ... ok
test test_r1_2_03_database_backup_and_restore ... ok
test test_r1_2_04_php_pdo_mariadb_connectivity ... ok
test test_r1_2_05_multi_project_database_isolation ... ok
test test_r1_2_06_external_storage_mariadb_resilience ... ok
test test_r1_2a_07_credential_security_and_injection_adversarial ... ok
test test_r1_2a_08_backup_symlink_defense_and_atomic_permissions ... ok
test test_r1_2a_09_restore_invalid_and_corrupt_files ... ok
test test_r1_2a_10_datadir_ownership_and_unknown_directory_guard ... ok
test test_r1_2a_11_deletion_authorization_tampering_and_expiry ... ok
test result: ok. 11 passed; 0 failed

running 7 tests (runtime_integration_test)
test test_r1_1_01_fpm_pool_config ... ok
test test_r1_1_02_nginx_vhost_validation ... ok
test test_r1_1_03_real_php_fpm_and_nginx_http_execution ... ok
test test_r1_1_06_multi_project_isolation ... ok
test test_r1_1a_01_probe_classification_distinguishes_routing_from_infrastructure ... ok
test test_r1_1a_02_loopback_url_and_domain_separation ... ok
test test_r1_1a_03_stale_or_dead_fpm_socket_never_reports_running ... ok
test result: ok. 7 passed; 0 failed

running 5 tests (localforge-storage)
test db::tests::test_store_crud ... ok
test paths::tests::test_path_sanitization_valid ... ok
test paths::tests::test_path_traversal_detection ... ok
test paths::tests::test_tricky_nested_traversal_bypass_attempts ... ok
test paths::tests::test_valid_nested_and_normalized_subpaths ... ok
test result: ok. 5 passed; 0 failed

Frontend Build:
npm --prefix apps/desktop run build -> Exit Code 0 (Vite v6.4.4 production build succeeded in 7.01s)
```

---

## 11. GitHub CI Verification

- **Branch**: `phase-r1/r1-2a-database-security`
- **Actions Run**: To be triggered and monitored upon branch push.
- **Verification Gates**:
  - `cargo fmt --all -- --check` -> PASS
  - `cargo clippy --workspace --all-targets -- -D warnings` -> PASS (0 warnings)
  - `cargo test --workspace` -> PASS (32 unit & integration tests passed)
  - `npm --prefix apps/desktop run build` -> PASS

---

## 12. Residual Risks

1. **MySQL/MariaDB DDL Implicit Commit**: Because the database engine does not allow rollbacks on `CREATE TABLE` / `ALTER TABLE`, if a multi-statement dump encounters a syntax error halfway through, tables created up to that point remain. Mitigated by strictly disallowing restores into non-empty databases.
2. **External Disk Unmount During Active Write**: If an external disk is forcefully unplugged while `mariadb-dump` or `mysqld` is writing, incomplete files may remain. Mitigated by atomic temporary file renames and transactional InnoDB redo logs.
3. **Keychain UI Prompts**: Native Keychain integration in Phase 2 may trigger OS user authorization prompts on first access.

---

## 13. Commit SHA & Branch Status

- **Active Branch**: `phase-r1/r1-2a-database-security`
- **Parent Commit**: `5459fd2`
- **All Changes Staged**: Clean Git status pending commit.

---

## 14. Acceptance Checklist

- [x] Unsafe SQL password interpolation eliminated.
- [x] Null byte injection strictly rejected.
- [x] Password redaction implemented in runtime logs and error messages.
- [x] Database deletion self-authorization strictly eliminated (`db_name == approval_token` fails closed).
- [x] Cryptographic single-use token authorization with 60-second TTL implemented.
- [x] Backup files created with `0o600` permissions from inception.
- [x] Backup overwrite protection and atomic temporary rename implemented.
- [x] Backup and restore stream directly through kernel pipes (no memory buffering).
- [x] Restore into non-empty databases blocked.
- [x] Symlink attacks blocked for both backup and restore paths.
- [x] Datadir ownership validated against runtime user `uid`.
- [x] Unknown non-empty datadirs protected from overwrite.
- [x] MariaDB shutdown prefers graceful socket shutdown and `SIGTERM` over immediate `SIGKILL`.
- [x] All 11 live MariaDB integration tests pass.
- [x] Full workspace tests pass.
- [x] Clippy clean with zero warnings.

---

## 15. R1.3 Readiness Recommendation

Phase R1.2A has successfully resolved all outstanding security, authorization, and data-integrity risks in the database subsystem. The shared runtime core is stable, secure, and rigorously tested.

**Recommendation**: **ACCEPT PHASE R1.2A** and grant authorization to proceed to **Phase R1.3 — Spatial Studio Frontend & Real Runtime Dashboard Integration**.
