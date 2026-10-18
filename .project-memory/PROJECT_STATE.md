# LocalForge — Project State

This document tracks the canonical technical and operational state of the LocalForge project.

---

## 1. Executive Summary

- **Active Phase**: **Phase R1.2A — Database Security Remediation**
- **Phase Status**: COMPLETED / AWAITING PRODUCT OWNER ACCEPTANCE
- **Active Branch**: `phase-r1/r1-2a-database-security`
- **Target Remote**: `origin/phase-r1/r1-2a-database-security`
- **Previous Phase Status**: **Phase R1.2 — CONDITIONALLY ACCEPTED**
- **Approved Design Baseline**: **Spatial Studio Visual Direction & Dual-Mode Architecture (APPROVED by PO)**.
- **R1 Subphase Split**: R1.0 (Architecture Refinement - Done), R1.1 (PHP-FPM/Nginx - Done), R1.1A (Acceptance Remediation - Done), R1.2 (MariaDB/Resilience - Conditionally Accepted), R1.2A (Database Security Remediation - Completed), R1.3 (Spatial Studio UI - Locked).
- **Test Matrix Status**: 37/37 automated tests pass across 8 crates (including 11 MariaDB integration & adversarial security tests, 7 runtime integration tests, 5 storage tests, 9 runtime unit tests, and 5 core tests), zero Clippy warnings, clean Vite production build.

---

## 2. Canonical Subsystem Status

| Subsystem / Phase | Audit Classification | Current Status | Notes & Governance |
|---|---|---|---|
| **Phase 0: Foundation & Memory** | VERIFIED | COMPLETED | Specifications, memory structure, and ADRs established. |
| **Phase 1: Desktop & Workspace Storage** | PARTIAL | IMPLEMENTATION EXISTS | Workspace crates & Tauri 2 shell compile; physical disconnect verification staged in R1.2. |
| **Phase 2: Local Runtime Engine** | REAL IMPLEMENTATION | COMPLETED (R1.1/R1.2/R1.2A) | Live unprivileged Darwin kernel verified FPM, MariaDB & shared Nginx loopback gateway. E2E HTTP 200 PDO PASS. |
| **Phase 3: Deployment & Interfaces** | SIMULATED (SFTP) | REMEDIATED (Fail-Closed) | Non-dry-run SFTP rejected with `DeploymentNotImplemented`. MCP self-approval bypass eliminated. Real transport deferred. |
| **Phase 4: Hardening & Packaging** | PARTIAL (CI Failed) | REMEDIATED (CI Repaired) | GitHub Actions CI Run `39142354888` Green on macOS Apple Silicon. Clean Mac packaging verification pending. |
| **Recovery Phase R0** | CRITICAL FIXES | CONDITIONALLY ACCEPTED | Safety stabilization, CI repair, regression suite (18/18 tests pass). |
| **Recovery Phase R1.0** | ARCHITECTURE | ACCEPTED | Architecture refinement: Domain resolution, MariaDB storage, process identity, performance budgets. |
| **Phase R1.1: Runtime Integration** | REAL DAEMONS | ACCEPTED | Live PHP-FPM 8.5.3 child spawning, Nginx 1.31.7 gateway, multi-project isolation. |
| **Phase R1.1A: Targeted Remediation** | HARDENED RUNTIME | ACCEPTED | Eliminated false RUNNING, hardened process inspection & group safety, honest loopback URL reporting. |
| **Phase R1.2: MariaDB & Resilience** | ISOLATED DB & E2E | CONDITIONALLY ACCEPTED | Unprivileged MariaDB 13.0.2, authenticated SQL health probe, PDO E2E, 0600 backups, VolumeBarrier. |
| **Phase R1.2A: Database Security** | SECURITY HARDENING | COMPLETED | SQL credential escaping & redaction, single-use deletion authorization, 0600 atomic stream backups, non-empty restore guard, datadir ownership validation. |

---

## 3. Environment Verification Snapshot

- **Host Machine**: Apple Silicon Mac (arm64, M1, Darwin 27.0.0, macOS 27.0)
- **Rust Toolchain**: `rustc 1.94.0`, `cargo 1.94.0`, `rustup 1.29.0` (Active & Verified)
- **Node & Runtimes**: Node.js `v20.19.5`, `npm 11.7.0`, `bun 1.3.12` (Active & Verified)
- **Build Tools**: Apple Clang `21.0.0` (Xcode CLI Tools at `/Applications/Xcode.app/Contents/Developer`)
- **Package Manager**: Homebrew `7.0.7-49-g012f02a`
- **Storage Environment**: APFS External Volume at `/Volumes/Mac Harici Disk` (931 GiB capacity, 135 GiB available)

---

## 4. Phase R1.2A Security Achievements

1. **SQL Credential Safety**:
   - Eliminated unsafe string formatting in `create_project_database`.
   - Implemented `escape_sql_string_literal` with null byte rejection (`\0`), quote escaping (`''`), backslash escaping (`\\`), and length constraints.
   - Password scrubbed from stderr in error reports.
2. **Database Deletion Authorization**:
   - Self-authorization strictly eliminated: `approval_token == db_name` immediately returns `DatabaseUnauthorized`.
   - Single-use cryptographic UUIDv4 token with 60s TTL via `request_database_deletion_approval`.
3. **Secure Backup Creation**:
   - `0o600` permissions enforced atomically via `OpenOptionsExt::mode(0o600)` at creation time.
   - Refuses to overwrite existing files.
   - Destination path canonicalized and symlinks rejected.
   - Streaming stdout directly to file descriptor (O(1) memory).
   - Validates non-empty SQL header (`--`, `/*`, `CREATE`) and renames temporary file atomically.
4. **Safe Restore**:
   - Destination database checked for existing tables (`information_schema.tables == 0`); refuses restore into non-empty database.
   - Symlinks and empty/corrupt files rejected.
   - Streams dump file directly into client stdin.
5. **Lifecycle & Datadir Safety**:
   - Datadir ownership verified: `meta.uid() == libc::getuid()`.
   - Unknown existing non-empty directory protected from overwrite.
   - Graceful shutdown via `mysqladmin` and `SIGTERM` before fallback `SIGKILL`.

---

## 5. Security & Architectural Invariants

- **Fail-Closed Default**: Remote deployments remain strictly disabled (`DeploymentNotImplemented`).
- **AI Agent Boundary**: AI models via MCP cannot self-authorize production executions.
- **Process Safety**: Never signal a PID without verifying executable identity and launch timestamp.
- **Database Integrity**: MariaDB data stays on internal storage until external disconnect resilience is proven.
- **Non-Root Execution**: Development runtimes (PHP-FPM, Nginx, MariaDB) execute under user permissions on unprivileged ports.
