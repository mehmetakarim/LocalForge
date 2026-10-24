# LocalForge — Project State

This document tracks the canonical technical and operational state of the LocalForge project.

---

## 1. Executive Summary

- **Active Phase**: **Phase R1.3B UAT Failure Recovery — Gate U5.B1 Routing Security Hardening & Acceptance Closeout**
- **Phase Status**: GATE U5.B1 COMPLETED / ROUTING SECURITY HARDENING & ACCEPTANCE CLOSEOUT COMPLETE / AWAITING PRODUCT OWNER ACCEPTANCE
- **Active Branch**: `phase-r1/r1-3b-uat-recovery`
- **Previous Phase Status**: **Phase R1.3B UAT Failure Recovery (Gate U5.B Isolated Loopback Routing & Branded Domains - Commit 55a965b - CI Run 39917796258 Green)**
- **Approved Design Baseline**: **Direction A+ Spatial Studio Stage (1600×1000 Baseline)**.
- **R1 Subphase Split**: R1.0 (Architecture - Done), R1.1/R1.1A (PHP-FPM/Nginx - Done), R1.2/R1.2A/R1.2B (MariaDB/Resilience - Done), R1.3A-V Stage A (Visual Gate - Done), R1.3A-V Stage B (Spatial Visual Recovery - Done), R1.3A-V1 (Final Functional Acceptance Fixes & Patch - Accepted), R1.3B Preparation (Nginx Ownership, Metadata Error Propagation, SQLite Persistence - Completed), R1.3B Visual Design Gate (Task A Nginx Path Matching, Task B 4 High-Fidelity Prototypes, Task C Data Mapping - Completed), R1.3B Brand Integration Stage A (Approved), R1.3B Stage B (Brand Identity & Native Workspace Implementation - Completed), R1.3B Final Acceptance Remediation (Completed), R1.3B Final Acceptance Closeout (Completed), R1.3B UAT Recovery (Gates 2A, 2B, 2C, 3, 4 Completed), Gate U5 (Runtime Readiness & Loopback Routing Architecture - Completed), Gate U5.A (Architecture Safety Reconciliation - Completed), Gate U5.B (Isolated Loopback Routing Engine - Completed), Gate U5.B1 (Routing Security Hardening & Acceptance Closeout - Completed).
- **Test Matrix Status**: 52/52 automated Rust tests pass across workspace (16 runtime unit, 14 runtime integration, 7 storage, 5 platform, 5 deployment, 3 MCP, 2 desktop lib); 32/32 automated frontend tests pass (84 total); clean TypeScript & Vite v6.4.4 build; zero clippy warnings (`cargo clippy --workspace -- -D warnings`); zero formatting diffs (`cargo fmt --check`); Nginx process identity verified before signaling with TOCTOU defenses; unsafe 8080 port fallback eliminated across UI/IPC; multi-tenant port isolation verified with fail-closed HTTP 404/503; SQLite V2 migration fault tolerance verified; genuine native macOS Tauri app screenshots preserved in `docs/design/r1_3b/localforge_r1_3b_native_tauri_screenshots.zip`.

---

## 2. Canonical Subsystem Status

| Subsystem / Phase | Audit Classification | Current Status | Notes & Governance |
|---|---|---|---|
| **Phase 0: Foundation & Memory** | VERIFIED | COMPLETED | Specifications, memory structure, and ADRs established. |
| **Phase 1: Desktop & Workspace Storage** | PARTIAL | IMPLEMENTATION EXISTS | Workspace crates & Tauri 2 shell compile; physical disconnect verification staged in R1.2. |
| **Phase 2: Local Runtime Engine** | REAL IMPLEMENTATION | COMPLETED (R1.1/R1.2/R1.2B) | Live unprivileged Darwin kernel verified FPM, MariaDB & shared Nginx loopback gateway. E2E HTTP 200 PDO PASS. |
| **Phase 3: Deployment & Interfaces** | SIMULATED (SFTP) | REMEDIATED (Fail-Closed) | Non-dry-run SFTP rejected with `DeploymentNotImplemented`. MCP self-approval bypass eliminated. Real transport deferred. |
| **Phase 4: Hardening & Packaging** | PARTIAL (CI Failed) | REMEDIATED (CI Repaired) | GitHub Actions CI Run `39142354888` Green on macOS Apple Silicon. Clean Mac packaging verification pending. |
| **Recovery Phase R0** | CRITICAL FIXES | CONDITIONALLY ACCEPTED | Safety stabilization, CI repair, regression suite (18/18 tests pass). |
| **Recovery Phase R1.0** | ARCHITECTURE | ACCEPTED | Architecture refinement: Domain resolution, MariaDB storage, process identity, performance budgets. |
| **Phase R1.1: Runtime Integration** | REAL DAEMONS | ACCEPTED | Live PHP-FPM 8.5.3 child spawning, Nginx 1.31.7 gateway, multi-project isolation. |
| **Phase R1.1A: Targeted Remediation** | HARDENED RUNTIME | ACCEPTED | Eliminated false RUNNING, hardened process inspection & group safety, honest loopback URL reporting. |
| **Phase R1.2: MariaDB & Resilience** | ISOLATED DB & E2E | ACCEPTED | Unprivileged MariaDB 13.0.2, authenticated SQL health probe, PDO E2E, 0600 backups, VolumeBarrier. |
| **Phase R1.2A: Database Security** | SECURITY HARDENING | ACCEPTED | SQL credential escaping & redaction, single-use deletion authorization, 0600 atomic stream backups, non-empty restore guard, datadir ownership validation. |
| **Phase R1.2B: Final Database Safety** | FINAL SAFETY GATE | ACCEPTED | Deletion self-authorization eliminated, deletion kept explicitly disabled in R1.2B pending UI gate, fail-closed restore occupancy check, partial restore tracking, Darwin getpwuid directory resolution. |
| **Phase R1.3A: Spatial Studio Foundation** | DESIGN SYSTEM & SHELL | COMPLETED | Floating liquid dock, 2.5D SpatialCard micro-tilt, live infrastructure telemetry ribbon, AI Command Capsule (⌘K), embedded dist bundle. |

---

## 3. Environment Verification Snapshot

- **Host Machine**: Apple Silicon Mac (arm64, M1, Darwin 27.0.0, macOS 27.0)
- **Rust Toolchain**: `rustc 1.94.0`, `cargo 1.94.0`, `rustup 1.29.0` (Active & Verified)
- **Node & Runtimes**: Node.js `v20.19.5`, `npm 11.7.0`, `bun 1.3.12` (Active & Verified)
- **Build Tools**: Apple Clang `21.0.0` (Xcode CLI Tools at `/Applications/Xcode.app/Contents/Developer`)
- **Package Manager**: Homebrew `7.0.7-49-g012f02a`
- **Storage Environment**: APFS External Volume at `/Volumes/Mac Harici Disk` (931 GiB capacity, 135 GiB available)

---

## 4. Phase R1.2B Achievements

1. **Separation of Deletion Request and Approval**:
   - `request_database_deletion_approval` returns an audit `request_id`, not an authorization decision.
   - Self-authorization using `db_name` or `request_id` strictly fails closed.
   - Database deletion is explicitly disabled in Phase R1.2B with typed error `DatabaseDeletionUnavailable`, preventing destructive actions until Phase R1.3 Spatial Studio UI is integrated.
2. **Fail-Closed Database Restore**:
   - Eliminated `if let Ok(...)` occupancy check; any query or parse failure halts restore immediately.
   - Non-empty destinations are rejected.
   - Subprocess failures trigger diagnostic table count inspection, accurately recording partial restores and documenting DDL implicit commit limitations.
3. **Robust Darwin Application Support Resolution**:
   - Removed all hardcoded `/Users/mehmetakar` paths.
   - Implemented `resolve_application_support_dir()` querying `$HOME` and `libc::getpwuid(libc::getuid())`.
   - Propagated to runtime supervisor, desktop app, CLI, and MCP server.

---

## 5. Security & Architectural Invariants

- **Fail-Closed Default**: Remote deployments remain strictly disabled (`DeploymentNotImplemented`).
- **AI Agent Boundary**: AI models via MCP cannot self-authorize production executions or destructive database operations.
- **Process Safety**: Never signal a PID without verifying executable identity and launch timestamp.
- **Database Integrity**: MariaDB data stays on internal storage until external disconnect resilience is proven.
- **Non-Root Execution**: Development runtimes (PHP-FPM, Nginx, MariaDB) execute under user permissions on unprivileged ports.
