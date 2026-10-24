# LOCALFORGE — GATE U5.F CONTROLLED REAL MAC ROUTING ACTIVATION PROPOSAL
## Final Corrected Execution Proposal & Readiness Reconciliation

**Gate:** U5.F — Controlled Real Mac Routing Activation (Corrected Proposal)  
**Branch:** `phase-r1/r1-3b-uat-recovery`  
**Verified Baseline Commit:** `a2efcbb302c0ba30b2872b946ce7c4dcf43d6e79`  
**CI Baseline:** Run `40141502541` — SUCCESS (100% Green on macOS Apple Silicon)  
**Status:** PROPOSAL CORRECTED — EXECUTION ON HOLD — AWAITING EXPLICIT PRODUCT OWNER ACTIVATION AUTHORIZATION  
**Execution Operator:** Antigravity Lead Agent (UID 501 `mehmetakar`)  
**Date:** 2026-10-25  

---

## 1. Executive Summary & Product Owner Directives

Following the Product Owner Directive for **Gate U5.F Final Execution Proposal Correction**, this document provides the final, corrected, and binding execution proposal for activating project-specific loopback routing on the Product Owner's real Apple Silicon Mac.

### Mandatory Compliance Commitments:
1. **Execution Hold:** No execution is authorized until explicit Product Owner approval is given. No database writes, package installations, MariaDB initializations, or system DNS changes have been or will be executed before approval.
2. **PO-U5-01 Strict Compliance:** `Projetest` remains pinned to PHP 8.4. It will not be started or migrated to PHP 8.5.
3. **PO-U5-02 Strict Compliance:** MariaDB datadir (`/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb`) remains 100% uninitialized.
4. **Single Canonical Migration:** Elimination of manual SQL execution alternatives; strict use of `MetadataStore::migrate_to_v2_loopback_ports()`.
5. **Exact Process Safety:** Zero hardcoded PIDs; dynamic kernel-verified process signaling; zero interference with foreign daemons (PIDs 4010, 63508 on 18097/18098 preserved).

---

## 2. Final Exact Write Set

Upon receiving explicit execution authorization, file system modifications will be strictly limited to the following approved targets:

### 1. Persistent SQLite Database:
- **Path:** `/Users/mehmetakar/Library/Application Support/LocalForge/localforge.db`
- **Schema Modifications (via `MetadataStore::migrate_to_v2_loopback_ports()`):**
  - Table `projects`: Column `loopback_port INTEGER NULL` added.
  - Partial Unique Index: `CREATE UNIQUE INDEX idx_projects_loopback_port ON projects(loopback_port) WHERE loopback_port IS NOT NULL;`
  - Version Tracking: Row inserted into `schema_migrations` (`version = 2`, `description = 'Add loopback_port with unique index'`).
- **Data Updates (via `MetadataStore::update_project_loopback_port()`):**
  - Project `proj_e92e56cd0e0c4c36952804a35b3587a3` (`r1_acceptance_project`): `loopback_port = 18081` (PHP version `8.5` preserved).
  - Project `proj_db5472bb224c4fd0960ff69f793590a7` (`Projetest`): `loopback_port = 18082` (PHP version strictly `8.4` preserved).

### 2. Nginx VHost Configuration Files:
- **Directory:** `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/nginx/conf.d/`
- **Files Created/Updated:**
  - `proj_e92e56cd0e0c4c36952804a35b3587a3.conf`:
    - Listeners: `127.0.0.1:18081` (dedicated loopback) and `127.0.0.1:8080` (gateway).
    - Host aliases: `r1-acceptance-project.localforge`, `r1_acceptance_project.test`, `r1-acceptance-project.localforge.localhost`.
    - Health endpoint: `/__localforge_health` returning `X-LocalForge-Project-ID: proj_e92e56cd0e0c4c36952804a35b3587a3`.
    - Upstream: PHP-FPM 8.5 FastCGI socket.
  - `proj_db5472bb224c4fd0960ff69f793590a7.conf`:
    - Listeners: `127.0.0.1:18082` (dedicated loopback) and `127.0.0.1:8080` (gateway).
    - Host aliases: `projetest.localforge`, `projetest.test`, `projetest.localforge.localhost`.
    - Health endpoint: `/__localforge_health` returning `X-LocalForge-Project-ID: proj_db5472bb224c4fd0960ff69f793590a7`.
    - Upstream: Configured for PHP 8.4 socket (expected to return 503 / stopped until PHP 8.4 is provisioned).

### 3. Backup and Quarantine Artifacts:
- **Private Backup Directory:** `/Users/mehmetakar/Library/Application Support/LocalForge/backups/u5f_activation_<timestamp>/` (`chmod 700`)
- **Atomic Backup File:** `localforge_v1_atomic_backup.db` (`chmod 600`)
- **Checksum Manifest:** `manifest.sha256` (`chmod 600`)
- **Nginx Config Snapshot:** `nginx_conf_d/` (`chmod 700`)

---

## 3. Backup and Restore Preconditions (Finding 01)

The backup procedure enforces strict ownership and permission boundaries:

```bash
# 1. Resolve and verify canonical Application Support directory
APP_SUPPORT="/Users/mehmetakar/Library/Application Support/LocalForge"
[ -d "$APP_SUPPORT" ] || { echo "ERROR: Application Support directory missing"; exit 1; }

# 2. Verify operator ownership (UID 501)
DIR_UID=$(stat -f "%u" "$APP_SUPPORT")
[ "$DIR_UID" -eq 501 ] || { echo "ERROR: Ownership mismatch (expected UID 501, got $DIR_UID)"; exit 1; }

# 3. Create private, ownership-restricted backup directory
BACKUP_DIR="$APP_SUPPORT/backups/u5f_activation_$(date +%Y%m%d_%H%M%S)"
mkdir -m 700 -p "$BACKUP_DIR"

# 4. Take snapshot of existing Nginx configuration
mkdir -m 700 -p "$BACKUP_DIR/nginx_conf_d"
cp -R "$APP_SUPPORT/runtime/nginx/conf.d/"* "$BACKUP_DIR/nginx_conf_d/" 2>/dev/null || true

# 5. Execute SQLite Online Backup API (Shared Read Lock, WAL Checkpoint Flush)
sqlite3 -readonly "$APP_SUPPORT/localforge.db" \
  ".backup '$BACKUP_DIR/localforge_v1_atomic_backup.db'"
chmod 600 "$BACKUP_DIR/localforge_v1_atomic_backup.db"

# 6. Verify backup integrity
INTEGRITY=$(sqlite3 "$BACKUP_DIR/localforge_v1_atomic_backup.db" "PRAGMA integrity_check;")
[ "$INTEGRITY" = "ok" ] || { echo "ERROR: Backup integrity check failed"; exit 1; }

# 7. Verify exact project records
COUNT=$(sqlite3 "$BACKUP_DIR/localforge_v1_atomic_backup.db" "SELECT count(*) FROM projects;")
[ "$COUNT" -eq 2 ] || { echo "ERROR: Project record count mismatch (expected 2, got $COUNT)"; exit 1; }

# Verify Project A: r1_acceptance_project (PHP 8.5)
P1_VERIFY=$(sqlite3 "$BACKUP_DIR/localforge_v1_atomic_backup.db" \
  "SELECT count(*) FROM projects WHERE id='proj_e92e56cd0e0c4c36952804a35b3587a3' AND php_version='8.5';")
[ "$P1_VERIFY" -eq 1 ] || { echo "ERROR: Project A record verification failed"; exit 1; }

# Verify Project B: Projetest (PHP 8.4)
P2_VERIFY=$(sqlite3 "$BACKUP_DIR/localforge_v1_atomic_backup.db" \
  "SELECT count(*) FROM projects WHERE id='proj_db5472bb224c4fd0960ff69f793590a7' AND php_version='8.4';")
[ "$P2_VERIFY" -eq 1 ] || { echo "ERROR: Project B record verification failed"; exit 1; }

# 8. Verify Schema V1 status (loopback_port must NOT exist in V1 backup)
HAS_PORT=$(sqlite3 "$BACKUP_DIR/localforge_v1_atomic_backup.db" \
  "SELECT count(*) FROM pragma_table_info('projects') WHERE name='loopback_port';")
[ "$HAS_PORT" -eq 0 ] || { echo "ERROR: Backup already contains loopback_port (not clean V1)"; exit 1; }

# 9. Compute cryptographic SHA-256 and store manifest
shasum -a 256 "$BACKUP_DIR/localforge_v1_atomic_backup.db" > "$BACKUP_DIR/manifest.sha256"
chmod 600 "$BACKUP_DIR/manifest.sha256"
```

**Stop Condition:** If ANY of steps 1–9 fails, the activation immediately aborts. Zero modifications are made to `localforge.db`.

---

## 4. Single Canonical Migration Mechanism (Finding 02)

Manual SQL execution paths are completely excluded. The migration is executed strictly through the compiled Rust API:

```
                  ┌─────────────────────────────────────────────────────────┐
                  │ MetadataStore::migrate_to_v2_loopback_ports()           │
                  │   ├── Open Connection at canonical path                 │
                  │   ├── BEGIN IMMEDIATE TRANSACTION                       │
                  │   ├── Validate existing schema & index status           │
                  │   ├── ALTER TABLE projects ADD COLUMN loopback_port     │
                  │   ├── CREATE UNIQUE INDEX idx_projects_loopback_port    │
                  │   │     WHERE loopback_port IS NOT NULL                 │
                  │   ├── INSERT INTO schema_migrations (version 2)         │
                  │   └── COMMIT TRANSACTION                                │
                  └────────────────────────────┬────────────────────────────┘
                                               │
                                               ▼
                  ┌─────────────────────────────────────────────────────────┐
                  │ MetadataStore::update_project_loopback_port()           │
                  │   ├── Set loopback_port = 18081 for r1_acceptance_proj  │
                  │   └── Set loopback_port = 18082 for Projetest (PHP 8.4) │
                  └─────────────────────────────────────────────────────────┘
```

### Exact Executable / Application Command:
The migration and port assignment will be invoked via the compiled LocalForge storage execution runner:
```bash
/Volumes/Mac\ Harici\ Disk/VibeProject/LocalForge/target/debug/localforge storage migrate-v2 \
  --db-path "/Users/mehmetakar/Library/Application Support/LocalForge/localforge.db"
```
Or via the direct deterministic Rust integration runner:
```bash
cargo test -p localforge-storage --test storage_integration_test -- test_v2_migration_and_index_lifecycle --exact
```

### Data Integrity Guarantees:
- **Version Tracking:** Schema version 2 explicitly recorded in `schema_migrations`.
- **Exact Project IDs Preserved:**
  - `proj_e92e56cd0e0c4c36952804a35b3587a3`
  - `proj_db5472bb224c4fd0960ff69f793590a7`
- **Volume UUID Preserved:** APFS hardware UUID `7E26530E-5179-414C-81B5-6FBDB429FC65`.
- **PHP 8.4 Pinned:** `Projetest` `php_version` column remains strictly `'8.4'`.
- **Unique Partial Index:** `idx_projects_loopback_port` enforces that no two projects can ever share a port, while allowing NULL for unallocated projects.

---

## 5. Corrected Per-Project HTTP Acceptance Matrix (Findings 03 & 04)

All identity probes MUST supply the expected `Host` header to validate virtual host routing and identity isolation.

| Project | Port | Host Header | Probe Command | Expected Status | Identity Validation | Runtime Note |
|---|---|---|---|---|---|---|
| **`r1_acceptance_project`** | `18081` | `r1-acceptance-project.localforge` | `curl -sS -i -H "Host: r1-acceptance-project.localforge" http://127.0.0.1:18081/__localforge_health` | **HTTP 200 OK** | `X-LocalForge-Project-ID: proj_e92e56cd0e0c4c36952804a35b3587a3` | Verified active PHP 8.5 project |
| **`r1_acceptance_project`** | `18081` | `r1-acceptance-project.localforge` | `curl -sS -i -H "Host: r1-acceptance-project.localforge" http://127.0.0.1:18081/` | **HTTP 200 OK** | Dynamic JSON response from `index.php` | Live application response |
| **`Projetest`** | `18082` | `projetest.localforge` | `curl -sS -i -H "Host: projetest.localforge" http://127.0.0.1:18082/__localforge_health` | **HTTP 200 OK** | `X-LocalForge-Project-ID: proj_db5472bb224c4fd0960ff69f793590a7` | Confirms dedicated port & vhost binding |
| **`Projetest`** | `18082` | `projetest.localforge` | `curl -sS -i -H "Host: projetest.localforge" http://127.0.0.1:18082/` | **HTTP 503 Service Unavailable** / Stopped | FastCGI socket unmounted | **Honest stopped-project outcome. PHP 8.4 not yet installed.** |
| **Legacy Gateway** | `8080` | `r1-acceptance-project.localforge` | `curl -sS -i -H "Host: r1-acceptance-project.localforge" http://127.0.0.1:8080/` | **HTTP 200 OK** | Dynamic JSON response | Preserves 8080 backward compatibility |
| **Legacy Domain** | `8080` | `r1_acceptance_project.test` | `curl -sS -i -H "Host: r1_acceptance_project.test" http://127.0.0.1:8080/` | **HTTP 200 OK** | Dynamic JSON response | Preserves `.test` metadata routing |

### Strict Stop Conditions for HTTP Probes:
- If `/__localforge_health` returns any project ID other than the expected ID, fail immediately.
- If `/__localforge_health` returns HTTP 404, it is classified as **UNHEALTHY**; fail immediately.
- If `http://127.0.0.1:18082/` routes to `r1_acceptance_project` (cross-project routing), fail immediately.
- `Projetest` is **NOT** expected to return HTTP 200. Returning 503 or stopped status is the authorized, honest behavior.

---

## 6. Execution Ownership, Process Identity & Verified Signaling (Finding 05)

### 1. Single Authorized Operator:
- Execution operator: **Antigravity Lead Agent** running strictly under current user `mehmetakar` (UID 501, GID 20).
- No `sudo` or root privileges are permitted or required.

### 2. Pre-Execution Quiescence Verification:
- Verify no LocalForge CLI or UI instances are running:
  ```bash
  pgrep -il localforge || true
  ```
- Verify no active writers hold locks on SQLite database:
  ```bash
  lsof "$APP_SUPPORT/localforge.db" || true
  ```

### 3. Dynamic Process Identity & Verified Signaling:
- **No Hardcoded PIDs:** The process identity is resolved dynamically at execution time.
- **Verification Steps:**
  1. Read PID from `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/run/nginx.pid`.
  2. Inspect PID via kernel process table (`proc_pidpath`, `proc_pidinfo`):
     - Executable path must match `/opt/homebrew/bin/nginx`.
     - Process UID must be `501`.
     - Configuration file path must contain `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/nginx/nginx.conf`.
     - Process start timestamp (`start_time_secs`) must be recorded.
  3. Validate full Nginx configuration syntax:
     ```bash
     /opt/homebrew/bin/nginx -t \
       -c "/Users/mehmetakar/Library/Application Support/LocalForge/runtime/nginx/nginx.conf" \
       -p "/Users/mehmetakar/Library/Application Support/LocalForge/runtime/nginx/"
     ```
  4. Signal configuration reload:
     - Transmit `libc::kill(pid, SIGHUP)` via `signal_process_verified`.
     - Confirm signal return code is 0.

### 4. Post-Reload Listener Health Verification:
- Probe operating system socket table via `check_listener_health(18081)` and `check_listener_health(18082)`.
- Confirm both ports transition to `Ready` status within 5 seconds.
- Foreign processes on 18097/18098 (PIDs 4010, 63508) are left untouched.

---

## 7. Rollback Procedure & Recovery Ordering (Finding 05)

If any verification fails after activation begins:

```
[FAILURE DETECTED]
        │
        ├── STEP 1: RESTORE NGINX CONFIGURATION (Conf First)
        │     ├── Copy "$BACKUP_DIR/nginx_conf_d/*" back to "$APP_SUPPORT/runtime/nginx/conf.d/"
        │     ├── Validate syntax: nginx -t -c ... -p ...
        │     └── Signal SIGHUP to verified Nginx master PID
        │
        ├── STEP 2: NON-DESTRUCTIVE DATABASE QUARANTINE (No Blind Deletion)
        │     ├── QUARANTINE_DIR="$APP_SUPPORT/quarantine/quarantine_$(date +%Y%m%d_%H%M%S)"
        │     ├── mkdir -m 700 -p "$QUARANTINE_DIR"
        │     └── mv "$APP_SUPPORT/localforge.db"* "$QUARANTINE_DIR/"
        │
        ├── STEP 3: ATOMIC DATABASE RESTORATION
        │     ├── cp "$BACKUP_DIR/localforge_v1_atomic_backup.db" "$APP_SUPPORT/localforge.db"
        │     └── chmod 600 "$APP_SUPPORT/localforge.db"
        │
        ├── STEP 4: POST-ROLLBACK INTEGRITY CHECK
        │     ├── sqlite3 "$APP_SUPPORT/localforge.db" "PRAGMA integrity_check;" == "ok"
        │     ├── sqlite3 "$APP_SUPPORT/localforge.db" "SELECT count(*) FROM projects;" == 2
        │     └── Verify Projetest php_version == "8.4"
        │
        └── STEP 5: ROLLBACK REPORT & ESCALATION
              └── If restoration fails:
                    ├── Quarantine directory remains 100% intact
                    ├── LocalForge halts immediately (fail-closed)
                    └── Generate Emergency Incident Report in Turkish
```

---

## 8. Final Product Owner Authorization Checkpoint

| Checkpoint | Target | Precondition | Verification Gate |
|---|---|---|---|
| **CP-1** | Quiescence & Backup | No active writers | Backup integrity ok, SHA-256 recorded |
| **CP-2** | V2 Migration | Backup verified | `migrate_to_v2_loopback_ports()` exits ok |
| **CP-3** | Port Allocation | Ports 18081/18082 free | `update_project_loopback_port()` exits ok |
| **CP-4** | VHost & Syntax | VHost files written | `nginx -t` exits code 0 |
| **CP-5** | Dynamic SIGHUP | Dynamic PID verified | Verified signal delivery, listeners Ready |
| **CP-6** | Project Acceptance | Host headers provided | `18081` 200 OK, `18082` 503 honest stopped |

**MANDATORY HOLD:** Execution will NOT begin until the Product Owner explicitly confirms approval of this proposal.

---

LOCALFORGE GATE U5.F — FINAL EXECUTION PROPOSAL READY. AWAITING PRODUCT OWNER EXPLICIT ACTIVATION AUTHORIZATION.
