# LocalForge — Task Backlog

This is the canonical task backlog. Tasks have stable identifiers, priorities, explicit dependencies, and strict ownership rules.

> [!WARNING]
> Only the **Lead Agent** or **Auditor Agent** may update this file. Worker agents must report completion to the Lead Agent and not edit directly.

---

## Recovery Phase R1: Runtime Integration & Spatial Studio (ACTIVE)

### Subphase R1.0: Architecture Refinement (COMPLETED / ACTIVE IN REVIEW)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.0-01** | Local Domain Resolution & Loopback Architecture | Critical | COMPLETED | Systems Architect | ADR-0010 | Decouple Host-header loopback from browser `.test` resolver |
| **TASK-R1.0-02** | MariaDB Storage Policy & Internal Isolation | Critical | COMPLETED | Database Architect | ADR-0011 | Internal app storage default; decouple external source |
| **TASK-R1.0-03** | Verified Process Identity & Lifecycle Safety | Critical | COMPLETED | Senior Rust Eng | ADR-0011 | Path + ticks PID reuse defense; PGID; shared Nginx lifecycle |
| **TASK-R1.0-04** | Spatial Studio Performance & Hardware Budgets | High | COMPLETED | Lead UI Designer | ADR-0010 | CPU <1%, RAM <80MB, max 2 blurs, animation pause guards |
| **TASK-R1.0-05** | R1 Subphase Governance & Spec Documents | Critical | COMPLETED | Principal Architect | R0 Acceptance | `R1_0_ARCHITECTURE_REFINEMENT.md` through `R1_3` |
| **TASK-R1.0-06** | Architecture Decision Records (ADR 10–12) | Critical | COMPLETED | Principal Architect | TASK-R1.0-01..05 | `ADR-0010`, `ADR-0011` (amended), `ADR-0012` |

### Subphase R1.1: Real PHP-FPM & Nginx Integration (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.1-01** | Unprivileged PHP-FPM Child Process Spawning | Critical | COMPLETED | Senior Rust Eng | R1.0 Approval | FastCGI unix sockets, dedicated pool configs, 104-byte limit mitigation |
| **TASK-R1.1-02** | Shared Nginx Loopback Master Process Gateway | High | COMPLETED | Systems Engineer | TASK-R1.1-01 | Unprivileged port 8080 routing, dynamic vhost reloads, `nginx -t` validation |
| **TASK-R1.1-03** | Verified Process Identity & Registry Engine | Critical | COMPLETED | Senior Rust Eng | TASK-R1.1-01 | Darwin `libc::proc_pidinfo` + `proc_bsdinfo` timestamp + path PID reuse defense |
| **TASK-R1.1-04** | End-to-End Loopback HTTP 200 Verification | Critical | COMPLETED | QA Lead | TASK-R1.1-01..03 | Automated HTTP GET verification against live socket, dynamic JSON payload |

### Subphase R1.2: MariaDB, Storage Resilience & Multi-Project Verification (PLANNED / LOCKED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.2-01** | User-Level MariaDB Internal Datadir Supervisor | High | PLANNED | Systems Engineer | R1.1 Approval | Port 3306 binding, socket ping, internal SSD datadir |
| **TASK-R1.2-02** | Concurrent Multi-Version PHP Project Execution | Critical | PLANNED | Senior Rust Eng | R1.1 Approval | Project A (8.4) & B (8.3) live side-by-side |
| **TASK-R1.2-03** | External APFS Volume Disconnect Resilience | Critical | PLANNED | Systems Engineer | TASK-R1.2-02 | Physical/simulated disconnect safety without write fallback |

### Subphase R1.3: Spatial Studio UI Implementation (PLANNED / LOCKED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.3-01** | Spatial Studio Design System Token Integration | High | PLANNED | Frontend Lead | R1.2 Approval | CSS tokens, void canvas, atmospheric indigo glow |
| **TASK-R1.3-02** | SpatialCard & 3D Interactive Stage | High | PLANNED | Frontend Engineer | TASK-R1.3-01 | 2.5D micro-tilt cards, status halos, action deck |
| **TASK-R1.3-03** | FloatingDock Navigation & AICommandCapsule | High | PLANNED | Desktop Engineer | TASK-R1.3-01 | Liquid glass dock, Cmd+K command palette |
| **TASK-R1.3-04** | Performance & Accessibility Budget Verification | Critical | PLANNED | QA Lead | TASK-R1.3-01..03 | CPU <1%, RAM <80MB, WCAG AA, reduced motion |

---

## Recovery Phase R0: Safety Stabilization & CI Repair (COMPLETED / CONDITIONALLY ACCEPTED)

| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R0-01** | Fail-Closed Deployment Engine | Critical | COMPLETED | Rust Security Auditor | Audit A1 | `crates/deployment/src/sftp.rs` rejects non-dry-run with `DeploymentNotImplemented` |
| **TASK-R0-02** | MCP Human Approval Security Barrier | Critical | COMPLETED | Security Engineer | Audit A1 | `tools/mcp-server/src/main.rs` eliminates `approval_token` exposure; ADR-0009 |
| **TASK-R0-03** | GitHub Actions CI & Clippy Repair | High | COMPLETED | DevOps / Release Eng | CI run 39108742495 | Fixed `clippy::useless_format` in `crates/runtime/src/php.rs`; CI Green (Run 39123438754) |
| **TASK-R0-04** | Project Memory Suite Reconciliation | Critical | COMPLETED | Principal Architect | Audit A1 | Reconciled memory files and documentation suite |
| **TASK-R0-05** | Security & Safety Regression Test Suite | Critical | COMPLETED | QA Lead | TASK-R0-01..003 | 18/18 workspace tests PASS |

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
| **TASK-105** | Desktop UI Project List & Volume Health Status | Medium | IMPLEMENTED | Frontend Engineer | Spatial Studio redesign staged in R1.3 |
| **TASK-106** | Phase 1 Verification & Test Suite | Critical | REQUIRES REVIEW | Auditor Agent | Multi-drive physical disconnect review staged in R1.2 |

### Phase 2: Local Runtime Engine (IN PROGRESS VIA RECOVERY R1)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-201** | PHP-FPM Multi-Version Process Supervisor | Critical | REFINED IN R1.0 | Senior Rust Eng | Active implementation in R1.1 |
| **TASK-202** | Nginx Reverse Proxy & Loopback Virtual Hosts | High | REFINED IN R1.0 | Systems Engineer | Shared gateway implementation in R1.1 |
| **TASK-203** | MariaDB Lifecycle & Isolated User Databases | High | REFINED IN R1.0 | Systems Engineer | Internal datadir implementation in R1.2 |
| **TASK-204** | Runtime Health Probes & Port Conflict Resolver | Medium | REFINED IN R1.0 | Backend / Rust Eng | Verified process identity & probes in R1.1 |

### Phase 3: Smart Deployment & Interfaces (REMEDIATED TO FAIL-CLOSED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-301** | Git-Aware Checksum Ledger & Diff Engine | Critical | COMPLETED | Senior Rust Eng | SHA-256 diff engine tested & working |
| **TASK-302** | SFTP Transfer Engine with Human Approval Gate | Critical | SIMULATION REMEDIATED | Security / Rust Eng | Simulated SFTP removed; fail-closed in R0 |
| **TASK-303** | LocalForge Headless CLI Binary | Medium | IMPLEMENTED | DevOps / Rust Eng | Commands compile and run |
| **TASK-304** | Model Context Protocol Server (Stdio) | High | HARDENED | AI Systems Eng | Self-approval bypass closed; token leakage removed |

### Phase 4: Hardening, Packaging & Release (REMEDIATED IN R0)
| Task ID | Title | Priority | Status | Assigned Role | Notes |
|---|---|---|---|---|---|
| **TASK-401** | GitHub Actions CI/CD & Automated Release Pipelines | High | REMEDIATED | DevOps / Release Eng | Clippy fix applied in R0; CI Green (Run 39123438754) |
| **TASK-402** | Security Hardening & Edge-Case Traversal Tests | Critical | COMPLETED | Security Engineer | Path traversal tests pass |
| **TASK-403** | macOS DMG Packaging & Hardened Runtime Setup | High | UNVERIFIED | Desktop Engineer | Clean Mac install unverified |
| **TASK-404** | Open-Source Community Health Suite | Medium | COMPLETED | Product Owner | Community files present |
| **TASK-405** | Final Phase 4 Verification & Closure Audit | Critical | AUDITED | Auditor Agent | Audit A1 identified deficiencies; R0 executed |
