# LocalForge Brain (`brain.md`)

Welcome to the centralized project brain of **LocalForge**. This file serves as the top-level index connecting architecture, active tasks, session state, and multi-agent coordination rules.

---

## 1. Project Snapshot

| Attribute | Details |
|---|---|
| **Project Name** | LocalForge |
| **Repo** | `mehmetakarim/LocalForge` |
| **Active Branch** | `phase-r1/r1-0-architecture-refinement` |
| **Current Phase** | **Recovery Phase R1.0 — Architecture Refinement** |
| **Phase Status** | ACTIVE (Architecture refined, R1 split into R1.0-R1.3, awaiting R1.1 gateway approval) |
| **Design Baseline** | **Spatial Studio Visual Direction & Dual-Mode UI** (PO Approved, ADR-0010) |
| **Runtime Architecture** | Verified Process Identity, Shared Nginx, MariaDB Internal Isolation (ADR-0011, ADR-0012) |
| **Release State** | Held pending completion of Phase R1 subphases |
| **Primary Platform** | macOS Apple Silicon (`arm64`) |
| **Core Architecture** | Tauri 2 (Desktop UI) + Shared Rust Core + CLI + MCP Server |

---

## 2. Fast Navigation

- **Engineering & Collaboration Rules**: [`AGENTS.md`](AGENTS.md)
- **Claude Code Entry**: [`CLAUDE.md`](CLAUDE.md)
- **Architecture Overview**: [`docs/architecture.md`](docs/architecture.md)
- **Architecture Decisions (ADRs)**: [`docs/architecture/adr/`](docs/architecture/adr/)
- **Spatial Studio Design System**: [`docs/design/DESIGN_SYSTEM.md`](docs/design/DESIGN_SYSTEM.md)
- **Component Specifications**: [`docs/design/COMPONENT_SPECIFICATIONS.md`](docs/design/COMPONENT_SPECIFICATIONS.md)
- **Active Tasks & Backlog**: [`docs/tasks.md`](docs/tasks.md) & [`.project-memory/TASKS.md`](.project-memory/TASKS.md)
- **Project State & Roadmap**: [`.project-memory/PROJECT_STATE.md`](.project-memory/PROJECT_STATE.md) & [`.project-memory/ROADMAP.md`](.project-memory/ROADMAP.md)
- **Phase R1 Subphase Specs**: [`docs/phases/R1_0_ARCHITECTURE_REFINEMENT.md`](docs/phases/R1_0_ARCHITECTURE_REFINEMENT.md) to `R1_3`
- **Session State & Handoff**: [`.project-memory/HANDOFF.md`](.project-memory/HANDOFF.md)
- **Threat Model & Security Policy**: [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md)

---

## 3. Core Architecture Pillars

1. **Modular Cargo Workspace (Shared Core)**:
   All business logic (project management, runtime lifecycle, deployment tracking, volume discovery) lives in shared Rust crates, reused across Desktop (Tauri 2), CLI, and MCP.
2. **First-Class External Storage with Isolated Databases**:
   External APFS SSDs (e.g. `/Volumes/...`) host project sources; MariaDB databases default to internal SSD storage (`~/Library/Application Support/LocalForge/mariadb/`) for data resilience.
3. **Verified Process Identity & Least Privilege**:
   Services run unprivileged without `sudo`. Processes are signaled only after verifying executable path and start timestamps to prevent PID reuse kills.
4. **Git-Aware Audited Deployment**:
   Independent deployment ledger with checksums, default-deny secrets, fail-closed transport, and application-mediated human approval (ADR-0006, ADR-0009).
5. **AI-Ready Model Context Protocol**:
   Integrated MCP server sharing the core engine with strict credential isolation and human authorization barriers.

---

## 4. Multi-Agent Memory & Modification Policy

> [!IMPORTANT]
> **Write Authority Restriction**:
> - Worker Agents **MUST NOT** directly edit `brain.md`, `docs/tasks.md`, `.project-memory/PROJECT_STATE.md`, or `.project-memory/TASKS.md`.
> - Only the **Lead Agent** or **Auditor Agent** updates these files following verified task completion.
> - For cross-session continuity, always update `.project-memory/HANDOFF.md` at session close.
