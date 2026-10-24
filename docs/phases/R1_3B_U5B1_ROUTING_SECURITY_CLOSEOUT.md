# LOCALFORGE — GATE U5.B1 CLOSEOUT REPORT
## Routing Security Hardening & Acceptance Closeout

**Gate:** U5.B1 — Routing Security Hardening & Acceptance Closeout  
**Branch:** `phase-r1/r1-3b-uat-recovery`  
**Base Commit:** `55a965b7d3c9bbce975795181aa3f51bd10fd2e1`  
**Status:** IMPLEMENTATION COMPLETE — 100% VERIFICATION PASSED  
**Author:** Antigravity (Lead Architect)  
**Date:** 2026-10-24  

---

## 1. Executive Summary & Authorization Compliance

In accordance with Product Owner directives for **Gate U5.B1**, all security findings, process identity vulnerabilities, port fallback hazards, Nginx virtual host collisions, and SQLite migration fault-tolerance requirements have been remediated and rigorously verified.

Work remained strictly confined to **isolated code and test environments**:
- **PO-U5-01 Compliance:** Zero mutation of the user's persistent SQLite database (`localforge.db`). `Projetest` remains strictly configured for PHP 8.4. Zero Homebrew packages installed or modified.
- **PO-U5-02 Compliance:** MariaDB datadir on the host remains strictly uninitialized and untouched.
- **PO-U5-03/04 Compliance:** The unsafe fallback to shared port 8080 has been completely eradicated across backend runtime IPC, metadata stores, routing engines, and UI layers.
- **PO-U5-05 Compliance:** LocalForge Branded Local Domains (`.localforge`, `.localhost`, `127.0.0.1:<port>`) are fully preserved with fail-closed security.

---

## 2. Root Cause Analysis & Architectural Remediations

### B1-01 (P0) — Nginx Process Identity Safety & TOCTOU Defenses

#### Root Cause:
Previously, `NginxSupervisor::stop()` invoked `nginx -s stop` using a PID read directly from `nginx.pid` without verifying:
1. Whether the executable running under that PID was genuinely an authentic Nginx binary.
2. Whether the command-line arguments contained LocalForge's specific configuration directory (`-c <conf_dir>/nginx.conf`).
3. Whether the process had been recycled by the operating system (PID reuse).
4. On macOS, `/var` is a symlink to `/private/var`, and Nginx mutates its process title at runtime, causing canonical path comparisons to fail when directories are deleted during test teardown.

#### Remediation:
- **Pre-Signal Verification Pipeline:** `NginxSupervisor::stop()` now executes a strict 7-step identity check BEFORE emitting any signal:
  1. Identifies candidate PID from in-memory lock or disk PID file, rejecting PIDs $\le 1$ or equal to self.
  2. Inspects target process via `MacOSProcessManager::inspect_process(pid)` using `proc_pidinfo`.
  3. Fails closed (`RuntimeError::ProcessMismatch`) if the binary is not an authentic canonical Nginx executable.
  4. Fails closed if the command line arguments do not reference LocalForge's exact configuration path.
  5. Re-verifies start time against in-memory state to protect against PID reuse.
  6. Executes `nginx -s stop` only on verified processes; then terminates the process group with verified grace.
  7. Safely reaps stale PID files if the process already exited (ESRCH) without raising errors.
- **macOS Path Normalization (`paths_match`):** Added `paths_match` handling `/private/var` vs `/var` symlink discrepancies transparently across all path verification checks.
- **Adversarial Test:** `test_adversarial_pid_file_replacement_and_unverified_process_rejection` proves that tampering with the PID file to point to foreign processes (e.g. `launchd`, fake mock binaries) is fail-closed rejected.

---

### B1-02 (P0) — Complete Elimination of Unsafe Port Fallback

#### Root Cause:
When a project had `loopback_port: None` or 0, previous layers silently fell back to port 8080 or 18081, creating a risk where an unassigned project could inadvertently open or collide with another project.

#### Remediation:
- **Rust Backend:**
  - `ProjectRoutingEngine::compute_profile`: When `allocated_port` is `None`, returns `GatewayPortStatus::Unassigned`, empty `loopback_endpoint`, empty `effective_url`, and `Scenario: Loopback Port Unassigned`.
  - `ProjectRuntimeService`: Reports `local_url` as empty string `""` when `loopback_port` is unassigned.
- **Frontend / UI (`apps/desktop`):**
  - `browserRouting.ts`: `computeProjectRouting` returns `canOpenBrowser: false`, `portStatus: "unassigned"`, `projectUrl: null`, `loopbackUrl: null`, `localhostUrl: null` whenever `loopback_port` is null, undefined, 0, or omitted. Only explicit `{ legacyMode: true }` enables 8080 fallback.
  - `App.tsx`: `handleOpenBrowser` verifies `canOpenBrowser`. If unassigned or out-of-range, browser launching is blocked with a clear Turkish notification.
  - `SpatialCard.tsx`: Displays `:${project.loopback_port}` or `"Atanmadı"` instead of `:${project.loopback_port || 8080}`.
  - `ProjectInspector.tsx`: "Tarayıcıda Aç ↗" button is strictly disabled with tooltip `"Port Tahsis Edilmedi"` when unassigned or stopped.
  - `RoutingAlertModal.tsx`: Displays explicit warning if loopback port is unassigned, disables direct browser opening, and hides loopback copy actions.
- **Port Range Enforcement:** Range `18081..=18999` is strictly validated. Any port outside this boundary is classified as `invalid_range` and rejected.

---

### B1-03 (P1) — Nginx Shared-Port Host Isolation & Duplicate Server Name Elimination

#### Root Cause:
When multiple project vhosts were generated on port 8080, both included generic `localhost` and `127.0.0.1` server names, causing Nginx warnings (`[warn] conflicting server name "localhost" on 127.0.0.1:8080, ignored`) and route collisions.

#### Remediation:
- **Dedicated Port Block:** Handles `<slug>.localforge`, `<slug>.localforge.localhost`, `<local_domain>`, `127.0.0.1`, and `localhost`. Includes a `default_server` returning HTTP 404 (`Host mismatch on dedicated project port`) for wrong host headers.
- **Shared Gateway Port Block (8080):** Matches **ONLY** project-specific domains (`<slug>.localforge`, `<slug>.localforge.localhost`, `<local_domain>`). Generic `127.0.0.1` and `localhost` are completely omitted from project vhosts on 8080.
- **Gateway Default Server:** Shared port 8080 default server returns HTTP 404 (`Host header required or project domain not found`).
- **Fail-Closed Validation:** `validate_configuration()` runs `nginx -t` and detects `conflicting server name` warnings in stderr, failing closed if any duplicate server name exists.

---

### B1-04 (P1) — SQLite V2 Migration Integrity & Fault Tolerance

#### Root Cause:
If migration `migrate_to_v2_loopback_ports` was interrupted after `ALTER TABLE projects ADD COLUMN loopback_port` but before index creation, re-running would fail with `duplicate column name: loopback_port`. Additionally, duplicate ports in pre-existing data were not validated before applying the unique index.

#### Remediation:
- Added `has_unique_index()` querying `PRAGMA index_list('projects')`.
- Migration checks both column and index presence:
  - If column exists but index is missing (interrupted migration), safely creates the unique index without re-adding the column.
  - If duplicates exist prior to indexing, migration fails closed (`LocalForgeError::DatabaseError`) with diagnostic details rather than corrupting metadata.
  - Fully idempotent: returns `applied: false` when both column and index are present.
- `insert_project` and `update_project_loopback_port` enforce `(18081..=18999).contains(&port)`.

---

## 3. Files Modified & Exact Security Invariants

| File | Subsystem | Remediations Applied |
|---|---|---|
| `crates/runtime/src/nginx.rs` | Gateway / Supervisor | Pre-signal identity check, cmdline verification, `paths_match` /private symlink normalization, `default_server` 404 blocks, duplicate server name removal. |
| `crates/runtime/src/error.rs` | Error types | Added `ProcessMismatch { expected: String, found: String }`. |
| `crates/runtime/src/routing.rs` | Routing Engine | Unassigned port status handling, zero 8080 fallback on missing loopback port. |
| `crates/runtime/src/service.rs` | Service IPC | Empty `local_url` on unassigned ports, deterministic reload pause. |
| `crates/storage/src/db.rs` | SQLite Storage | Interrupted migration recovery, unique index verification, duplicate fail-closed detection, port bounds enforcement (`18081..=18999`). |
| `crates/runtime/tests/runtime_integration_test.rs` | Integration Tests | Verified port isolation, host mismatch 404, bare IP 404, stopped 503, interrupted migration tests. |
| `apps/desktop/src/utils/browserRouting.ts` | Desktop Routing | `PortStatus`, `canOpenBrowser`, unassigned port rejection, range validation. |
| `apps/desktop/src/App.tsx` | Desktop UI | Guarded `handleOpenBrowser`, blocks unassigned/invalid port browser launching. |
| `apps/desktop/src/components/SpatialCard.tsx` | Desktop UI | Removed `loopback_port || 8080` fallback, displays "Atanmadı". |
| `apps/desktop/src/components/ProjectInspector.tsx` | Desktop UI | Displays "Tahsis Edilmedi", disables browser open button for unassigned/stopped projects. |
| `apps/desktop/src/components/RoutingAlertModal.tsx` | Desktop UI | Disables direct opening and copy when loopback port is unassigned. |
| `apps/desktop/test/r1_3b_u5b_routing.test.mjs` | Test Suite | Unit tests for unassigned port, invalid range, legacy mode fallback. |
| `apps/desktop/test/r1_3b_remediation.test.mjs` | Test Suite | Updated to verify unassigned port behavior alongside valid port routing. |
| `apps/desktop/test/r1_3b_uat_recovery.test.mjs` | Test Suite | Updated to verify loopback port domain routing. |

---

## 4. Verification & Test Evidence Matrix

### 4.1 Rust Test Suite (100% Pass)
```
localforge-storage:
  test paths::tests::test_path_traversal_detection ... ok
  test paths::tests::test_tricky_nested_traversal_bypass_attempts ... ok
  test paths::tests::test_valid_nested_and_normalized_subpaths ... ok
  test paths::tests::test_path_sanitization_valid ... ok
  test db::tests::test_store_crud ... ok
  test db::tests::test_interrupted_migration_recovery_and_duplicate_detection ... ok
  test db::tests::test_v2_migration_and_port_constraints ... ok
  Result: 7 passed; 0 failed

localforge-runtime (unit):
  test health::tests::test_port_search ... ok
  test mariadb::tests::test_application_support_directory_resolution ... ok
  test mariadb::tests::test_mariadb_identifier_validation ... ok
  test mariadb::tests::test_escape_sql_string_literal ... ok
  test mariadb::tests::test_mariadb_config_generation ... ok
  test mariadb::tests::test_mariadb_supervisor_paths ... ok
  test mariadb::tests::test_database_deletion_approval_lifecycle ... ok
  test php::tests::test_pool_config_generation ... ok
  test routing::tests::test_routing_scenarios ... ok
  test routing::tests::test_slugify ... ok
  test nginx::tests::test_nginx_config_generation ... ok
  test routing::tests::test_port_allocator_deterministic_and_unique ... ok
  test nginx::tests::test_adversarial_pid_file_replacement_and_unverified_process_rejection ... ok
  test nginx::tests::test_is_localforge_nginx_cmdline_exact_path_matching_and_substring_rejection ... ok
  test nginx::tests::test_nginx_ownership_verification_unrelated_process_port_conflict ... ok
  test php::tests::test_invalid_php_version_fails_safely ... ok
  Result: 16 passed; 0 failed

localforge-runtime (integration):
  test test_r1_1_01_fpm_pool_config ... ok
  test test_r1_1_02_nginx_vhost_validation ... ok
  test test_r1_1_03_real_php_fpm_and_nginx_http_execution ... ok
  test test_r1_1_06_multi_project_isolation ... ok
  test test_r1_1a_01_probe_classification_distinguishes_routing_from_infrastructure ... ok
  test test_r1_1a_02_loopback_url_and_domain_separation ... ok
  test test_r1_1a_03_stale_or_dead_fpm_socket_never_reports_running ... ok
  test test_r1_3a_v1_01_sync_all_projects_surfaces_failures_and_syncs_sqlite ... ok
  test test_r1_3b_live_domain_routing_acceptance_and_isolation ... ok
  test test_r1_3b_prep_01_nginx_ownership_verification_and_port_conflict ... ok
  test test_r1_3b_prep_02_persistent_metadata_safety_and_error_propagation ... ok
  test test_r1_3b_prep_03_adversarial_legitimate_nginx_exists_but_unrelated_process_owns_port ... ok
  test test_r1_3b_prep_04_adversarial_cmdline_spoofing_and_substring_matching_fails_closed ... ok
  test test_r1_3b_stage_b_nginx_default_server_blocks_cross_contamination_and_requires_host_header ... ok
  Result: 14 passed; 0 failed

localforge-platform: 5 passed; 0 failed
localforge-deployment: 5 passed; 0 failed
localforge-mcp-server: 3 passed; 0 failed
localforge-desktop: 2 passed; 0 failed
```

### 4.2 Desktop Frontend Test Suite & Build (100% Pass)
```
npm test --prefix apps/desktop:
  ✔ 32 tests passed; 0 failed

npm run build --prefix apps/desktop:
  ✓ built in 9.40s (0 errors, 0 warnings)
```

### 4.3 Static Code Quality & Formatting
- `cargo fmt --check`: Clean (0 violations).
- `cargo clippy --workspace -- -D warnings`: Clean (0 warnings).

---

## 5. Security & Verification Evidence

1. **Host Mismatch Isolation:** Foreign Host header sent to Project B dedicated port returns HTTP 404 (`Host mismatch on dedicated project port`).
2. **Bare IP on Shared Port:** Request to `127.0.0.1:8080` without matching Host returns HTTP 404 (`Host header required or project domain not found`).
3. **Stopped Project Isolation:** Direct request to a stopped project's dedicated port returns HTTP 503 (`LocalForge Gateway: Proje Başlatılmamış veya Durdurulmuş (Project Stopped)`).
4. **PID File Tampering:** Replacing `nginx.pid` with an unverified or foreign PID is rejected fail-closed without signaling.
5. **Port Allocation Bounded:** Allocation strictly bounded to `18081..=18999`. Out-of-range ports are rejected at DB and routing layers.

---

## 6. Remaining Limitations & Real-macOS Deployment Prerequisites

1. **Host SQLite Database (`localforge.db`):** The user's real database has NOT been migrated to V2. It will be safely migrated upon explicit authorization in Gate U5.C.
2. **Homebrew / PHP 8.4:** PHP 8.4 has not been installed on the host. `Projetest` continues to point to PHP 8.4 safely.
3. **System DNS / `/etc/hosts`:** System resolver has not been modified. All tests used simulated resolution or Host headers.
4. **MariaDB Datadir:** Datadir initialization remains held pending separate explicit authorization.

---

LOCALFORGE GATE U5.B1 — ROUTING SECURITY HARDENING COMPLETE. AWAITING PRODUCT OWNER ACCEPTANCE.
