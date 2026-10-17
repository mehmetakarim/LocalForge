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

## Phase 2: Local Runtime Engine (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-201** | PHP-FPM Lifecycle & Multi-Version Manager | Critical | COMPLETED | Senior Rust Eng | `crates/runtime` FPM lifecycle supervisor |
| **TASK-202** | Nginx Reverse Proxy & Loopback Virtual Hosts | High | COMPLETED | Systems Engineer | Non-root Nginx config builder & loopback proxy |
| **TASK-203** | MariaDB Lifecycle & Database Manager | High | COMPLETED | Systems Engineer | MariaDB process manager & data directory guards |
| **TASK-204** | Runtime Health Probes & Port Conflict Resolver | Medium | COMPLETED | Backend / Rust Eng | HTTP/TCP health probes & conflict diagnostics |

---

## Phase 3: Smart Deployment & CLI / MCP Integration (COMPLETED)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-301** | Git-Aware Change Detection & Deployment Checksums | Critical | COMPLETED | Senior Rust Eng | `crates/deployment` SHA-256 change tracker |
| **TASK-302** | SFTP Transfer Engine with Atomic Remote Sync | Critical | COMPLETED | Security / Rust Eng | Resume-capable SFTP client & diff previewer |
| **TASK-303** | LocalForge CLI Binary (`tools/cli`) | Medium | COMPLETED | DevOps / Rust Eng | Headless terminal utility (`localforge`) |
| **TASK-304** | Model Context Protocol Server (`tools/mcp-server`) | High | COMPLETED | AI Systems Eng | JSON-RPC Stdio MCP server for Claude/Codex |

---

## Phase 4: Hardening, Packaging & Open-Source Release (ACTIVE - COMPLETED)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-401** | GitHub Actions CI/CD & Automated Release Pipelines | High | COMPLETED | DevOps / Release Eng | `.github/workflows/ci.yml`, `release.yml` |
| **TASK-402** | Security Hardening & Edge-Case Traversal Tests | Critical | COMPLETED | Security Engineer | `crates/storage/src/paths.rs` security suite |
| **TASK-403** | macOS DMG Packaging & Hardened Runtime Setup | High | COMPLETED | Desktop Engineer | `tauri.conf.json` with overlay titlebar & DMG |
| **TASK-404** | Open-Source Community Health Suite | Medium | COMPLETED | Product Owner | `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue & PR templates |
| **TASK-405** | Final Phase 4 Verification & Closure Audit | Critical | COMPLETED | Auditor Agent | 13/13 tests PASS, 0 clippy warnings, handoff |
