# LocalForge Brain (`brain.md`)

Welcome to the centralized project brain of **LocalForge**. This file serves as the top-level index connecting architecture, active tasks, session state, and multi-agent coordination rules.

---

## 1. Project Snapshot

| Attribute | Details |
|---|---|
| **Project Name** | LocalForge |
| **Repo** | `mehmetakarim/LocalForge` |
| **Active Branch** | `phase-0/foundation` (targeting `main`) |
| **Current Phase** | **Phase 0 — Foundation & Project Memory** |
| **Phase Status** | IN PROGRESS (Technical scaffolding & documentation) |
| **Next Phase** | Phase 1 — Desktop Foundation (PENDING PO APPROVAL) |
| **Primary Platform** | macOS Apple Silicon (`arm64`) |
| **Core Architecture** | Tauri 2 (Desktop UI) + Shared Rust Core + CLI + MCP Server |

---

## 2. Fast Navigation

- **Engineering & Collaboration Rules**: [`AGENTS.md`](AGENTS.md)
- **Claude Code Entry**: [`CLAUDE.md`](CLAUDE.md)
- **Architecture Overview**: [`docs/architecture.md`](docs/architecture.md)
- **Architecture Decisions (ADRs)**: [`docs/architecture/adr/`](docs/architecture/adr/)
- **Product Vision & Requirements**: [`docs/product/PRODUCT_VISION.md`](docs/product/PRODUCT_VISION.md)
- **Active Tasks & Backlog**: [`docs/tasks.md`](docs/tasks.md) & [`.project-memory/TASKS.md`](.project-memory/TASKS.md)
- **Session State & Handoff**: [`.project-memory/HANDOFF.md`](.project-memory/HANDOFF.md)
- **Threat Model & Security Policy**: [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md)

---

## 3. Core Architecture Pillars

1. **Modular Cargo Workspace (Shared Core)**:
   All business logic (project management, runtime lifecycle, deployment tracking, volume discovery) lives in shared Rust crates, reused across Desktop (Tauri 2), CLI, and MCP.
2. **First-Class External Storage**:
   External SSDs (e.g. `/Volumes/DeveloperSSD/...`) are handled natively with persistent UUID identification and disconnected-volume safety guards.
3. **Local Loopback & Least Privilege**:
   Services run unprivileged without `sudo` and bind exclusively to loopback (`127.0.0.1`) by default.
4. **Git-Aware Audited Deployment**:
   Independent deployment ledger with checksums, default-deny secrets, and human approval gates.
5. **AI-Ready Model Context Protocol**:
   Integrated MCP server sharing the core engine with strict capability boundaries.

---

## 4. Multi-Agent Memory & Modification Policy

> [!IMPORTANT]
> **Write Authority Restriction**:
> - Worker Agents **MUST NOT** directly edit `brain.md`, `docs/tasks.md`, `.project-memory/PROJECT_STATE.md`, or `.project-memory/TASKS.md`.
> - Only the **Lead Agent** or **Auditor Agent** updates these files following verified task completion.
> - For cross-session continuity, always update `.project-memory/HANDOFF.md` at session close.
