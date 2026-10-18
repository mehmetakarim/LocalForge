# LocalForge — Engineering Roadmap

This document outlines the sequential phases, primary objectives, dependencies, and completion criteria for the development of LocalForge.

---

## Phase Lifecycle & Current Status Overview

```
+-----------------------------------------------------------------------------------+
| Phase 0: Foundation & Project Memory (COMPLETED)                                  |
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
| -> Configuration generators exist; active process supervision in Phase R1         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 3: Smart Deployment & Interfaces (REMEDIATED - FAIL-CLOSED)                 |
| -> Git-aware Checksum Ledger, Fail-Closed SFTP Engine, Hardened MCP Server        |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Phase 4: Hardening & Packaging (REMEDIATED - CI REPAIRED)                         |
| -> Clippy fix, CI Green, DMG Packaging Baseline                                   |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Recovery Phase R0: Safety Stabilization & CI Repair (CONDITIONALLY ACCEPTED)      |
| -> Fail-closed SFTP, MCP approval boundary, CI Green (Run 39123438754)            |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+===================================================================================+
| >>> RECOVERY PHASE R1: RUNTIME INTEGRATION & SPATIAL STUDIO SUITE <<<             |
+===================================================================================+
| [R1.0] Architecture Refinement (ACTIVE - IN REVIEW)                               |
|        Domain resolution, MariaDB storage policy, process identity, budgets       |
|                                         |                                         |
|                                         v (Awaiting PO Approval)                  |
| [R1.1] Real PHP-FPM and Nginx Integration (PLANNED)                               |
|        Unprivileged child spawn, FastCGI unix sockets, loopback HTTP 200 flow     |
|                                         |                                         |
|                                         v (Awaiting PO Approval)                  |
| [R1.2] MariaDB, Storage Resilience & Multi-Project Verification (PLANNED)         |
|        Internal database datadir, multi-project concurrency, disconnect safety    |
|                                         |                                         |
|                                         v (Awaiting PO Approval)                  |
| [R1.3] Spatial Studio UI Implementation (PLANNED)                                 |
|        Dual-mode Spatial Studio / Native Workspace, performance budgets           |
+===================================================================================+
```

---

## Detailed Recovery Phase R1 Breakdown

### R1.0: Architecture Refinement
- **Status**: **ACTIVE / IN REVIEW**
- **Focus**: Resolving technical ambiguities before writing runtime code:
  - Local domain resolution: HTTP Host-header loopback vs explicit `.test` resolver.
  - MariaDB storage: Default internal application storage isolation (`~/Library/Application Support/LocalForge/mariadb/`).
  - Process identity: Executable path and launch-ticks verification preventing PID reuse kills.
  - Shared Nginx vs independent PHP-FPM pool lifecycles.
  - Spatial Studio performance constraints (CPU <1%, RAM <80MB, max 2 blur layers, animation pause guards).
  - Four subphase specifications authored: R1.0, R1.1, R1.2, R1.3.

### R1.1: Real PHP-FPM and Nginx Integration
- **Status**: *PLANNED (LOCKED)*
- **Prerequisite**: Formal approval of R1.0.
- **Scope**: Live child process supervision for PHP-FPM and shared Nginx loopback gateway. FastCGI unix sockets, verified process identity, bounded 5s SIGTERM escalation, HTTP 200 loopback verification.

### R1.2: MariaDB, Storage Resilience & Multi-Project Verification
- **Status**: *PLANNED (LOCKED)*
- **Prerequisite**: Formal approval of R1.1.
- **Scope**: Supervise user-level MariaDB on internal SSD. Concurrently run Project A (PHP 8.4) and Project B (PHP 8.3). Test external APFS volume disconnect safety without internal storage pollution.

### R1.3: Spatial Studio UI Implementation
- **Status**: *PLANNED (LOCKED)*
- **Prerequisite**: Formal approval of R1.2.
- **Scope**: Implement approved Spatial Studio design system in `apps/desktop`. Deploy `SpatialCard`, `FloatingDock`, `AICommandCapsule`, `StorageTopologyCard`, and `ServiceHealthPill` under strict performance budgets.
