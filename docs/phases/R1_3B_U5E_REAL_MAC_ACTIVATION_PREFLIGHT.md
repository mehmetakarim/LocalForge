# LOCALFORGE — GATE U5.E REAL MAC ACTIVATION RUNBOOK & PREFLIGHT REPORT
## Reconciled Activation Runbook, Disposable Drill Evidence & Controlled Migration Plan

**Gate:** U5.E — Real Mac Activation Preflight & Controlled Migration Plan (Reconciled)  
**Branch:** `phase-r1/r1-3b-uat-recovery`  
**Verified Code Baseline Commit:** `0954d49d799015eb6f8df7cf2d5bcf0e976bcecf`  
**Verified Documentation Commit:** `86cb93b2a0c49741e4da250495eb5a6e70908866`  
**CI Baseline:** Run `40123012479` (Code & Tests) & Run `40140575584` (Preflight & Plan) — SUCCESS (100% Green on macOS Apple Silicon)  
**Status:** RUNBOOK RECONCILED — READ-ONLY AUDIT & DISPOSABLE DRILL COMPLETE — AWAITING PRODUCT OWNER EXPLICIT ACTIVATION APPROVAL  
**Author:** Antigravity (Lead Architect)  
**Date:** 2026-10-25  

---

## 1. Executive Summary & Authorization Compliance

In accordance with the Product Owner Directive for **Gate U5.E Activation Plan Reconciliation**, this document reconciles the activation runbook to address all five core findings:
1. **SQLite Backup & Recovery Safety:** Canonical online backup API, elimination of unsafe raw WAL/SHM copying, non-destructive quarantining, and concrete disposable drill evidence.
2. **Nginx Activation Safety:** Dynamic process identity verification (no hardcoded PIDs), distinction between syntax validation and listener activation, and preservation of unrelated daemons.
3. **Canonical Domain Identity:** RFC 1123 DNS-safe slugification (`r1_acceptance_project` → `r1-acceptance-project.localforge`), preservation of legacy `.test` metadata, and honest disclosure of port requirements for unprivileged local domains.
4. **Honest Runtime Acceptance:** Proposed vs reserved port semantics, preservation of PHP 8.4 on `Projetest` with expected 503/stopped outcomes prior to runtime provisioning, and separation of DNS resolution from interactive browser validation.
5. **Strict Prohibitions Adherence:** Zero database mutations to `localforge.db`, zero port reservations, zero real Nginx reloads, zero package installations, zero MariaDB initializations, and zero DNS alterations.

---

## 2. Real macOS Environment Evidence Snapshot (Read-Only)

A non-destructive preflight audit of the host environment was performed on 2026-10-24/2026-10-25:

```
Host Architecture:         Apple Silicon M1 (arm64)
Kernel Version:            Darwin 27.0.0 (macOS 27.0)
User & Group:              mehmetakar (UID 501, GID 20 / staff)
External Volume Path:      /Volumes/Mac Harici Disk
External Volume UUID:      7E26530E-5179-414C-81B5-6FBDB429FC65 (APFS Formatted)
Application Support Dir:   /Users/mehmetakar/Library/Application Support/LocalForge
Persistent Database Path:  /Users/mehmetakar/Library/Application Support/LocalForge/localforge.db
Database Journal Mode:     WAL (Write-Ahead Logging)
Database WAL File:         /Users/mehmetakar/Library/Application Support/LocalForge/localforge.db-wal (8,272 bytes)
Database SHM File:         /Users/mehmetakar/Library/Application Support/LocalForge/localforge.db-shm (32,768 bytes)
Database Main File:        /Users/mehmetakar/Library/Application Support/LocalForge/localforge.db (12,288 bytes)
SQLite Integrity Check:    ok (PRAGMA integrity_check returned 'ok')
Current Schema Version:    Schema V1 (10 columns, nullable loopback_port absent)
```

### Active Project Records (Read-Only Inspection):

| Field | Project 1 (`r1_acceptance_project`) | Project 2 (`Projetest`) | Compliance Note |
|---|---|---|---|
| **ID** | `proj_e92e56cd0e0c4c36952804a35b3587a3` | `proj_db5472bb224c4fd0960ff69f793590a7` | Stable UUIDv4 |
| **Name** | `r1_acceptance_project` | `Projetest` | Preserved |
| **Path** | `/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/r1_acceptance_project` | `/Volumes/Mac Harici Disk/Projetest` | Verified on external SSD |
| **Volume UUID** | `7E26530E-5179-414C-81B5-6FBDB429FC65` | `7E26530E-5179-414C-81B5-6FBDB429FC65` | Exact match with APFS hardware UUID |
| **Document Root** | `public` | `public` | Verified directory structure |
| **PHP Version** | `8.5` | `8.4` | **Strictly Preserved per PO-U5-01** |
| **Local Domain** | `r1_acceptance_project.test` | `projetest.test` | Legacy domain preserved |
| **Status** | `degraded` | `stopped` | Actual persistent state |
| **Loopback Port** | `None` (Schema V1) | `None` (Schema V1) | Proposed: `18081` & `18082` |

### Runtime Binaries & Process Identity Preflight:

```
Nginx Binary:              /opt/homebrew/bin/nginx (v1.31.7)
Nginx Config Syntax:       ok (/Users/.../nginx.conf test is successful)
Active Gateway PID:        Dynamic (preflight observed master 39566 under mehmetakar)
Active Gateway Listener:   127.0.0.1:8080 (LISTEN)
Gateway Response Probe:    Without Host header: HTTP 404 (Gateway Host header required)
                           With 'Host: r1_acceptance_project.test': HTTP 200 (Active PHP-FPM execution)
                           With 'Host: projetest.test': HTTP 404 (Project stopped, vhost unmounted)
PHP-FPM Binary:            /opt/homebrew/sbin/php-fpm (v8.5.3)
Active PHP-FPM PID:        Dynamic (preflight observed master 59625 under mehmetakar)
PHP 8.4 Host Status:       NOT INSTALLED (/opt/homebrew/opt/php@8.4 is an alias symlink to 8.5.3)
MariaDB Binary:            /opt/homebrew/bin/mariadbd (v13.0.2)
MariaDB Datadir:           /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb
MariaDB Datadir State:     UNINITIALIZED (0 files, 64 bytes total; strictly preserved per PO-U5-02)
```

### Port Preflight (`18081..=18999`):
An inspection of `lsof -iTCP -sTCP:LISTEN -n -P` identified:
- **Port 18097:** Occupied by test Nginx instance (PID 63508). Preserved; not terminated.
- **Port 18098:** Occupied by test Nginx instance (PID 4010). Preserved; not terminated.
- **Ports 18081..18096, 18099..18999:** Completely **FREE** (916 available candidate ports).
- **Proposed Port Allocations:**
  - `r1_acceptance_project`: Proposed `18081`
  - `Projetest`: Proposed `18082`
  - *Constraint:* Ports are treated strictly as **proposed**, never reserved, and will be re-inspected via `check_listener_health(port)` at the exact moment of activation.

---

## 3. SQLite Backup, Quiescence, Non-Destructive Quarantine & Disposable Drill Evidence

### 1. Invalidation of Raw WAL/SHM File Copying:
In SQLite Write-Ahead Logging (`wal`) mode:
- Copying `localforge.db`, `localforge.db-wal`, and `localforge.db-shm` via filesystem `cp` is **strictly non-atomic**. If any uncommitted write is in flight or WAL pages are checkpointed asynchronously during the copy, the resulting backup can suffer from torn pages, corrupt SHM indices, or missing transaction frames.
- **Policy:** LocalForge **rejects** separate raw copies of active WAL/SHM files as a consistent backup strategy.

### 2. Canonical Consistent Backup: SQLite Online Backup API:
The canonical backup must be taken using SQLite's native online backup API (`sqlite3_backup_*` exposed via `.backup` in the CLI or programmatic API):
```bash
sqlite3 -readonly "$HOME/Library/Application Support/LocalForge/localforge.db" \
  ".backup '$BACKUP_DIR/localforge_v1_atomic_backup.db'"
```
- **Guarantees:** Acquires a consistent shared read lock, flushes uncheckpointed WAL frames into the backup file, and produces a single, self-contained, fully checkpointed SQLite database file without tearing.
- **Verification Criteria:**
  1. `PRAGMA integrity_check;` returns `ok`.
  2. `SELECT count(*) FROM projects;` returns exactly `2`.
  3. `SELECT count(*) FROM pragma_table_info('projects') WHERE name = 'loopback_port';` returns `0` (confirms Schema V1).
  4. SHA-256 checksum recorded in manifest.

### 3. Quiescent Restoration Window & Non-Destructive Quarantine:
Before any migration or restoration:
1. **Quiescence Verification:** Ensure no LocalForge processes or background workers are running (`pgrep -il localforge` must be empty).
2. **Non-Destructive Quarantine (Zero Blind Deletion):**
   - Active WAL/SHM files must **NEVER** be deleted blindly.
   - If a rollback is triggered, the active database set (`localforge.db`, `localforge.db-wal`, `localforge.db-shm`) is moved atomically to a timestamped quarantine directory:
     ```bash
     QUARANTINE_DIR="$APP_SUPPORT/quarantine/quarantine_$(date +%Y%m%d_%H%M%S)"
     mkdir -p "$QUARANTINE_DIR"
     mv "$APP_SUPPORT/localforge.db"* "$QUARANTINE_DIR/"
     ```
   - Only after successful quarantine is the verified atomic backup copied into place as `localforge.db`.
3. **Rollback Failure & Escalation Protocol:**
   - If the restore copy fails (e.g. disk write failure, permission denial), the quarantine directory remains intact and untouched. LocalForge will immediately halt, refuse to overwrite partial files, and log an emergency diagnostic report in Turkish alerting the human engineer.
4. **Honest POSIX Safety Boundary:**
   - LocalForge provides deterministic software-level atomicity. However, unconditional zero data loss cannot be promised in the event of catastrophic physical SSD disconnection, kernel panics, or sudden power loss during raw block writes.

### 4. Disposable Drill Evidence (Real Host Execution):
To mathematically prove the backup, migration, quarantine, and rollback lifecycle without touching the Product Owner's real database, a disposable drill was executed on a temporary V1 copy in `/tmp/localforge_disposable_drill/run_drill.sh`.

#### Actual Terminal Log from Disposable Drill:
```
=== STAGE 1: SEED DISPOSABLE LIVE DATABASE (WAL MODE) ===
total 88
drwxr-xr-x@ 5 mehmetakar  wheel    160 Oct 25 02:05 .
drwxr-xr-x@ 5 mehmetakar  wheel    160 Oct 25 02:05 ..
-rw-r--r--@ 1 mehmetakar  wheel  12288 Oct 25 02:05 localforge.db
-rw-r--r--@ 1 mehmetakar  wheel  32768 Oct 25 02:05 localforge.db-shm
-rw-r--r--@ 1 mehmetakar  wheel      0 Oct 25 02:05 localforge.db-wal

=== STAGE 2: CANONICAL CONSISTENT BACKUP VIA SQLITE ONLINE BACKUP API ===
Online backup completed: /tmp/localforge_disposable_drill/workspace/backup/localforge_v1_atomic_backup.db
-rw-r--r--@ 1 mehmetakar  wheel  12288 Oct 25 02:05 /tmp/localforge_disposable_drill/workspace/backup/localforge_v1_atomic_backup.db

=== STAGE 3: BACKUP VERIFICATION & RECORD VALIDATION ===
Integrity check: ok
Project record count: 2
Has loopback_port in V1 backup: 0 (expected 0)

=== STAGE 4: EXECUTE V2 SCHEMA MIGRATION & PORT ASSIGNMENT ===
Schema after V2 migration:
CREATE TABLE projects (
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
            , loopback_port INTEGER NULL);
CREATE UNIQUE INDEX idx_projects_loopback_port ON projects(loopback_port) WHERE loopback_port IS NOT NULL;

=== STAGE 5: VALIDATE INDEX INTEGRITY & ASSIGN PORTS ===
Index SQL: CREATE UNIQUE INDEX idx_projects_loopback_port ON projects(loopback_port) WHERE loopback_port IS NOT NULL
Migrated Projects in Live DB:
           id: proj_e92e56cd0e0c4c36952804a35b3587a3
         name: r1_acceptance_project
  php_version: 8.5
loopback_port: 18081

           id: proj_db5472bb224c4fd0960ff69f793590a7
         name: Projetest
  php_version: 8.4
loopback_port: 18082

=== STAGE 6: SIMULATE CONTROLLED ROLLBACK WITH NON-DESTRUCTIVE QUARANTINE ===
Quarantining live database state...
Quarantined files:
total 96
drwxr-xr-x@ 5 mehmetakar  wheel    160 Oct 25 02:05 .
drwxr-xr-x@ 3 mehmetakar  wheel     96 Oct 25 02:05 ..
-rw-r--r--@ 1 mehmetakar  wheel  16384 Oct 25 02:05 localforge.db
-rw-r--r--@ 1 mehmetakar  wheel  32768 Oct 25 02:05 localforge.db-shm
-rw-r--r--@ 1 mehmetakar  wheel      0 Oct 25 02:05 localforge.db-wal

Restoring from verified atomic backup...
total 24
drwxr-xr-x@ 3 mehmetakar  wheel     96 Oct 25 02:05 .
drwxr-xr-x@ 5 mehmetakar  wheel    160 Oct 25 02:05 ..
-rw-r--r--@ 1 mehmetakar  wheel  12288 Oct 25 02:05 localforge.db

=== STAGE 7: POST-RESTORATION VERIFICATION ===
Restored DB integrity: ok
Restored DB record count: 2
Restored DB has loopback_port: 0 (0 means successfully reverted to Schema V1)
Projetest PHP version: 8.4 (must be 8.4)

DISPOSABLE DRILL SUCCESSFUL: Complete cycle of seed -> online backup -> verify -> migrate -> assign -> quarantine -> restore -> re-verify completed with 100% precision.
```

---

## 4. Nginx Activation Safety & Verified Dynamic Process Signaling

### 1. Zero Hardcoded PIDs:
- Preflight PID numbers (such as 39566) are **strictly historical observation artifacts** and must **NEVER** be hardcoded.
- At activation time, the supervisor resolves the exact Nginx master process identity dynamically via `NginxSupervisor::get_running_identity()`.

### 2. Runtime Identity Verification Protocol:
Immediately before emitting any configuration reload signal:
1. Read PID from `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/run/nginx.pid`.
2. Inspect PID via Darwin kernel `proc_pidinfo` and `proc_pidpath`.
3. Verify:
   - Canonical executable path matches `/opt/homebrew/bin/nginx`.
   - Process UID matches current user (`501`).
   - Command-line arguments contain exact `-c` and `-p` paths.
   - Start timestamp (`start_time_secs`, `start_time_usecs`) matches recorded identity.

### 3. Syntax Validation vs Listener Activation Distinction:
- `nginx -t` tests only configuration file syntax and basic semantic rules. It **does NOT** guarantee that socket ports can be bound (e.g. if an external process grabs the port between validation and reload).
- Therefore, activation enforces a **two-phase verification**:
  - **Phase 1 (Syntax):** `nginx -t` passes with exit code 0.
  - **Phase 2 (Listener Verification):** After graceful `SIGHUP`, `NginxSupervisor::check_listener_health(port)` probes the operating system listener table to confirm that LocalForge Nginx master process has successfully bound `127.0.0.1:<allocated_port>`.

### 4. Preservation of Unrelated Processes:
- Unrelated daemons (including background test Nginx PIDs 4010, 63508 on 18097/18098) will **NOT** be signaled or terminated.
- Existing port 8080 gateway listener behavior is preserved.

---

## 5. Canonical Domain Identity & Deterministic Slugification

### 1. RFC 1123 DNS-Safe Slugification:
To comply with DNS hostname standards (which disallow underscores `_`):
- **`Projetest`** → `projetest.localforge`
- **`r1_acceptance_project`** → `r1-acceptance-project.localforge` (underscore converted to hyphen).

### 2. Legacy Domain Metadata Preservation:
Existing project records retain their `.test` domain metadata without modification:
- `r1_acceptance_project`: retains `local_domain: r1_acceptance_project.test`
- `Projetest`: retains `local_domain: projetest.test`

### 3. Portless HTTP Clarification:
- **DNS Resolution ≠ Portless Access:** Resolving `<slug>.localforge` via `/etc/hosts` or `/etc/resolver/localforge` maps the hostname to `127.0.0.1`. However, web browsers default to port 80 for HTTP and port 443 for HTTPS.
- Because LocalForge operates strictly unprivileged (non-root) on dedicated loopback ports (`18081..=18999`), navigating to `http://<slug>.localforge` without a port specification will **NOT** reach the dedicated project port unless a privileged port-forwarding proxy is configured.
- Therefore, the dedicated loopback port must always be specified when accessing unprivileged loopback listeners:
  - Canonical DNS-independent URL: `http://<slug>.localforge.localhost:<port>`
  - Direct technical fallback: `http://127.0.0.1:<port>`
  - Preferred branded domain with port: `http://<slug>.localforge:<port>`

---

## 6. Honest Runtime Acceptance & Expected HTTP Outcomes

| Project | Configured PHP | Proposed Port | Expected Browser Outcome (Pre-PHP 8.4 Provisioning) | Expected Browser Outcome (Post-PHP 8.4 Provisioning) |
|---|---|---|---|---|
| **`r1_acceptance_project`** | `8.5` | `18081` | **HTTP 200 OK** (Active PHP-FPM 8.5.3 execution, dynamic JSON payload) | **HTTP 200 OK** |
| **`Projetest`** | `8.4` | `18082` | **Stopped / HTTP 503 Service Unavailable** (Legitimate, honest failure; PHP 8.4 runtime binary missing on host) | **HTTP 200 OK** (Once PHP 8.4 keg-only is provisioned in Gate U5.C) |

### Key Safety Principles:
1. **No Fabricated Success:** `Projetest` is **NOT** expected to return HTTP 200 before PHP 8.4 is genuinely provisioned. Returning HTTP 503 or stopped status is the correct, honest, fail-closed behavior.
2. **Identity-Aware Health Separation:** Listener presence on port 18082 proves only that Nginx is listening. Proving project readiness requires probing `/__localforge_health` to verify `project_id: "proj_db5472bb224c4fd0960ff69f793590a7"`.
3. **Real-Browser Verification Honesty:** DNS resolution verified via `dscacheutil` proves only local resolver mapping. Real Safari and Chrome rendering will be documented as verified only after real interactive browser executions are conducted post-activation.

---

## 7. Reconciled Step-by-Step Activation Sequence (Post-Approval)

When Product Owner activation approval is explicitly granted, the execution must proceed strictly as follows:

```
[CHECKPOINT 1: QUIESCENCE & BACKUP]
  ├── Verify pgrep -il localforge is empty (all desktop writers stopped)
  ├── Execute online backup: sqlite3 -readonly localforge.db ".backup '$BACKUP_DIR/localforge_v1_atomic_backup.db'"
  ├── Verify backup: PRAGMA integrity_check == ok, count == 2, loopback_port count == 0
  └── Record backup SHA-256

[CHECKPOINT 2: V2 SCHEMA MIGRATION]
  ├── Execute migrate_to_v2_loopback_ports() inside exclusive transaction
  └── Run validate_loopback_port_index() -> confirm IndexValidationStatus::Valid

[CHECKPOINT 3: DYNAMIC PORT ALLOCATION]
  ├── Re-probe OS listening ports (18081..=18999) via check_listener_health()
  ├── Allocate proposed port 18081 to r1_acceptance_project
  ├── Allocate proposed port 18082 to Projetest (PHP 8.4 preserved)
  └── Update SQLite records atomically

[CHECKPOINT 4: NGINX MULTI-PORT VHOST GENERATION]
  ├── Generate r1_acceptance_project vhost (listeners: 18081 & 8080; slug: r1-acceptance-project.localforge; /__localforge_health)
  ├── Generate Projetest vhost (listeners: 18082 & 8080; slug: projetest.localforge; /__localforge_health)
  ├── Validate full configuration via nginx -t -> must exit 0
  ├── Dynamically resolve active LocalForge Nginx master PID via get_running_identity()
  └── Signal SIGHUP via signal_process_verified(dynamic_pid, ...)

[CHECKPOINT 5: TWO-PHASE LISTENER & IDENTITY VERIFICATION]
  ├── Verify listener health: check_listener_health(18081) == Ready
  ├── Verify listener health: check_listener_health(18082) == Ready
  ├── Probe http://127.0.0.1:18081/__localforge_health -> confirm project_id matches
  └── Confirm port 8080 legacy routing intact

[ROLLBACK ESCALATION TRIGGER]
  └── If ANY step fails:
        ├── Non-destructively move live DB files to quarantine/quarantine_<timestamp>/
        ├── Restore localforge_v1_atomic_backup.db to localforge.db
        ├── Restore pre-activation single-vhost conf.d
        ├── Signal SIGHUP to verified Nginx master PID
        └── Halt and report in Turkish with exact error diagnostics
```

---

LOCALFORGE GATE U5.E — ACTIVATION RUNBOOK RECONCILED. AWAITING PRODUCT OWNER EXPLICIT ACTIVATION APPROVAL.
