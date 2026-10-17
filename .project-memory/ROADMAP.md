# LocalForge — Engineering Roadmap

This document outlines the sequential phases, primary objectives, dependencies, and completion criteria for the development of LocalForge.

---

## Phase Overview

```
+-----------------------------------------------------------------------------------+
| Phase 0: Foundation & Project Memory (ACTIVE)                                     |
| -> Architecture, ADRs, Project Memory, Threat Model, Standards, Phase 1 Plan      |
+-----------------------------------------------------------------------------------+
                                         |
                                         v (Human PO Approval Required)
+-----------------------------------------------------------------------------------+
| Phase 1: Desktop Foundation & Workspace Storage (PLANNED)                         |
| -> Tauri 2 App, Shared Core Crates, APFS External Volume Detection, SQLite Store  |
+-----------------------------------------------------------------------------------+
                                         |
                                         v (Acceptance Review)
+-----------------------------------------------------------------------------------+
| Phase 2: Local Runtime Engine (PLANNED)                                           |
| -> Isolated PHP-FPM Pools, Loopback Nginx Gateway, MariaDB, Health Probes         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v (Acceptance Review)
+-----------------------------------------------------------------------------------+
| Phase 3: Smart Deployment & CLI / MCP Integration (PLANNED)                       |
| -> Git-aware Checksum Ledger, SFTP Engine, Headless CLI, Stdio MCP Server         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v (Acceptance Review)
+-----------------------------------------------------------------------------------+
| Phase 4: Hardening, Packaging & Open-Source Release (PLANNED)                     |
| -> Security Audits, macOS Notarization & Code Signing, CI/CD, Community Launch    |
+-----------------------------------------------------------------------------------+
```

---

## Phase Details

### Phase 0: Foundation & Project Memory
- **Status**: **IN PROGRESS**
- **Objectives**:
  - Perform environment and toolchain audit on target host.
  - Formulate architecture decisions (ADR-0001 to ADR-0008).
  - Establish persistent, file-based project memory (`.project-memory/`).
  - Draft product specifications (Vision, Requirements, Feature Matrix, Non-Goals).
  - Draft security baseline (Threat Model, Security Policy).
  - Set engineering standards (CONTRIBUTING, .gitignore, Git branching & commits).
  - Define Phase 1 execution plan.
- **Completion Criteria**:
  - All Phase 0 documentation completed and verified.
  - Zero compiler errors (no code yet).
  - Final engineering report delivered to Product Owner in Turkish.
  - Formal human approval obtained to authorize Phase 1.

### Phase 1: Desktop Foundation & Workspace Storage
- **Status**: *PLANNED (LOCKED)*
- **Dependencies**: Phase 0 human approval.
- **Objectives**:
  - Initialize Tauri 2 desktop shell with React 19, Vite, and Tailwind CSS.
  - Scaffold modular Cargo workspace (`crates/core`, `crates/platform`, `crates/storage`, `crates/project-manager`).
  - Implement macOS APFS volume identification (DiskArbitration / IOKit) and persistent volume UUID mapping.
  - Implement external volume disconnect/reconnect detection with safety barriers.
  - Implement SQLite metadata repository in WAL mode on internal storage (`~/Library/Application Support/LocalForge/`).
  - Build UI project list and external disk health indicator.
- **Completion Criteria**:
  - Desktop app launches on Apple Silicon with <50MB RAM idle.
  - Disconnecting external drive safely sets project state to `DISCONNECTED` without crashing or writing to internal disk.

### Phase 2: Local Runtime Engine
- **Status**: *PLANNED (LOCKED)*
- **Dependencies**: Phase 1 completion and acceptance.
- **Objectives**:
  - Implement `crates/runtime` supervisory tree.
  - Support independent PHP-FPM versions (e.g. 8.1, 8.2, 8.3, 8.4) with project-specific socket binding.
  - Loopback Nginx reverse proxy routing requests by local domain (e.g., `http://myproject.test`) without sudo.
  - Manage user-level MariaDB instance with internal database storage.
  - Real-time health monitoring and automated port conflict diagnostics.
- **Completion Criteria**:
  - Two distinct projects can run different PHP versions concurrently.
  - Browser successfully loads local WordPress/Laravel site over HTTP loopback.

### Phase 3: Smart Deployment & Interfaces (CLI + MCP)
- **Status**: *PLANNED (LOCKED)*
- **Dependencies**: Phase 2 completion and acceptance.
- **Objectives**:
  - Build `crates/deployment` change detection comparing working tree against last-deployed checksum ledger.
  - Default-deny secret file filtering engine.
  - Implement SFTP transfer engine with resume and SHA-256 verification.
  - Build LocalForge CLI (`tools/cli`) reusing core crates.
  - Build LocalForge MCP Server (`tools/mcp-server`) with strictly scoped tool permissions.
- **Completion Criteria**:
  - Deployment preview shows exact file diff and triggers interactive approval dialog.
  - Claude Code / Codex can inspect project status and generate deployment preview via MCP without root privileges.

### Phase 4: Hardening & Distribution
- **Status**: **COMPLETED**
- **Dependencies**: Phase 3 completion and acceptance.
- **Objectives**:
  - Apple Developer ID code signing and notarization pipeline.
  - Comprehensive fuzzing of project manifest parser and path sanitizers.
  - Automated GitHub Actions release pipeline building universal/arm64 DMGs.
  - Open-source dual-license publication (Apache-2.0 / MIT).
