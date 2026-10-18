# LocalForge Tasks & Work Packages

This document mirrors and indexes the active engineering tasks across project phases. For the canonical, machine-readable backlog, see [`.project-memory/TASKS.md`](../.project-memory/TASKS.md).

---

## Task Management Rules
1. **Lead / Auditor Authority**: Only the Lead Agent or Auditor Agent may modify task states. Worker agents report results and must not modify this file directly.
2. **Phase Discipline**: Work must strictly remain within the currently authorized phase (**Recovery Phase R0**).

---

## Recovery Phase R0: Safety Stabilization & CI Repair (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R0-01** | Fail-Closed Deployment Engine | Critical | COMPLETED | Rust Security Auditor | `crates/deployment/src/sftp.rs` rejects non-dry-run with `DeploymentNotImplemented`; 0 fake transfers reported; ledger immutable |
| **TASK-R0-02** | MCP Human Approval Security Barrier | Critical | COMPLETED | Security Engineer | `tools/mcp-server/src/main.rs` eliminates `approval_token` exposure; rejects non-dry-run execute; ADR-0009 |
| **TASK-R0-03** | GitHub Actions CI & Clippy Repair | High | COMPLETED | DevOps / Release Eng | Fixed `clippy::useless_format` in `crates/runtime/src/php.rs`; updated `.github/workflows/ci.yml` |
| **TASK-R0-04** | Project Memory Suite Reconciliation | Critical | COMPLETED | Principal Architect | Reconciled `PROJECT_STATE.md`, `ROADMAP.md`, `TASKS.md`, `HANDOFF.md`, `brain.md`, `docs/tasks.md` |
| **TASK-R0-05** | Security & Safety Regression Test Suite | Critical | COMPLETED | QA Lead | 18/18 workspace tests PASS (deployment ledger immutability, MCP self-approval rejection, dry-run simulation) |

---

## Historical Phase Baseline (Post-Audit A1 Assessment)

### Phase 0: Foundation & Project Memory (COMPLETED)
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
| **TASK-009** | Phase 0 Verification & Acceptance Audit | Critical | COMPLETED | Auditor Agent | Link validation, criteria verification, handoff |

### Phase 1: Desktop Foundation & Workspace Storage (IMPLEMENTED - PENDING FORMAL REVIEW)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-101** | Tauri 2 & React + Vite Scaffolding | High | IMPLEMENTED | Desktop Engineer | `apps/desktop/` with React 19 & Tailwind |
| **TASK-102** | Core Crates Setup in Cargo Workspace | High | IMPLEMENTED | Senior Rust Eng | Root Cargo workspace & typed domain primitives |
| **TASK-103** | External Volume Detection & Safety Guards | Critical | IMPLEMENTED | Systems Engineer | Diskutil / IOKit bindings & volume disconnect barrier |
| **TASK-104** | Project Registration & SQLite Metadata Store | High | IMPLEMENTED | Backend / Rust Eng | `.localforge/project.yaml` parser & SQLite schema |
| **TASK-105** | Desktop UI Project Browser & Volume Health Indicator | Medium | IMPLEMENTED | Frontend Engineer | Project list view & disk health badge |
| **TASK-106** | Phase 1 Automated Test Suite & Acceptance Audit | Critical | REQUIRES REVIEW | Auditor Agent | Unit/integration tests & Phase 1 closure report |

### Phase 2: Local Runtime Engine (PARTIALLY IMPLEMENTED - INTEGRATION INCOMPLETE)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-201** | PHP-FPM Lifecycle & Multi-Version Manager | Critical | PARTIAL | Senior Rust Eng | `crates/runtime` FPM lifecycle supervisor |
| **TASK-202** | Nginx Reverse Proxy & Loopback Virtual Hosts | High | PARTIAL | Systems Engineer | Non-root Nginx config builder & loopback proxy |
| **TASK-203** | MariaDB Lifecycle & Database Manager | High | PARTIAL | Systems Engineer | MariaDB process manager & data directory guards |
| **TASK-204** | Runtime Health Probes & Port Conflict Resolver | Medium | IMPLEMENTED | Backend / Rust Eng | HTTP/TCP health probes & conflict diagnostics |

### Phase 3: Smart Deployment & CLI / MCP Integration (REMEDIATED TO FAIL-CLOSED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-301** | Git-Aware Change Detection & Deployment Checksums | Critical | COMPLETED | Senior Rust Eng | `crates/deployment` SHA-256 change tracker |
| **TASK-302** | SFTP Transfer Engine with Atomic Remote Sync | Critical | SIMULATION REMEDIATED | Security / Rust Eng | Simulated SFTP removed; fail-closed in R0 |
| **TASK-303** | LocalForge CLI Binary (`tools/cli`) | Medium | IMPLEMENTED | DevOps / Rust Eng | Headless terminal utility (`localforge`) |
| **TASK-304** | Model Context Protocol Server (`tools/mcp-server`) | High | HARDENED | AI Systems Eng | JSON-RPC Stdio MCP server for Claude/Codex |

### Phase 4: Hardening, Packaging & Open-Source Release (REMEDIATED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-401** | GitHub Actions CI/CD & Automated Release Pipelines | High | REMEDIATED | DevOps / Release Eng | Clippy warnings fixed; branch triggers expanded |
| **TASK-402** | Security Hardening & Edge-Case Traversal Tests | Critical | COMPLETED | Security Engineer | `crates/storage/src/paths.rs` security test suite |
| **TASK-403** | macOS DMG Packaging & Hardened Runtime Setup | High | UNVERIFIED | Desktop Engineer | `tauri.conf.json` with overlay titlebar, DMG config |
| **TASK-404** | Open-Source Community Health Suite | Medium | COMPLETED | Product Owner | `SECURITY.md`, `CODE_OF_CONDUCT.md`, templates |
| **TASK-405** | Final Phase 4 Verification & Closure Audit | Critical | AUDITED | Auditor Agent | Audit A1 identified deficiencies; R0 executed |
