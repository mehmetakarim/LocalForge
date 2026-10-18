# LocalForge — Engineering Roadmap

This document outlines the sequential phases, primary objectives, dependencies, and completion criteria for the development of LocalForge.

---

## Phase Lifecycle & Current Status Overview

```
+-----------------------------------------------------------------------------------+
| Phase 0: Foundation & Project Memory (COMPLETED / REMEDIATED)                     |
| -> Architecture, ADRs, Threat Model, Engineering Standards                        |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 1: Desktop Foundation & Workspace Storage (IMPLEMENTATION EXISTS)           |
| -> Tauri 2 App, Core Crates, APFS External Volume Detection, SQLite Store         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 2: Local Runtime Engine (PARTIALLY IMPLEMENTED)                             |
| -> Isolated PHP-FPM Pools, Loopback Nginx Gateway, MariaDB, Health Probes         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 3: Smart Deployment & CLI / MCP Integration (REMEDIATED - FAIL-CLOSED)      |
| -> Git-aware Checksum Ledger, SFTP Engine (Fail-Closed in R0), CLI, MCP Server    |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 4: Hardening, Packaging & Release (REMEDIATED - CI REPAIRED)                |
| -> Security Hardening, GitHub Actions CI Repair, Packaging Baseline               |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+===================================================================================+
| >>> RECOVERY PHASE R0: Safety Stabilization & CI Repair (ACTIVE) <<<              |
| -> Fail-closed SFTP, MCP approval boundary, Clippy/CI repair, Memory sync        |
+===================================================================================+
                                         |
                                         v (Awaiting Product Owner Acceptance)
+-----------------------------------------------------------------------------------+
| Phase R1: Real SFTP Transport & Application-Mediated Deployment UI (PLANNED)      |
| -> Real SSH2/SFTP transfer, Native desktop approval modal, Clean Mac verification |
+-----------------------------------------------------------------------------------+
```

---

## Detailed Phase Statuses (Audit A1 Baseline)

### Phase 0: Foundation & Project Memory
- **Status**: **COMPLETED (REMEDIATED IN R0)**
- **Audit Findings**: Documentation and ADRs were established, but memory files had fallen out of sync with subsequent phase claims. Remediated during Recovery Phase R0.

### Phase 1: Desktop Foundation & Workspace Storage
- **Status**: **IMPLEMENTED (PENDING FORMAL ACCEPTANCE REVIEW)**
- **Audit Findings**: Modular Cargo workspace (`crates/core`, `crates/platform`, `crates/storage`, `crates/project-manager`) and Tauri 2 desktop shell compile and pass local tests. Real multi-drive disconnect edge cases require live hardware acceptance verification.

### Phase 2: Local Runtime Engine
- **Status**: **PARTIALLY IMPLEMENTED (INTEGRATION INCOMPLETE)**
- **Audit Findings**: PHP-FPM supervisor, Nginx template generator, and MariaDB supervisor modules exist with configuration generation unit tests. End-to-end multi-project concurrent routing requires live runtime integration testing.

### Phase 3: Smart Deployment & Interfaces (CLI + MCP)
- **Status**: **REMEDIATED (FAIL-CLOSED IN R0)**
- **Audit Findings**: Audit A1 discovered that SFTP transfer was simulated in memory while falsely reporting transferred files and mutating the on-disk ledger. MCP preview exposed approval tokens directly to AI models.
- **R0 Remediation**:
  - Non-dry-run deployments now fail closed with `LocalForgeError::DeploymentNotImplemented`.
  - Dry-run simulation reports zero transferred files and never touches the ledger.
  - MCP preview no longer exposes `approval_token`.
  - Real SFTP transport and application-mediated approval deferred to Phase R1.

### Phase 4: Hardening & Packaging
- **Status**: **REMEDIATED (CI REPAIRED IN R0)**
- **Audit Findings**: GitHub Actions CI failed in run `39108742495` due to `clippy::useless_format` in `crates/runtime/src/php.rs`. Clean Mac DMG installation and notarization remain unverified.
- **R0 Remediation**: Clippy warnings eliminated; `cargo fmt` check added; CI trigger expanded to recovery branches.

---

## Recovery Phase R0: Safety Stabilization & CI Repair (ACTIVE)

- **Status**: **ACTIVE / IN REMEDIATION**
- **Objectives**:
  1. Fail-closed deployment implementation (`crates/deployment/src/sftp.rs`).
  2. MCP server credential isolation & self-approval prevention (`tools/mcp-server/src/main.rs`).
  3. GitHub Actions CI clippy fix (`crates/runtime/src/php.rs` & `.github/workflows/ci.yml`).
  4. Memory reconciliation across all canonical tracking documents.
  5. Security regression test suite (18 passing tests across workspace).
