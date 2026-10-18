# LocalForge Tasks & Work Packages

This document mirrors and indexes the active engineering tasks across project phases. For the canonical, machine-readable backlog, see [`.project-memory/TASKS.md`](../.project-memory/TASKS.md).

---

## Task Management Rules
1. **Lead / Auditor Authority**: Only the Lead Agent or Auditor Agent may modify task states. Worker agents report results and must not modify this file directly.
2. **Phase Discipline**: Work must strictly remain within the currently authorized subphase (**Recovery Phase R1.0 — Architecture Refinement**). Do not start implementation of R1.1, R1.2, or R1.3 without explicit Product Owner approval.

---

## Recovery Phase R1: Runtime Integration & Spatial Studio (ACTIVE)

### Subphase R1.0: Architecture Refinement (COMPLETED / ACTIVE IN REVIEW)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.0-01** | Local Domain Resolution & Loopback Architecture | Critical | COMPLETED | Systems Architect | Decouple Host-header loopback from browser `.test` resolver |
| **TASK-R1.0-02** | MariaDB Storage Policy & Internal Isolation | Critical | COMPLETED | Database Architect | Internal app storage default; decouple external source |
| **TASK-R1.0-03** | Verified Process Identity & Lifecycle Safety | Critical | COMPLETED | Senior Rust Eng | Path + ticks PID reuse defense; PGID; shared Nginx lifecycle |
| **TASK-R1.0-04** | Spatial Studio Performance & Hardware Budgets | High | COMPLETED | Lead UI Designer | CPU <1%, RAM <80MB, max 2 blurs, animation pause guards |
| **TASK-R1.0-05** | R1 Subphase Governance & Spec Documents | Critical | COMPLETED | Principal Architect | `R1_0_ARCHITECTURE_REFINEMENT.md` through `R1_3` |
| **TASK-R1.0-06** | Architecture Decision Records (ADR 10–12) | Critical | COMPLETED | Principal Architect | `ADR-0010`, `ADR-0011` (amended), `ADR-0012` |

### Subphase R1.1: Real PHP-FPM & Nginx Integration (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.1-01** | Unprivileged PHP-FPM Child Process Spawning | Critical | COMPLETED | Senior Rust Eng | FastCGI unix sockets, dedicated pool configs |
| **TASK-R1.1-02** | Shared Nginx Loopback Master Process Gateway | High | COMPLETED | Systems Engineer | Unprivileged port 8080 routing, dynamic vhost reloads |
| **TASK-R1.1-03** | Verified Process Identity & Registry Engine | Critical | COMPLETED | Senior Rust Eng | In-memory registry, PID reuse check, bounded SIGTERM |
| **TASK-R1.1-04** | End-to-End Loopback HTTP 200 Verification | Critical | COMPLETED | QA Lead | Automated HTTP GET verification against live socket |

### Subphase R1.1A: Targeted Acceptance Remediation (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.1A-01** | Eliminate False RUNNING Status | Critical | COMPLETED | Senior Rust Eng | Gateway health assessment, probe classification |
| **TASK-R1.1A-02** | Harden Process Ownership Verification | Critical | COMPLETED | Security Specialist | UID matching, PGID isolation |
| **TASK-R1.1A-03** | Honest Local URL Reporting | High | COMPLETED | Desktop/CLI Eng | Report 127.0.0.1:8080, loopback separation |
| **TASK-R1.1A-04** | Negative Security & Regression Suite | Critical | COMPLETED | QA Lead | 7 regression tests covering probes & sockets |

### Subphase R1.2: MariaDB, Storage Resilience & Multi-Project Verification (COMPLETED / CONDITIONALLY ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.2-01** | User-Level MariaDB Internal Datadir Supervisor | High | COMPLETED | Systems Engineer | Port 3306 binding, socket ping, internal SSD datadir |
| **TASK-R1.2-02** | Concurrent Multi-Version PHP Project Execution | Critical | COMPLETED | Senior Rust Eng | Project A & B live side-by-side |
| **TASK-R1.2-03** | External APFS Volume Disconnect Resilience | Critical | COMPLETED | Systems Engineer | Physical/simulated disconnect safety without write fallback |

### Subphase R1.2A: Database Security Remediation (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.2A-01** | SQL Credential Escaping & Redaction | Critical | COMPLETED | Security Specialist | `escape_sql_string_literal`, null byte rejection, secret scrubbing |
| **TASK-R1.2A-02** | Secure Atomic Backup Streaming (0600) | Critical | COMPLETED | Database Architect | O(1) RAM streaming, atomic temp file, symlink rejection, 0600 creation |
| **TASK-R1.2A-03** | Elimination of Deletion Self-Authorization | Critical | COMPLETED | Security Specialist | Single-use UUIDv4 token with 60s TTL, anti-self-approval |
| **TASK-R1.2A-04** | Safe Restore & Non-Empty Database Protection | Critical | COMPLETED | Database Architect | Rejection of non-empty target databases, streaming stdin |
| **TASK-R1.2A-05** | Datadir Ownership & Graceful Termination | High | COMPLETED | Systems Engineer | Datadir UID validation, `mysqladmin` & SIGTERM before SIGKILL |
| **TASK-R1.2A-06** | Adversarial Security Test Suite | Critical | COMPLETED | QA Lead | 5 new adversarial tests covering SQL injection, symlinks, replay |

### Subphase R1.2B: Final Database Safety Gate (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.2B-01** | Deletion Authorization Separation & UI Gate | Critical | COMPLETED | Security Specialist | Separate `DatabaseDeletionRequest`, reject self-approval, disable deletion in R1.2B |
| **TASK-R1.2B-02** | Fail-Closed Restore Verification & Partial Diagnostics | Critical | COMPLETED | Database Architect | Replaced `if let Ok`, strict parse, partial restore detection, DDL rollback note |
| **TASK-R1.2B-03** | Darwin Application Support Resolution | High | COMPLETED | Systems Engineer | Removed `/Users/mehmetakar` fallback, `resolve_application_support_dir` with getpwuid |
| **TASK-R1.2B-04** | R1.2B Safety Regression Test Suite | Critical | COMPLETED | QA Lead | 13 MariaDB integration tests, occupied destination, broken dump diagnostics |

### Subphase R1.3A: Spatial Studio Foundation & Shell (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3A-01** | Spatial Studio Design System Token Integration | High | COMPLETED | Frontend Lead | CSS tokens, void canvas, atmospheric indigo glow in `index.css` |
| **TASK-R1.3A-02** | SpatialCard & 3D Interactive Stage | High | COMPLETED | Frontend Engineer | 2.5D micro-tilt cards, status halos, action deck |
| **TASK-R1.3A-03** | FloatingDock Navigation & AICommandCapsule | High | COMPLETED | Desktop Engineer | Liquid glass dock (`⌘1..7`), Cmd+K command palette with MCP actions |
| **TASK-R1.3A-04** | Embedded Dist Distribution & Window Integration | Critical | COMPLETED | Tauri Architect | `devUrl: null`, embedded dist frontend, traffic-light clearance, native dragging |
| **TASK-R1.3A-05** | Performance & Accessibility Budget Verification | Critical | COMPLETED | QA Lead | CPU <0.3%, RAM 52MB, max 2 blurs, WCAG AA, real screenshot captures |

### Subphase R1.3B: Native Workspace & Detailed Inspector (PLANNED / LOCKED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3B-01** | Native Workspace Detailed Inspector View | High | PLANNED | Frontend Lead | Per-project detailed telemetry, live log stream, environment variable editor |
| **TASK-R1.3B-02** | Interactive Human Approval Gate for Deletions | Critical | PLANNED | Security Engineer | Multi-step human confirmation modal for destructive database drops |

---

## Recovery Phase R0: Safety Stabilization & CI Repair (COMPLETED / CONDITIONALLY ACCEPTED)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R0-01** | Fail-Closed Deployment Engine | Critical | COMPLETED | Rust Security Auditor | `crates/deployment/src/sftp.rs` rejects non-dry-run with `DeploymentNotImplemented` |
| **TASK-R0-02** | MCP Human Approval Security Barrier | Critical | COMPLETED | Security Engineer | `tools/mcp-server/src/main.rs` eliminates `approval_token` exposure; ADR-0009 |
| **TASK-R0-03** | GitHub Actions CI & Clippy Repair | High | COMPLETED | DevOps / Release Eng | Fixed `clippy::useless_format` in `crates/runtime/src/php.rs`; CI Green (Run 39123438754) |
| **TASK-R0-04** | Project Memory Suite Reconciliation | Critical | COMPLETED | Principal Architect | Reconciled memory files and documentation suite |
| **TASK-R0-05** | Security & Safety Regression Test Suite | Critical | COMPLETED | QA Lead | 18/18 workspace tests PASS |
