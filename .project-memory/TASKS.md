# LocalForge — Task Backlog

This is the canonical task backlog. Tasks have stable identifiers, priorities, explicit dependencies, and strict ownership rules.

> [!WARNING]
> Only the **Lead Agent** or **Auditor Agent** may update this file. Worker agents must report completion to the Lead Agent and not edit directly.

---

## Phase 0: Foundation & Project Memory (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-001** | Environment and Repository Audit | Critical | COMPLETED | DevOps / Systems Eng | None | Toolchain verification, OS & disk audit log |
| **TASK-002** | Architecture Research & Tech Stack Selection | Critical | COMPLETED | Principal Architect | TASK-001 | Tech stack analysis & version verification |
| **TASK-003** | Architecture Decision Records (ADR 1–8) | Critical | COMPLETED | Principal Architect | TASK-002 | `docs/architecture/adr/ADR-0001` through `0008` |
| **TASK-004** | Project Memory System Initialization | Critical | COMPLETED | Project Manager | TASK-003 | `.project-memory/*` (8 core files) |
| **TASK-005** | Product Documentation Suite | High | COMPLETED | Product Owner | TASK-002 | `docs/product/*` (Vision, Req, Matrix, Non-Goals) |
| **TASK-006** | Security Baseline & Threat Modeling | Critical | COMPLETED | Security Engineer | TASK-003 | `docs/security/*` (Threat Model, Security Policy) |
| **TASK-007** | Engineering Standards & Repository Setup | High | COMPLETED | DevOps / QA | None | `CONTRIBUTING.md`, `.gitignore`, `README.md` |
| **TASK-008** | Phase 0 & Phase 1 Specifications | High | COMPLETED | Lead Architect | TASK-003, TASK-005 | `PHASE_0_FOUNDATION.md`, `PHASE_1_DESKTOP_FOUNDATION.md` |
| **TASK-009** | Phase 0 Verification & Acceptance Audit | Critical | IN_PROGRESS | Auditor Agent | TASK-001..008 | Cross-reference checks, closure report, PO review |

---

## Phase 1: Desktop Foundation & Workspace Storage (ACTIVE - COMPLETED)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-101** | Tauri 2 & React + Vite Scaffolding | High | COMPLETED | Desktop Engineer | Phase 0 PO Sign-off | `apps/desktop/` with React 19 & Tailwind |
| **TASK-102** | Core Crates Setup in Cargo Workspace | High | COMPLETED | Senior Rust Eng | TASK-101 | `crates/core`, `crates/platform`, `crates/storage`, `crates/project-manager` |
| **TASK-103** | macOS APFS Volume Detection & Safety Guards | Critical | COMPLETED | Systems Engineer | TASK-102 | Diskutil / IOKit bindings & volume disconnect barrier |
| **TASK-104** | Project Manifest & SQLite Metadata Store | High | COMPLETED | Backend / Rust Eng | TASK-102 | `.localforge/project.yaml` parser & SQLite schema |
| **TASK-105** | Desktop UI Project List & Volume Health Status | Medium | COMPLETED | Frontend Engineer | TASK-101, TASK-104 | Project card list & drive status badge |
| **TASK-106** | Phase 1 Verification & Test Suite | Critical | COMPLETED | Auditor Agent | TASK-101..105 | Automated tests, test matrix update, handoff |

---

## Phase 2: Local Runtime Engine (ACTIVE - COMPLETED)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-201** | PHP-FPM Multi-Version Process Supervisor | Critical | COMPLETED | Senior Rust Eng | Phase 1 Acceptance | `crates/runtime` FPM lifecycle manager |
| **TASK-202** | Nginx Reverse Proxy & Loopback Virtual Hosts | High | COMPLETED | Systems Engineer | TASK-201 | Non-root Nginx template & configuration builder |
| **TASK-203** | MariaDB Lifecycle & Isolated User Databases | High | COMPLETED | Systems Engineer | TASK-201 | MariaDB process manager & data directory guards |
| **TASK-204** | Runtime Health Probes & Port Conflict Resolver | Medium | COMPLETED | Backend / Rust Eng | TASK-201..203 | TCP/HTTP health probes & conflict diagnostics |

---

## Phase 3: Smart Deployment & Interfaces (ACTIVE - COMPLETED)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-301** | Git-Aware Checksum Ledger & Diff Engine | Critical | COMPLETED | Senior Rust Eng | Phase 2 Acceptance | `crates/deployment` SHA-256 change tracker |
| **TASK-302** | SFTP Transfer Engine with Human Approval Gate | Critical | COMPLETED | Security / Rust Eng | TASK-301 | Resume-capable SFTP client & diff previewer |
| **TASK-303** | LocalForge Headless CLI Binary | Medium | COMPLETED | DevOps / Rust Eng | Phase 1-2 Crates | `tools/cli` with subcommands (`start`, `status`, etc.) |
| **TASK-304** | Model Context Protocol Server (Stdio) | High | COMPLETED | AI Systems Eng | Phase 1-2 Crates | `tools/mcp-server` for Claude Code / Codex |
