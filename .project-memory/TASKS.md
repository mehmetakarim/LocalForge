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

### Subphase R1.1: Real PHP-FPM & Nginx Integration (COMPLETED / CONDITIONALLY ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.1-01** | Unprivileged PHP-FPM Child Process Spawning | Critical | COMPLETED | Senior Rust Eng | R1.0 Approval | FastCGI unix sockets, dedicated pool configs, 104-byte limit mitigation |
| **TASK-R1.1-02** | Shared Nginx Loopback Master Process Gateway | High | COMPLETED | Systems Engineer | TASK-R1.1-01 | Unprivileged port 8080 routing, dynamic vhost reloads, `nginx -t` validation |
| **TASK-R1.1-03** | Verified Process Identity & Registry Engine | Critical | COMPLETED | Senior Rust Eng | TASK-R1.1-01 | Darwin `libc::proc_pidinfo` + `proc_bsdinfo` timestamp + path PID reuse defense |
| **TASK-R1.1-04** | End-to-End Loopback HTTP 200 Verification | Critical | COMPLETED | QA Lead | TASK-R1.1-01..03 | Automated HTTP GET verification against live socket, dynamic JSON payload |

### Subphase R1.1A: Targeted Acceptance Remediation (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.1A-01** | Eliminate False RUNNING Status | Critical | COMPLETED | Senior Rust Eng | R1.1 Findings | `GatewayHealthAssessment`, 502/504 -> Degraded, 404 routing separation |
| **TASK-R1.1A-02** | Harden Process Ownership Verification | Critical | COMPLETED | Security Specialist | R1.1 Findings | Removed basename fallback; verify UID; isolate PGID against own group |
| **TASK-R1.1A-03** | Honest Local URL Reporting | High | COMPLETED | Desktop/CLI Eng | R1.1 Findings | Report `127.0.0.1:8080`, mark `.test` pending, curl copy action |
| **TASK-R1.1A-04** | Negative Security & Regression Suite | Critical | COMPLETED | QA Lead | TASK-R1.1A-01..03 | 7 regression tests covering probe classification, dead sockets, spoofing |

### Subphase R1.2: MariaDB, Storage Resilience & Multi-Project Verification (COMPLETED / CONDITIONALLY ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.2-01** | User-Level MariaDB Internal Datadir Supervisor | High | COMPLETED | Systems Engineer | R1.1A Approval | `MariaDbSupervisor`, Darwin identity, unprivileged Homebrew `mariadbd` lifecycle, datadir init |
| **TASK-R1.2-02** | Authenticated SQL Health Probes & Isolation | Critical | COMPLETED | Senior Rust Eng | TASK-R1.2-01 | `SELECT 1;` socket ping, latency measurement, user/db provisioning, drop token guard |
| **TASK-R1.2-03** | Logical Backup & Restore (0600 Permissions) | High | COMPLETED | Database Architect | TASK-R1.2-02 | `mariadb-dump` with `0o600` file permissions and complete SQL restoration |
| **TASK-R1.2-04** | PHP PDO End-to-End on External APFS Storage | Critical | COMPLETED | QA Lead | TASK-R1.2-01..03 | Live PHP-FPM + Nginx + PDO on `/Volumes/.../scratch`, data persistence across restart |
| **TASK-R1.2-05** | Multi-Project FastCGI & Database Concurrency | Critical | COMPLETED | Senior Rust Eng | TASK-R1.2-01..04 | Site Alpha & Site Beta concurrent execution, independent FastCGI pools, shared Nginx |
| **TASK-R1.2-06** | External Storage Resilience & VolumeBarrier Guard | Critical | COMPLETED | Systems Engineer | TASK-R1.2-04 | Simulated/unmounted drive fail-closed protection, zero fallback writes to boot disk |

### Subphase R1.2A: Database Security Remediation (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.2A-01** | SQL Credential Escaping & Redaction | Critical | COMPLETED | Security Specialist | R1.2 Review | `escape_sql_string_literal`, null byte rejection, secret redaction from logs |
| **TASK-R1.2A-02** | Secure Atomic Backup Streaming (0600) | Critical | COMPLETED | Database Architect | TASK-R1.2A-01 | O(1) RAM streaming, atomic temp file, symlink rejection, 0600 creation |
| **TASK-R1.2A-03** | Elimination of Deletion Self-Authorization | Critical | COMPLETED | Security Specialist | TASK-R1.2A-01 | Strict failure on `db_name == token`, single-use UUIDv4 with 60s TTL |
| **TASK-R1.2A-04** | Safe Restore & Non-Empty Database Protection | Critical | COMPLETED | Database Architect | TASK-R1.2A-02 | Rejection of non-empty target databases, streaming stdin, header validation |
| **TASK-R1.2A-05** | Datadir Ownership & Graceful Termination | High | COMPLETED | Systems Engineer | TASK-R1.2A-01 | Ownership validation (`uid == geteuid()`), `mysqladmin` & SIGTERM before SIGKILL |
| **TASK-R1.2A-06** | Adversarial Security Test Suite | Critical | COMPLETED | QA Lead | TASK-R1.2A-01..05 | 5 new adversarial tests covering SQL injection, symlinks, replay, corrupt dumps |

### Subphase R1.2B: Final Database Safety Gate (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.2B-01** | Deletion Authorization Separation & UI Gate | Critical | COMPLETED | Security Specialist | R1.2A Findings | Separate `DatabaseDeletionRequest`, reject `del_req_` self-approval, disable deletion in R1.2B |
| **TASK-R1.2B-02** | Fail-Closed Restore Verification & Partial Diagnostics | Critical | COMPLETED | Database Architect | TASK-R1.2B-01 | Replaced `if let Ok`, strict parse, partial restore detection, DDL rollback note |
| **TASK-R1.2B-03** | Darwin Application Support Resolution | High | COMPLETED | Systems Engineer | TASK-R1.2B-01 | Removed `/Users/mehmetakar` fallback, `resolve_application_support_dir` with getpwuid |
| **TASK-R1.2B-04** | R1.2B Safety Regression Test Suite | Critical | COMPLETED | QA Lead | TASK-R1.2B-01..03 | 13 MariaDB integration tests, occupied destination, broken dump diagnostics |

### Subphase R1.3A-V: Spatial Studio Visual Recovery (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.3A-V-01** | Stage A: High-Fidelity Direction A+ Proposal | Critical | COMPLETED | Lead UI Designer | PO Review | High-res Direction A+ HTML prototype & 4 scenario screenshots |
| **TASK-R1.3A-V-02** | Stage B: Spatial Monoliths & Stage Composition | Critical | COMPLETED | Frontend Lead | Stage A Gate | Translucent vertical monoliths, floor reflections, centered 3D perspective |
| **TASK-R1.3A-V-03** | Stage B: Project Inspector Drawer Integration | High | COMPLETED | Desktop Engineer | TASK-R1.3A-V-02 | Slide-over inspector (`ProjectInspector.tsx`), technical telemetry, cURL helper |
| **TASK-R1.3A-V-04** | Stage B: Honest Runtime & APFS Telemetry Binding | Critical | COMPLETED | Senior Rust Eng | TASK-R1.3A-V-02 | Zero fabricated data in production path; honest hero status derivation |
| **TASK-R1.3A-V-05** | Stage B: Multi-Project & Desktop Visual Capture | Critical | COMPLETED | QA Lead | TASK-R1.3A-V-01..04 | 5 high-res screenshots (0, 2, 6 projects, inspector, 1024x768 responsive) |

### Subphase R1.3A-V1: Final Functional Acceptance Fixes (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.3A-V1-01** | Remove Production-Accessible Fixtures | Critical | COMPLETED | Desktop Architect | R1.3A-V Findings | `visualFixtures.ts` with `import.meta.env.PROD` exclusion gate; fail-closed web fallback |
| **TASK-R1.3A-V1-02** | Verified System-Health Messaging | Critical | COMPLETED | Frontend Lead | R1.3A-V Findings | `computeHeroHealthContent` checking volumes, Nginx 8080, MariaDB, probe failures, FPM |
| **TASK-R1.3A-V1-03** | Propagate Synchronization Failures | Critical | COMPLETED | Senior Rust Eng | R1.3A-V Findings | Eliminated `let _ =`, synced `Unknown` status on probe failures, updated cards/inspector |
| **TASK-R1.3A-V1-04** | Multi-Project Spatial Pagination (>6 Projects) | High | COMPLETED | Lead UI Designer | R1.3A-V Findings | Truthful project count, fluid pagination controls, keyboard arrow navigation |
| **TASK-R1.3A-V1-05** | Negative Regression & Verification Suite | Critical | COMPLETED | QA Lead | TASK-R1.3A-V1-01..04 | 50 Rust tests + 15 frontend tests pass (65/65 PASS); Final Acceptance Patch Report |

### Subphase R1.3B Visual Design Gate & Narrow Reliability Review (COMPLETED / AWAITING APPROVAL)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.3B-PREP-01** | Nginx Canonical Exact Path & Port Conflict Hardening | Critical | COMPLETED | Senior Rust Eng | R1.3A-V1 Acceptance | Exact canonical config path matching, substring rejection, fail-closed multi-listener conflict handling |
| **TASK-R1.3B-PREP-02** | Metadata Query Error Propagation | High | COMPLETED | Senior Rust Eng | TASK-R1.3B-PREP-01 | Removed `list_projects().unwrap_or_default()`, propagate DB errors to UI |
| **TASK-R1.3B-PREP-03** | Persistent Metadata Safety & Startup Failure Handling | Critical | COMPLETED | Database Architect | TASK-R1.3B-PREP-02 | In-memory store isolation, `reconnect_metadata_store`, fail-closed SQLite init in AppState |
| **TASK-R1.3B-PREP-04** | Adversarial Regression Suite & Workspace Test Verification | Critical | COMPLETED | QA Lead | TASK-R1.3B-PREP-01..03 | 57 Rust tests + 16 frontend tests (73/73 PASS); `test_is_localforge_nginx_cmdline_exact_path_matching_and_substring_rejection`, `test_r1_3b_prep_04` |
| **TASK-R1.3B-GATE-01** | Direction A+ 4 High-Fidelity Visual Prototypes & PNGs | Critical | COMPLETED | Lead UI Designer | Direction A+ PO Ref | 4 1600x1000 HTML prototypes & rendered PNGs (Projects, Services, Storage, Inspector) in `docs/design/r1_3b/` |
| **TASK-R1.3B-GATE-02** | Data Capability Mapping & Log Safeguards Matrix | Critical | COMPLETED | Systems Architect | TASK-R1.3B-GATE-01 | Metric-to-command mapping, backend availability audit, safe fallbacks, ring-buffer log specification |

### Subphase R1.3B: Brand Integration & Native Workspace Implementation (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Dependencies | Deliverables |
|---|---|---|---|---|---|---|
| **TASK-R1.3B-STAGE-A** | Canonical Brand Integration & Prototype Assets | Critical | APPROVED | Lead UI Designer | Brand Kit ZIP | 6 high-res HTML prototypes & PNGs; CI Run 39499020338 Green |
| **TASK-R1.3B-B1** | Spatial Studio Brand Alignment & Direction A+ Monoliths | Critical | COMPLETED | Frontend Lead | Stage A Gate | Translucent vertical monoliths, specular rim, perspective floor reflection, 0/2/6 project behavior |
| **TASK-R1.3B-B2** | Projects Workspace Native View | High | COMPLETED | Frontend Lead | TASK-R1.3B-B1 | Real persistent metadata listing, search input, filter pills, safe lifecycle controls |
| **TASK-R1.3B-B3** | Services & Runtime Native View | Critical | COMPLETED | Systems Engineer | TASK-R1.3B-B1 | Nginx port 8080 process ownership verification, PHP-FPM socket pool, MariaDB engine, zero fabricated telemetry |
| **TASK-R1.3B-B4** | Storage & Volumes Native View | High | COMPLETED | Systems Engineer | TASK-R1.3B-B1 | APFS Tier-1 external volume, internal container, persistent UUIDs, disk usage gauges, VolumeBarrier guard |
| **TASK-R1.3B-B5** | Detailed Project Inspector Drawer | Critical | COMPLETED | Security Engineer | TASK-R1.3B-B1 | Slide-over drawer, brand identity, database deletion lock strictly enforced, SFTP dry-run only guard |
| **TASK-R1.3B-B6** | Welcome / First-Run Splash Experience | Medium | COMPLETED | Lead UI Designer | Stage A Gate | Official Apple squircle app icon, quickstart onboarding cards, non-intrusive dismiss logic |
| **TASK-R1.3B-B7** | Full Workspace Automated Test & Visual Verification | Critical | COMPLETED | QA Lead | TASK-R1.3B-B1..B6 | 57 Rust + 16 frontend tests (73/73 PASS); clean TypeScript build; 9 verified high-res PNG screenshots packaged in ZIP |

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
