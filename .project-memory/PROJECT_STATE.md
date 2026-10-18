# LocalForge — Project State

This document tracks the canonical technical and operational state of the LocalForge project.

---

## 1. Executive Summary

- **Active Phase**: **Phase R1.1 — Real PHP-FPM & Nginx Runtime Integration**
- **Phase Status**: COMPLETED / AWAITING PRODUCT OWNER ACCEPTANCE
- **Active Branch**: `phase-r1/r1-1-runtime-implementation`
- **Target Remote**: `origin/phase-r1/r1-1-runtime-implementation` (base: `main`)
- **Previous Phase Status**: **Recovery Phase R1.0 — CONDITIONALLY ACCEPTED** by Product Owner.
- **Approved Design Baseline**: **Spatial Studio Visual Direction & Dual-Mode Architecture (APPROVED by PO)**.
- **R1 Subphase Split**: R1.0 (Architecture Refinement - Done), R1.1 (PHP-FPM/Nginx - Done), R1.2 (MariaDB/Resilience - Locked), R1.3 (Spatial Studio UI - Locked).
- **Test Matrix Status**: 25/25 automated tests pass, zero Clippy warnings, live HTTP 200 FastCGI verified on external APFS volume.

---

## 2. Canonical Subsystem Status

| Subsystem / Phase | Audit Classification | Current Status | Notes & Governance |
|---|---|---|---|
| **Phase 0: Foundation & Memory** | VERIFIED | COMPLETED | Specifications, memory structure, and ADRs established. |
| **Phase 1: Desktop & Workspace Storage** | PARTIAL | IMPLEMENTATION EXISTS | Workspace crates & Tauri 2 shell compile; physical disconnect verification staged in R1.2. |
| **Phase 2: Local Runtime Engine** | REAL IMPLEMENTATION | COMPLETED (R1.1) | Live unprivileged Darwin kernel verified FPM & shared Nginx loopback gateway. E2E HTTP 200 PASS. |
| **Phase 3: Deployment & Interfaces** | SIMULATED (SFTP) | REMEDIATED (Fail-Closed) | Non-dry-run SFTP rejected with `DeploymentNotImplemented`. MCP self-approval bypass eliminated. Real transport deferred. |
| **Phase 4: Hardening & Packaging** | PARTIAL (CI Failed) | REMEDIATED (CI Repaired) | GitHub Actions CI Run `39123438754` Green on macOS Apple Silicon. Clean Mac packaging verification pending. |
| **Recovery Phase R0** | CRITICAL FIXES | CONDITIONALLY ACCEPTED | Safety stabilization, CI repair, regression suite (18/18 tests pass). |
| **Recovery Phase R1.0** | ARCHITECTURE | ACCEPTED | Architecture refinement: Domain resolution, MariaDB storage, process identity, performance budgets. |
| **Phase R1.1: Runtime Integration** | REAL DAEMONS | COMPLETED | Live PHP-FPM 8.5.3 child spawning, Nginx 1.31.7 gateway, PID reuse defense, multi-project isolation. |

---

## 3. Environment Verification Snapshot

- **Host Machine**: Apple Silicon Mac (arm64, M1, Darwin 27.0.0, macOS 27.0)
- **Rust Toolchain**: `rustc 1.94.0`, `cargo 1.94.0`, `rustup 1.29.0` (Active & Verified)
- **Node & Runtimes**: Node.js `v20.19.5`, `npm 11.7.0`, `bun 1.3.12` (Active & Verified)
- **Build Tools**: Apple Clang `21.0.0` (Xcode CLI Tools at `/Applications/Xcode.app/Contents/Developer`)
- **Package Manager**: Homebrew `7.0.7-49-g012f02a`
- **Storage Environment**: APFS External Volume at `/Volumes/Mac Harici Disk` (931 GiB capacity, 135 GiB available)

---

## 4. Active Priorities (Phase R1.0 Architecture Refinement)

1. **Local Domain Resolution Clarity**:
   - Guarantee unprivileged loopback access via `127.0.0.1:8080` (or `localhost:8080`) with Nginx Host-header dispatch.
   - Decouple system-wide `.test` domain resolver configuration as a separately verified capability.
2. **MariaDB Storage Policy Revision**:
   - Mandate internal application data directory (`~/Library/Application Support/LocalForge/mariadb/`) as default.
   - Keep source code on external APFS drive while isolating database files on internal SSD.
3. **Verified Process Identity & Lifecycle Safety**:
   - Enforce executable path and start-timestamp checks before signaling PIDs to prevent PID reuse kills.
   - Establish shared Nginx gateway vs independent per-project PHP-FPM pools.
4. **Spatial Studio Performance & Hardware Budgets**:
   - Codify idle CPU (<1%), memory (<80MB), frame rate (60fps baseline, 120fps opportunistic), and blur budgets.
   - Implement continuous animation pause guards and pagination for >12 projects.
5. **R1 Subphase Governance & Planning**:
   - Establish explicit scopes, test matrices, and exit criteria for R1.0, R1.1, R1.2, and R1.3.

---

## 5. Security & Architectural Invariants

- **Fail-Closed Default**: Remote deployments remain strictly disabled (`DeploymentNotImplemented`).
- **AI Agent Boundary**: AI models via MCP cannot self-authorize production executions.
- **Process Safety**: Never signal a PID without verifying executable identity and launch timestamp.
- **Database Integrity**: MariaDB data stays on internal storage until external disconnect resilience is proven.
- **Non-Root Execution**: Development runtimes (PHP-FPM, Nginx, MariaDB) execute under user permissions on unprivileged ports.
