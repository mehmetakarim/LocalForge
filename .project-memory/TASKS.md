# LocalForge — Task Backlog

This is the canonical task backlog. Tasks have stable identifiers, priorities, explicit dependencies, and strict ownership rules.

> [!WARNING]
> Only the **Lead Agent** or **Auditor Agent** may update this file. Worker agents must report completion to the Lead Agent and not edit directly.

---

## Recovery Phase R0: Safety Stabilization & CI Repair (ACTIVE)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R0-01** | Fail-Closed Deployment Engine | Critical | COMPLETED | Rust Security Auditor | Audit A1 | `crates/deployment/src/sftp.rs` rejects non-dry-run with `DeploymentNotImplemented`; 0 fake transfers reported; ledger immutable |
| **TASK-R0-02** | MCP Human Approval Security Barrier | Critical | COMPLETED | Security Engineer | Audit A1 | `tools/mcp-server/src/main.rs` eliminates `approval_token` exposure; rejects non-dry-run execute; ADR-0009 |
| **TASK-R0-03** | GitHub Actions CI & Clippy Repair | High | COMPLETED | DevOps / Release Eng | CI run 39108742495 | Fixed `clippy::useless_format` in `crates/runtime/src/php.rs`; updated `.github/workflows/ci.yml` |
| **TASK-R0-04** | Project Memory Suite Reconciliation | Critical | COMPLETED | Principal Architect | Audit A1 | Reconciled `PROJECT_STATE.md`, `ROADMAP.md`, `TASKS.md`, `HANDOFF.md`, `brain.md`, `docs/tasks.md` |
| **TASK-R0-05** | Security & Safety Regression Test Suite | Critical | COMPLETED | QA Lead | TASK-R0-01..003 | 18/18 workspace tests PASS (deployment ledger immutability, MCP self-approval rejection, dry-run simulation) |

---

## Historical Phase Baseline (Post-Audit A1 Assessment)

### Phase 0: Foundation & Project Memory (COMPLETED)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-001** | Environment and Repository Audit | Critical | COMPLETED | DevOps / Systems Eng | Verified hardware & toolchain |
| **TASK-002** | Architecture Research & Tech Stack Selection | Critical | COMPLETED | Principal Architect | Stack selection verified |
| **TASK-003** | Architecture Decision Records (ADR 1–8) | Critical | COMPLETED | Principal Architect | ADR-0001 through ADR-0008 |
| **TASK-004** | Project Memory System Initialization | Critical | COMPLETED | Project Manager | Remediated in R0 |
| **TASK-005** | Product Documentation Suite | High | COMPLETED | Product Owner | Vision, Req, Matrix, Non-Goals |
| **TASK-006** | Security Baseline & Threat Modeling | Critical | COMPLETED | Security Engineer | Threat Model, Security Policy |
| **TASK-007** | Engineering Standards & Repository Setup | High | COMPLETED | DevOps / QA | CONTRIBUTING, .gitignore, README |
| **TASK-008** | Phase 0 & Phase 1 Specifications | High | COMPLETED | Lead Architect | Specs established |
| **TASK-009** | Phase 0 Verification & Acceptance Audit | Critical | COMPLETED | Auditor Agent | Audit A1 performed |

### Phase 1: Desktop Foundation & Workspace Storage (IMPLEMENTATION EXISTS - PENDING FORMAL REVIEW)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-101** | Tauri 2 & React + Vite Scaffolding | High | IMPLEMENTED | Desktop Engineer | Compiles, runs locally |
| **TASK-102** | Core Crates Setup in Cargo Workspace | High | IMPLEMENTED | Senior Rust Eng | Modular workspace established |
| **TASK-103** | macOS APFS Volume Detection & Safety Guards | Critical | IMPLEMENTED | Systems Engineer | Verified via unit tests |
| **TASK-104** | Project Manifest & SQLite Metadata Store | High | IMPLEMENTED | Backend / Rust Eng | Tested in memory & SQLite |
| **TASK-105** | Desktop UI Project List & Volume Health Status | Medium | IMPLEMENTED | Frontend Engineer | Apple design aesthetic applied |
| **TASK-106** | Phase 1 Verification & Test Suite | Critical | REQUIRES REVIEW | Auditor Agent | Multi-drive physical disconnect review needed |

### Phase 2: Local Runtime Engine (PARTIALLY IMPLEMENTED - INTEGRATION INCOMPLETE)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-201** | PHP-FPM Multi-Version Process Supervisor | Critical | PARTIAL | Senior Rust Eng | Binary detection & config generation works; full daemon supervision unverified |
| **TASK-202** | Nginx Reverse Proxy & Loopback Virtual Hosts | High | PARTIAL | Systems Engineer | Non-root config generation tested; active routing unverified |
| **TASK-203** | MariaDB Lifecycle & Isolated User Databases | High | PARTIAL | Systems Engineer | Process helper exists; isolated db creation unverified |
| **TASK-204** | Runtime Health Probes & Port Conflict Resolver | Medium | IMPLEMENTED | Backend / Rust Eng | Port search and socket check tests pass |

### Phase 3: Smart Deployment & Interfaces (REMEDIATED TO FAIL-CLOSED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-301** | Git-Aware Checksum Ledger & Diff Engine | Critical | COMPLETED | Senior Rust Eng | SHA-256 diff engine tested & working |
| **TASK-302** | SFTP Transfer Engine with Human Approval Gate | Critical | SIMULATION REMEDIATED | Security / Rust Eng | Simulated SFTP removed; fail-closed in R0; real transport deferred to R1 |
| **TASK-303** | LocalForge Headless CLI Binary | Medium | IMPLEMENTED | DevOps / Rust Eng | Commands compile and run |
| **TASK-304** | Model Context Protocol Server (Stdio) | High | HARDENED | AI Systems Eng | Self-approval bypass closed; token leakage removed |

### Phase 4: Hardening, Packaging & Release (REMEDIATED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-401** | GitHub Actions CI/CD & Automated Release Pipelines | High | REMEDIATED | DevOps / Release Eng | Clippy fix applied in R0; CI trigger updated |
| **TASK-402** | Security Hardening & Edge-Case Traversal Tests | Critical | COMPLETED | Security Engineer | Path traversal tests pass |
| **TASK-403** | macOS DMG Packaging & Hardened Runtime Setup | High | UNVERIFIED | Desktop Engineer | DMG workflow configured; clean Mac install unverified |
| **TASK-404** | Open-Source Community Health Suite | Medium | COMPLETED | Product Owner | Community files present |
| **TASK-405** | Final Phase 4 Verification & Closure Audit | Critical | AUDITED | Auditor Agent | Audit A1 identified deficiencies; R0 executed |
