# LocalForge — Project State

This document tracks the canonical technical and operational state of the LocalForge project.

---

## 1. Executive Summary

- **Active Phase**: **Recovery Phase R0 — Safety Stabilization & CI Repair**
- **Phase Status**: IN PROGRESS (Stabilization & regression testing underway)
- **Active Branch**: `recovery/r0-safety-stabilization`
- **Target Branch**: `main`
- **Latest Commit Baseline**: Commit `a22b871` + R0 remediation
- **Audit Findings Baseline**: Independent Technical Audit A1 (`LOCALFORGE_INDEPENDENT_AUDIT_A1.md`)

---

## 2. Canonical Subsystem Status (Post-Audit A1 Reconciliation)

In accordance with the Honest Verification Policy (AGENTS.md Section 5), phase statuses have been reconciled from actual code and verified test evidence:

| Subsystem / Phase | Audit Classification | Current Status | Notes & Remediation |
|---|---|---|---|
| **Phase 0: Foundation & Memory** | VERIFIED | COMPLETED (Remediated) | Core specifications and ADRs established; memory contradictions reconciled in R0. |
| **Phase 1: Desktop & Workspace Storage** | PARTIAL | IMPLEMENTATION EXISTS | Workspace crates & Tauri 2 shell compiled; formal acceptance evidence requires review. |
| **Phase 2: Local Runtime Engine** | PARTIAL | INCOMPLETE INTEGRATION | Process managers & configuration builders exist; end-to-end multi-project orchestration unverified. |
| **Phase 3: Deployment & Interfaces** | SIMULATED (SFTP) | REMEDIATED (Fail-Closed) | In-memory deployment simulation was falsely writing to disk. Remediated to fail-closed (`DeploymentNotImplemented`). MCP self-approval token exposure eliminated. Real SFTP transport pending future phase. |
| **Phase 4: Hardening & Packaging** | PARTIAL (CI Failed) | REMEDIATED (CI Repaired) | GitHub Actions CI Clippy failure fixed in R0. Clean Mac DMG installation & notarization remains unverified. |

---

## 3. Environment Verification Snapshot

- **Host Machine**: Apple Silicon Mac (arm64, M1, Darwin 27.0.0, macOS 27.0)
- **Rust Toolchain**: `rustc 1.94.0`, `cargo 1.94.0`, `rustup 1.29.0` (Active & Verified)
- **Node & Runtimes**: Node.js `v20.19.5`, `npm 11.7.0`, `bun 1.3.12` (Active & Verified)
- **Build Tools**: Apple Clang `21.0.0` (Xcode CLI Tools at `/Applications/Xcode.app/Contents/Developer`)
- **Package Manager**: Homebrew `7.0.7-49-g012f02a`
- **Storage Environment**: APFS External Volume at `/Volumes/Mac Harici Disk` (931 GiB capacity, 135 GiB available)

---

## 4. Active Priorities (Recovery Phase R0)

1. **R0-01: Fail-Closed Deployment**:
   - Reject non-dry-run deployments with `LocalForgeError::DeploymentNotImplemented`.
   - Prevent fake file transfer reporting and false ledger state persistence.
   - Verified via unit & regression tests.
2. **R0-02: MCP Human Approval Security**:
   - Eliminate `approval_token` leakage in MCP preview responses.
   - Reject automated production deployment execution via MCP (`UNAUTHORIZED_MCP_DEPLOYMENT_EXECUTION`).
   - Document future application-mediated approval architecture in ADR-0009.
3. **R0-03: GitHub CI Repair**:
   - Fix `clippy::useless_format` warnings in `crates/runtime/src/php.rs`.
   - Add `cargo fmt` check to CI workflow and expand branch triggers to `recovery/*`.
4. **R0-04: Project Memory Reconciliation**:
   - Eliminate contradictions across memory suite; establish single source of truth.
5. **R0-05: Comprehensive Regression Testing**:
   - Validate all 18 unit/integration tests across workspace.

---

## 5. Security & Architectural Invariants

- **Fail-Closed Default**: No unverified transfer, unapproved token, or missing volume may proceed silently.
- **AI Agent Boundary**: AI agents (via MCP or automated tools) MUST NOT self-authorize remote production operations.
- **External Volume Safety**: Disconnected external volumes must never cause directory creation or state pollution on the internal disk.
- **Non-Root Execution**: Development runtimes (PHP-FPM, Nginx, MariaDB) must run under the user with unprivileged ports.
