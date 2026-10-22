# LocalForge — Gate U5 Acceptance Matrix & Verification Scenarios
## Comprehensive Test Plan for Multi-Version PHP, MariaDB Initialization & Loopback Port Routing (Reconciled)

**Author:** Principal Software Architect & QA Lead  
**Governance:** Gate U5.A — Architecture Safety Reconciliation & Implementation Readiness  
**Status:** RECONCILED ACCEPTANCE MATRIX — IMPLEMENTATION ON HOLD (STRICT NO-GO)  
**Date:** 2026-10-22

---

## 1. Acceptance Criteria & Honest Verification Policy

In strict accordance with the LocalForge Honest Verification Policy (AGENTS.md Section 5) and Product Owner Directive Gate U5.A:
- **`DESIGN REVIEWED`**: The architectural design, security analysis, or runbook has been drafted and peer-reviewed, but does not represent executable code or verified runtime state.
- **`PREFLIGHT VERIFIED`**: Empirically measured and validated on the host machine using read-only non-destructive system tools.
- **`IMPLEMENTED & TESTED`**: Code exists in the repository, compiles, and passes automated unit/integration tests in the existing CI baseline.
- **`NOT RUN`**: Specified test scenario that has not been executed yet because implementation is on hold awaiting authorization.
- **`BLOCKED`**: Cannot be run because a prerequisite dependency or approval gate has not been satisfied.

> [!IMPORTANT]
> Documentation completion must never be classified as working runtime implementation or "PASS".

---

## 2. Gate U5 Verification Scenario Matrix

| Test ID | Test Category | Target Scenario & Invariant | Expected Behavior | Gate U5.A Status | Verification Method |
|---|---|---|---|---|---|
| **TEST-U5-01** | PHP Discovery | Read-only discovery of installed Homebrew PHP versions | Discovers `PHP 8.5.3 (fpm-fcgi arm64)`; marks `8.4` as missing | **PREFLIGHT VERIFIED** | Live telemetry audit via `PhpFpmSupervisor::discover_installed_details()` on macOS host |
| **TEST-U5-02** | PHP Preservation | Project `Projetest` retains PHP 8.4 assignment | `localforge.db` retains `php_version = "8.4"`; zero silent migration | **PREFLIGHT VERIFIED** | Verified SQLite record: `SELECT php_version FROM projects WHERE name='Projetest'` -> `8.4` |
| **TEST-U5-03** | PHP Missing Banner | UI displays Turkish missing runtime warning for Projetest | Displays amber warning box with recovery guidance and manual confirmed version switcher | **IMPLEMENTED & TESTED** | Tested in live Tauri app; captured in `tauri_project_inspector_missing_php.png` |
| **TEST-U5-04** | PHP Keg-Only Plan | Provisioning plan specifies keg-only `php@8.4` installation | Zero collision with global PHP 8.5.3; testable preconditions specified | **DESIGN REVIEWED** | Specified in `R1_3B_U5_RUNTIME_READINESS_REPORT.md` (Not executed) |
| **TEST-U5-05** | MariaDB Datadir Audit | Non-destructive audit of MariaDB Application Support storage | Directory is empty (64 bytes); `data/` not initialized; port 3306 is free | **PREFLIGHT VERIFIED** | Verified via `ls -la`, `df -h`, and `lsof -iTCP:3306` on host |
| **TEST-U5-06** | MariaDB Protection | Automatic datadir initialization is strictly refused | `start()` fails safely requiring PO authorization; zero disk corruption | **IMPLEMENTED & TESTED** | Automated test in `mariadb_integration_test.rs` & `crates/runtime/src/mariadb.rs` |
| **TEST-U5-07** | MariaDB Runbook | 12-point approval-gated runbook with staging & rollback | Specifies private staging, socket peercred auth, atomic promotion, and PO checkpoint | **DESIGN REVIEWED** | Specified in `R1_3B_U5_MARIADB_INITIALIZATION_RUNBOOK.md` (Not executed) |
| **TEST-U5-08** | Loopback Port Design | Deterministic port allocation architecture (`18081..18999`) | Dedicated loopback ports assigned per project; zero `/etc/hosts` DNS requirement | **DESIGN REVIEWED** | Specified in `R1_3B_U5_LOOPBACK_ROUTING_ARCHITECTURE.md` (Not implemented) |
| **TEST-U5-09** | Nginx Isolation | Shared Nginx multi-port master (`Option A`) | Port X maps strictly to Project X root and socket; fail-closed wrong-project routing | **DESIGN REVIEWED** | Evaluated and specified in `R1_3B_U5_LOOPBACK_ROUTING_ARCHITECTURE.md` |
| **TEST-U5-10** | Security Threat Model | Threat modeling covering STRIDE vectors and cookie scope | Evaluates host spoofing, port races, symlinks, PID reuse, RFC 6265 cookie scope | **DESIGN REVIEWED** | Documented in `R1_3B_U5_SECURITY_THREAT_MODEL.md` |
| **TEST-U5-11** | UX Prototypes | 4 high-fidelity Direction A+ visual prototypes | Auth gates, MariaDB approval, port endpoints, troubleshooting states | **DESIGN REVIEWED** | Authored under `docs/design/r1_3b/u5/` (Static review artifacts) |
| **TEST-U5-12** | Two Concurrent Projects | Live simultaneous execution of Project A (8.5) and Project B (8.4) | Separate ports (e.g. 18081 and 18082) serve their respective projects simultaneously | **NOT RUN** | Blocked awaiting Gate U5.B and U5.C approval |
| **TEST-U5-13** | Framework Redirects | WordPress / Laravel absolute redirects on loopback port | Applications generate correct URLs including port number (`port_in_redirect on`) | **NOT RUN** | Blocked awaiting Gate U5.B implementation |
| **TEST-U5-14** | External Disconnect | Physical disconnection of `/Volumes/Mac Harici Disk` | VolumeBarrier trips; Nginx refuses stale docroot; safe fail-closed state | **NOT RUN** | Blocked awaiting Gate U5.B implementation |

---

## 3. Existing CI Baseline vs Future Integration Suite

### 3.1 Existing CI Baseline (Commit `4e2d762`, Run `39848607260` — 100% PASS)
The following automated tests are active in the repository and pass on every commit:
- **Rust Core Workspace (58 tests)**:
  - `crates/core` (6 tests): Domain model validation, project manifests, sanitize_project_path.
  - `crates/platform` (8 tests): macOS volume detection, APFS parsing, process identity.
  - `crates/storage` (12 tests): SQLite migration, metadata queries, reconnect safety.
  - `crates/runtime` (21 tests): PHP telemetry discovery, Nginx command-line exact path matching, MariaDB lifecycle guards, process identity validation.
  - `crates/deployment` (11 tests): Cryptographic SHA-256 ledger, exclusion filter, fail-closed SFTP.
- **Frontend Test Suite (26 tests)**:
  - Storage capacity calculations, services telemetry formatting, error state HUD notifications, confirmed version switcher logic.

### 3.2 Future Integration Tests (To Be Implemented in Gates U5.B–D)
The following tests will be constructed when implementation is authorized:
1. `test_multi_port_nginx_config_generation`: Unit test validating multi-port `server` block synthesis.
2. `test_sqlite_loopback_port_migration`: Database migration test verifying `loopback_port` unique constraints.
3. `test_port_collision_fail_closed`: Simulates occupied port, verifies foreign PID diagnosis without termination.
4. `test_php84_keg_only_coexistence`: Verifies PHP 8.4 and PHP 8.5 FastCGI pools responding concurrently.
5. `test_mariadb_staging_atomic_promotion`: Tests staging directory creation, ownership check, and atomic rename.

---

## 4. What Remains Untested on the Actual Product Owner Mac

To maintain complete transparency with the Product Owner, the following runtime actions have **NOT been executed or tested on the user's physical machine**:
1. **Actual Homebrew installation of `php@8.4`**: `brew install php@8.4` has NOT been run.
2. **Spawning of a real PHP 8.4 FastCGI process**: No PHP 8.4 worker has run on the machine.
3. **Execution of MariaDB system table initialization**: `mariadb-install-db` has NOT been run; data directory remains 64 bytes empty.
4. **Binding of ports 18081/18082 in production Nginx**: Current Nginx still listens solely on port 8080.
5. **Live browser testing of WordPress or Laravel redirects on port 18081**: Untested on actual browser.
6. **Live physical disconnection of `/Volumes/Mac Harici Disk` under multi-port load**: Untested in live state.
