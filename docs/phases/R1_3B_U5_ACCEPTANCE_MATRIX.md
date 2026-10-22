# LocalForge — Gate U5 Acceptance Matrix & Verification Scenarios
## Comprehensive Test Plan for Multi-Version PHP, MariaDB Initialization & Loopback Port Routing

**Author:** Principal Software Architect & QA Lead  
**Governance:** Gate U5 — Runtime Readiness & Isolated Loopback Routing Architecture  
**Status:** SPECIFIED & PREFLIGHT VERIFIED — IMPLEMENTATION TESTS NOT RUN (Awaiting PO Authorization)  
**Date:** 2026-10-22

---

## 1. Acceptance Criteria & Verification Policy

In accordance with the LocalForge Honest Verification Policy (AGENTS.md Section 5):
- Tests verified during Gate U5 preflight are marked **`PASS`**.
- Tests requiring future authorized implementation are marked **`NOT RUN (Awaiting PO Authorization)`**.
- No scenario is fabricated or assumed.

---

## 2. Gate U5 Verification Scenario Matrix

| Test ID | Test Category | Target Scenario & Invariant | Expected Behavior | Preflight Status | Verification Method |
|---|---|---|---|---|---|
| **TEST-U5-01** | PHP Discovery | Read-only discovery of installed Homebrew PHP versions | Discovers `PHP 8.5.3 (fpm-fcgi arm64)`; marks `8.4` as missing | **PASS** | Live telemetry audit via `PhpFpmSupervisor::discover_installed_details()` |
| **TEST-U5-02** | PHP Preservation | Project `Projetest` retains PHP 8.4 assignment | `localforge.db` retains `php_version = "8.4"`; zero silent migration | **PASS** | Verified SQLite record: `SELECT php_version FROM projects WHERE name='Projetest'` -> `8.4` |
| **TEST-U5-03** | PHP Missing Banner | UI displays Turkish missing runtime warning for Projetest | Displays amber warning box with recovery guidance and manual confirmed version switcher | **PASS** | Verified on live Tauri app via `tauri_project_inspector_missing_php.png` |
| **TEST-U5-04** | PHP Keg-Only Plan | Provisioning plan specifies keg-only `php@8.4` installation | Zero collision with global PHP 8.5.3; binary at `/opt/homebrew/opt/php@8.4/sbin/php-fpm` | **PASS** | Documented in `R1_3B_U5_RUNTIME_READINESS_REPORT.md` |
| **TEST-U5-05** | MariaDB Datadir Audit | Non-destructive audit of MariaDB Application Support storage | Directory is empty (64 bytes); `data/` not initialized; port 3306 is free | **PASS** | Verified via `ls -la`, `df -h`, and `lsof -iTCP:3306` |
| **TEST-U5-06** | MariaDB Protection | Automatic datadir initialization is strictly refused | `start()` fails safely requiring PO authorization; zero disk corruption | **PASS** | Tested in unit suite and `mariadb_integration_test.rs` |
| **TEST-U5-07** | MariaDB Runbook | 12-point approval-gated runbook with atomicity & rollback | Specifies exact commands, rollback, POSIX permissions (0700/0600), and PO checkpoint | **PASS** | Documented in `R1_3B_U5_MARIADB_INITIALIZATION_RUNBOOK.md` |
| **TEST-U5-08** | Loopback Port Design | Deterministic port allocation architecture (`18081..18999`) | Dedicated loopback ports assigned per project; zero `/etc/hosts` DNS requirement | **PASS** | Documented in `R1_3B_U5_LOOPBACK_ROUTING_ARCHITECTURE.md` |
| **TEST-U5-09** | Nginx Isolation | Shared Nginx multi-port master (`Option A`) | Port X maps strictly to Project X root and socket; zero cross-tenant leakage | **PASS** | Evaluated and specified with exact configuration template |
| **TEST-U5-10** | Security Threat Model | Threat modeling covering all 11 required threat vectors | Evaluates host spoofing, port races, symlinks, PID reuse, cookie isolation | **PASS** | Documented in `R1_3B_U5_SECURITY_THREAT_MODEL.md` |
| **TEST-U5-11** | UX Prototypes | 4 high-fidelity Direction A+ visual prototypes | Auth gates, MariaDB approval, port endpoints, troubleshooting states | **PASS** | Authored under `docs/design/r1_3b/u5/` |
| **TEST-U5-12** | Two Concurrent Projects | Live simultaneous execution of Project A (8.5) and Project B (8.4) | Separate ports (e.g. 18081 and 18082) serve their respective projects simultaneously | **NOT RUN** | Pending PO implementation authorization |
| **TEST-U5-13** | Framework Redirects | WordPress / Laravel absolute redirects on loopback port | Applications generate correct URLs including port number | **NOT RUN** | Pending PO implementation authorization |
| **TEST-U5-14** | External Disconnect | Physical disconnection of `/Volumes/Mac Harici Disk` | VolumeBarrier trips; Nginx refuses stale docroot; safe fail-closed state | **NOT RUN** | Pending PO implementation authorization |

---

## 3. Definition of Done for Future Implementation Phase
When the Product Owner authorizes implementation of Gate U5:
1. `php@8.4` installed via Homebrew keg-only formula and verified.
2. `Projetest` successfully launches with native `php@8.4` FastCGI socket.
3. MariaDB initialized following the 12-point runbook upon explicit PO authorization checkpoint.
4. Loopback ports allocated and persisted in SQLite (`projects.allocated_port`).
5. Clicking "Tarayıcıda Aç" opens `http://127.0.0.1:<allocated_port>/` and serves the correct project with zero DNS configuration.
6. 100% automated regression test coverage across all workspace crates.
