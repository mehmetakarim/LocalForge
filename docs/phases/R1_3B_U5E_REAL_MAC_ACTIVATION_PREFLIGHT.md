# LOCALFORGE — GATE U5.E REAL MAC ACTIVATION PREFLIGHT & MIGRATION PLAN
## Real-Host Preflight Audit, SQLite V2 Migration & Controlled Routing Activation

**Gate:** U5.E — Real Mac Activation Preflight & Controlled Migration Plan  
**Branch:** `phase-r1/r1-3b-uat-recovery`  
**Verified Code Baseline Commit:** `0954d49d799015eb6f8df7cf2d5bcf0e976bcecf`  
**Verified Documentation Commit:** `c24f81ef69c10bc6368d57813a07ce1e1a533421`  
**CI Baseline:** Run `40123012479` (Code & Tests) & Run `40123183411` (Docs & Memory) — SUCCESS (100% Green on macOS Apple Silicon)  
**Status:** PREFLIGHT COMPLETE — READ-ONLY AUDIT PERFORMED — AWAITING PRODUCT OWNER ACTIVATION APPROVAL  
**Author:** Antigravity (Lead Architect)  
**Date:** 2026-10-24  

---

## 1. Executive Summary & Authorization Compliance

In accordance with the Product Owner directives for **Gate U5.E**, this document presents a non-destructive, read-only preflight inspection of the Product Owner's real macOS Apple Silicon host environment, coupled with a controlled, fail-closed activation and migration plan for the LocalForge routing engine.

### Strict Boundary Adherence:
- **Zero Database Writes:** The real persistent database `localforge.db` was accessed exclusively with `-readonly` mode flags. Zero schema changes, zero row updates, and zero port allocations have been executed.
- **PHP 8.4 Preservation (Decision PO-U5-01):** Project `Projetest` remains strictly configured for PHP 8.4 in persistent metadata. Zero automatic migrations, zero package updates, and zero Homebrew mutations were executed.
- **MariaDB Hold (Decision PO-U5-02):** The host MariaDB datadir at `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb` was inspected and verified to be 100% uninitialized and untouched (0 files, 64 bytes).
- **Zero Host Network Mutations:** No modifications have been made to `/etc/hosts`, macOS resolvers, or firewall configurations.
- **Zero Process Termination:** No daemons or processes (including running test Nginx instances or MariaDB test instances) were terminated or reloaded during this preflight.

---

## 2. Real macOS Environment Evidence Snapshot

A comprehensive, non-destructive audit of the host environment was performed on 2026-10-24:

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
| **Loopback Port** | `None` (Schema V1) | `None` (Schema V1) | Unassigned pending migration |

### Runtime Binaries & Process Identity Preflight:

```
Nginx Binary:              /opt/homebrew/bin/nginx (v1.31.7)
Nginx Config Syntax:       ok (/Users/.../nginx.conf test is successful)
Active Gateway PID:        39566 (master process under mehmetakar) + 39567 (worker)
Active Gateway Listener:   127.0.0.1:8080 (LISTEN)
Gateway Response Probe:    Without Host header: HTTP 404 (Gateway Host header required)
                           With 'Host: r1_acceptance_project.test': HTTP 200 (Active PHP-FPM execution)
                           With 'Host: projetest.test': HTTP 404 (Project stopped, vhost unmounted)
PHP-FPM Binary:            /opt/homebrew/sbin/php-fpm (v8.5.3)
Active PHP-FPM PID:        59625 (master process under mehmetakar)
PHP 8.4 Host Status:       NOT INSTALLED (/opt/homebrew/opt/php@8.4 is an alias symlink to 8.5.3)
MariaDB Binary:            /opt/homebrew/bin/mariadbd (v13.0.2)
MariaDB Datadir:           /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb
MariaDB Datadir State:     UNINITIALIZED (0 files, 64 bytes total; strictly preserved per PO-U5-02)
```

### Port Preflight (`18081..=18999`):
An inspection of `lsof -iTCP -sTCP:LISTEN -n -P` identified:
- **Port 18097:** Occupied by transient test Nginx instance (PID 63508).
- **Port 18098:** Occupied by transient test Nginx instance (PID 4010).
- **Ports 18081..18096, 18099..18999:** Completely **FREE** (916 available candidate ports).
- **Port Allocator Guarantee:** LocalForge's `PortAllocator` proactively tests port availability before allocation and automatically bypasses occupied ports (18097, 18098), safely allocating from available slots (e.g., `18081` for Project 1, `18082` for Project 2).

### Native macOS DNS Resolution Audit:
```
Query 'projetest.localforge':               Unresolved (NXDOMAIN without local name resolver)
Query 'projetest.localforge.localhost':     Resolved to 127.0.0.1 and ::1 (Native RFC 6761 loopback)
Query 'projetest.test':                     Unresolved (Requires /etc/hosts or dnsmasq)
```
*Evidence:* As predicted in ADR-0010 and PO-U5-05, all `.localhost` subdomains natively resolve to loopback on macOS Apple Silicon without administrative rights or `/etc/hosts` changes.

---

## 3. SQLite Backup & Consistency Strategy (WAL/SHM Handling)

### The SQLite WAL Backup Challenge:
`localforge.db` operates in Write-Ahead Logging (`wal`) mode. Currently, uncheckpointed database transactions reside in `localforge.db-wal` (8,272 bytes). 
- **Danger of Raw File Copying:** A naive copy of `localforge.db` without `localforge.db-wal` results in data loss or corrupted pages.
- **Danger of Asynchronous Copying:** Copying files while the database is actively being read or written can produce a "torn page" backup.

### Canonical Recommended Procedure: Online Atomic Backup via SQLite API
SQLite provides a dedicated online backup and vacuum mechanism that guarantees transactional consistency while reading:

```bash
# 1. Define backup timestamp and directory
BACKUP_DIR="$HOME/Library/Application Support/LocalForge/backups/pre_u5_activation_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# 2. Perform atomic online backup using SQLite's .backup command
# This acquires a shared read lock, flushes all WAL pages into the destination,
# and outputs a single, self-contained, fully checkpointed SQLite database.
sqlite3 "$HOME/Library/Application Support/LocalForge/localforge.db" \
  ".backup '$BACKUP_DIR/localforge_v1_atomic.db'"

# 3. For defense-in-depth, capture raw file triple as secondary backup
cp "$HOME/Library/Application Support/LocalForge/localforge.db" "$BACKUP_DIR/localforge.db.raw"
cp "$HOME/Library/Application Support/LocalForge/localforge.db-wal" "$BACKUP_DIR/localforge.db-wal.raw"
cp "$HOME/Library/Application Support/LocalForge/localforge.db-shm" "$BACKUP_DIR/localforge.db-shm.raw"

# 4. Verification Drill of the Atomic Backup:
sqlite3 -readonly "$BACKUP_DIR/localforge_v1_atomic.db" "PRAGMA integrity_check;"
# MUST output: ok

sqlite3 -readonly "$BACKUP_DIR/localforge_v1_atomic.db" "SELECT count(*) FROM projects;"
# MUST output: 2
```

---

## 4. Controlled Future Migration & Activation Sequence

When the Product Owner explicitly authorizes activation, the procedure will execute strictly in the following sequence:

### Checkpoint 01 — Pre-Activation Lock & Verification
1. Verify no active instances of `localforge-desktop` are executing (`pgrep -il localforge`).
2. Execute the verified online atomic backup to `$BACKUP_DIR`.
3. Verify integrity of both backup and live database.

### Checkpoint 02 — Controlled Schema Migration V2
1. Open database connection using LocalForge `MetadataStore`.
2. Execute `migrate_to_v2_loopback_ports()`:
   ```sql
   ALTER TABLE projects ADD COLUMN loopback_port INTEGER NULL;
   CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_loopback_port 
       ON projects(loopback_port) 
       WHERE loopback_port IS NOT NULL;
   ```
3. Run `validate_loopback_port_index()`: Confirm index targets `loopback_port`, uniqueness is enforced, and partial predicate `WHERE loopback_port IS NOT NULL` is verified.

### Checkpoint 03 — Deterministic Port Allocation
1. Query available ports via `PortAllocator::find_available_port(18081..=18999)`:
   - Allocate `18081` to `r1_acceptance_project`.
   - Allocate `18082` to `Projetest`.
   - Verify neither port conflicts with existing PID 63508 (18097) or PID 4010 (18098).
2. Update SQLite records atomically:
   - `r1_acceptance_project`: `loopback_port = 18081`
   - `Projetest`: `loopback_port = 18082` (PHP version remains `8.4`).

### Checkpoint 04 — Nginx Multi-Port VHost Generation
1. Generate multi-port Nginx configuration for `r1_acceptance_project`:
   - Dedicated listener: `listen 127.0.0.1:18081;`
   - Shared gateway listener: `listen 127.0.0.1:8080;`
   - Server names: `r1_acceptance_project.localforge`, `r1-acceptance-project.localforge.localhost`, `r1_acceptance_project.test`
   - Health endpoint: `/__localforge_health` returning JSON `project_id: "proj_e92e56cd0e0c4c36952804a35b3587a3"` and `port: 18081`
2. Generate multi-port Nginx configuration for `Projetest`:
   - Dedicated listener: `listen 127.0.0.1:18082;`
   - Shared gateway listener: `listen 127.0.0.1:8080;`
   - Server names: `projetest.localforge`, `projetest.localforge.localhost`, `projetest.test`
   - Health endpoint: `/__localforge_health` returning JSON `project_id: "proj_db5472bb224c4fd0960ff69f793590a7"` and `port: 18082`
3. Execute `nginx -t` validation using `NginxSupervisor::validate_configuration()`.
4. If validation succeeds, issue graceful reload to verified master PID 39566 via `signal_process_verified(39566, expected_bin, start_secs, start_usecs, SIGHUP)`.

---

## 5. Real-Browser Acceptance Scenarios (Safari & Chrome)

Once activated, the routing behavior across browsers will operate according to the following deterministic matrix:

```
+--------------------------------------------------------------------------------------------------+
| ADDRESS / URI PATTERN                                  | SAFARI BEHAVIOR   | CHROME BEHAVIOR    |
+--------------------------------------------------------------------------------------------------+
| 1. Direct Loopback Fallback                            |                   |                    |
|    http://127.0.0.1:18081                              | Opens immediately | Opens immediately  |
|    (Dedicated project port; zero DNS dependency)       | HTTP 200 payload  | HTTP 200 payload   |
+--------------------------------------------------------------------------------------------------+
| 2. Branded RFC 6761 Candidate                          |                   |                    |
|    http://r1-acceptance-project.localforge.localhost:18081 | Resolves natively | Resolves natively  |
|    (Branded, port-isolated, zero /etc/hosts)           | HTTP 200 payload  | HTTP 200 payload   |
+--------------------------------------------------------------------------------------------------+
| 3. Preferred Branded Domain                            |                   |                    |
|    http://r1_acceptance_project.localforge            | Before DNS:       | Before DNS:        |
|    (Standard HTTP port 80 or local resolver)           | Server Not Found  | DNS_PROBE_FINISHED |
|                                                        | After DNS:        | After DNS:         |
|                                                        | HTTP 200 payload  | HTTP 200 payload   |
+--------------------------------------------------------------------------------------------------+
| 4. Legacy Domain Alias                                 |                   |                    |
|    http://r1_acceptance_project.test:8080              | Requires          | Requires           |
|    (Preserved for backward compatibility)              | /etc/hosts entry  | /etc/hosts entry   |
+--------------------------------------------------------------------------------------------------+
| 5. Zero Cross-Tenant Leakage Check                     |                   |                    |
|    http://127.0.0.1:18081 with                         | HTTP 404          | HTTP 404           |
|    'Host: foreign-project.test'                        | Host Mismatch     | Host Mismatch      |
+--------------------------------------------------------------------------------------------------+
```

---

## 6. Rollback Criteria & Procedures

If any unexpected condition occurs during activation, the following automated rollback procedure will restore the host to its exact preflight state:

1. **Rollback Trigger Conditions:**
   - Database migration fails or index validation reports `IndexValidationStatus::Incompatible`.
   - Nginx configuration validation (`nginx -t`) fails with syntax error or conflicting server names.
   - Six-layer browser readiness probe fails to verify listener ownership or health JSON matching.
2. **Rollback Execution Steps:**
   - **Step A (Database):** Restore `localforge.db` from `$BACKUP_DIR/localforge_v1_atomic.db`. Remove any leftover WAL/SHM files.
   - **Step B (Nginx):** Restore `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/nginx/conf.d` to its pre-activation single-vhost state (`proj_e92e56cd0e0c4c36952804a35b3587a3.conf`).
   - **Step C (Reload):** Issue graceful SIGHUP to Nginx master process 39566 via `signal_process_verified`.
   - **Step D (Verification):** Run `curl -H "Host: r1_acceptance_project.test" http://127.0.0.1:8080/` to verify restoration of initial HTTP 200 baseline.

---

## 7. Real macOS Activation Checklist (Post-Approval)

When Product Owner activation approval is granted:

- [ ] Execute atomic online SQLite backup and verify with `PRAGMA integrity_check`.
- [ ] Run `migrate_to_v2_loopback_ports()` in `localforge-storage`.
- [ ] Verify `validate_loopback_port_index()` passes with `IndexValidationStatus::Valid`.
- [ ] Assign dedicated port `18081` to `r1_acceptance_project` and `18082` to `Projetest`.
- [ ] Verify `Projetest` retains `php_version: '8.4'`.
- [ ] Generate dual-port vhost files in `runtime/nginx/conf.d/`.
- [ ] Verify `nginx -t` passes with exit code 0.
- [ ] Signal Nginx master process 39566 to reload configuration without dropping connections.
- [ ] Probe `http://127.0.0.1:18081/__localforge_health` and confirm JSON `project_id`.
- [ ] Test real macOS browser launch with Safari and Chrome via `verify_browser_readiness`.
- [ ] Confirm MariaDB datadir remains uninitialized.

---

LOCALFORGE GATE U5.E — PREFLIGHT COMPLETE. AWAITING PRODUCT OWNER ACTIVATION APPROVAL.
