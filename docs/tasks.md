# LocalForge Tasks & Work Packages

This document mirrors and indexes the active engineering tasks across project phases. For the canonical, machine-readable backlog, see [`.project-memory/TASKS.md`](../.project-memory/TASKS.md).

---

## Task Management Rules
1. **Lead / Auditor Authority**: Only the Lead Agent or Auditor Agent may modify task states. Worker agents report results and must not modify this file directly.
2. **Phase Discipline**: Only tasks in the active phase (**Phase 0**) may be in progress. Phase 1+ tasks are strictly planned and locked.

---

## Phase 0: Foundation & Project Memory (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-001** | Environment and Repository Audit | Critical | COMPLETED | DevOps / Systems Eng | Audit log, CLI versions, hardware specs |
| **TASK-002** | Architecture Research & Tech Stack Selection | Critical | COMPLETED | Principal Architect | Tech stack validation, dependency versions |
| **TASK-003** | Architecture Decision Records (ADR 1–8) | Critical | COMPLETED | Principal Architect | `docs/architecture/adr/ADR-0001` to `0008` |
| **TASK-004** | Project Memory Initialization | Critical | COMPLETED | Project Manager | `.project-memory/*` canonical memory files |
| **TASK-005** | Product Documentation Suite | High | COMPLETED | Product Owner | Vision, Requirements, Feature Matrix, Non-Goals |
| **TASK-006** | Security Baseline & Threat Modeling | Critical | COMPLETED | Security Engineer | `THREAT_MODEL.md`, `SECURITY_POLICY.md` |
| **TASK-007** | Engineering Standards & Repository Setup | High | COMPLETED | DevOps / QA | `CONTRIBUTING.md`, `.gitignore`, `README.md` |
| **TASK-008** | Phase 0 & Phase 1 Specifications | High | COMPLETED | Lead Architect | `PHASE_0_FOUNDATION.md`, `PHASE_1_DESKTOP_FOUNDATION.md` |
| **TASK-009** | Phase 0 Verification & Acceptance Audit | Critical | IN PROGRESS | Auditor Agent | Link validation, criteria verification, handoff |

---

## Phase 1: Desktop Foundation & Workspace Storage (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-101** | Tauri 2 & React + Vite Scaffolding | High | COMPLETED | Desktop Engineer | `apps/desktop/` with React 19 & Tailwind |
| **TASK-102** | Core Crates Setup (`crates/core`, `crates/platform`, `crates/storage`, `crates/project-manager`) | High | COMPLETED | Senior Rust Eng | Root Cargo workspace & typed domain primitives |
| **TASK-103** | External Volume Detection & Safety Guards | Critical | COMPLETED | Systems Engineer | Diskutil / IOKit bindings & volume disconnect barrier |
| **TASK-104** | Project Registration & SQLite Metadata Store | High | COMPLETED | Backend / Rust Eng | `.localforge/project.yaml` parser & SQLite schema |
| **TASK-105** | Desktop UI Project Browser & Volume Health Indicator | Medium | COMPLETED | Frontend Engineer | Project list view & disk health badge |
| **TASK-106** | Phase 1 Automated Test Suite & Acceptance Audit | Critical | COMPLETED | Auditor Agent | Unit/integration tests & Phase 1 closure report |

---

## Future Phases (LOCKED)

### Phase 2: Local Runtime Engine (PLANNED)
- **TASK-201**: PHP-FPM Lifecycle & Multi-Version Manager
- **TASK-202**: Nginx Reverse Proxy & Loopback Virtual Hosts
- **TASK-203**: MariaDB Lifecycle & Database Manager
- **TASK-204**: Runtime Health Probes & Port Conflict Resolver

### Phase 3: Smart Deployment & CLI / MCP Integration (PLANNED)
- **TASK-301**: Git-Aware Change Detection & Deployment Checksums
- **TASK-302**: SFTP Transfer Engine with Atomic Remote Sync
- **TASK-303**: LocalForge CLI Binary (`tools/cli`)
- **TASK-304**: Model Context Protocol Server (`tools/mcp-server`)
