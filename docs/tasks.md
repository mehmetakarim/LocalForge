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

### Subphase R1.3A-V: Spatial Studio Visual Recovery (COMPLETED / ACCEPTED)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3A-V-01** | Stage A: High-Fidelity Direction A+ Proposal | Critical | COMPLETED | Lead UI Designer | High-res Direction A+ HTML prototype & 4 scenario screenshots |
| **TASK-R1.3A-V-02** | Stage B: Spatial Monoliths & Stage Composition | Critical | COMPLETED | Frontend Lead | Translucent vertical monoliths, floor reflections, centered 3D perspective |
| **TASK-R1.3A-V-03** | Stage B: Project Inspector Drawer Integration | High | COMPLETED | Desktop Engineer | Slide-over inspector (`ProjectInspector.tsx`), technical telemetry, cURL helper |
| **TASK-R1.3A-V-04** | Stage B: Honest Runtime & APFS Telemetry Binding | Critical | COMPLETED | Senior Rust Eng | Zero fabricated data in production path; honest hero status derivation |
| **TASK-R1.3A-V-05** | Stage B: Multi-Project & Desktop Visual Capture | Critical | COMPLETED | QA Lead | 5 high-res screenshots (0, 2, 6 projects, inspector, 1024x768 responsive) |

### Subphase R1.3A-V1: Final Functional Acceptance Fixes (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3A-V1-01** | Remove Production-Accessible Fixtures | Critical | COMPLETED | Desktop Architect | `visualFixtures.ts` with `import.meta.env.PROD` exclusion gate; fail-closed web fallback |
| **TASK-R1.3A-V1-02** | Verified System-Health Messaging | Critical | COMPLETED | Frontend Lead | `computeHeroHealthContent` checking volumes, Nginx 8080, MariaDB, probe failures, FPM |
| **TASK-R1.3A-V1-03** | Propagate Synchronization Failures | Critical | COMPLETED | Senior Rust Eng | Eliminated `let _ =`, synced `Unknown` status on probe failures, updated cards/inspector |
| **TASK-R1.3A-V1-04** | Multi-Project Spatial Pagination (>6 Projects) | High | COMPLETED | Lead UI Designer | Truthful project count, fluid pagination controls, keyboard arrow navigation |
| **TASK-R1.3A-V1-05** | Negative Regression & Verification Suite | Critical | COMPLETED | QA Lead | 50 Rust tests + 15 frontend tests pass (65/65 PASS); Final Acceptance Patch Report |

### Subphase R1.3B Visual Design Gate & Narrow Reliability Review (COMPLETED / AWAITING APPROVAL)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3B-PREP-01** | Nginx Canonical Exact Path & Port Conflict Hardening | Critical | COMPLETED | Senior Rust Eng | Exact canonical config path matching, substring rejection, fail-closed multi-listener conflict handling |
| **TASK-R1.3B-PREP-02** | Metadata Query Error Propagation | High | COMPLETED | Senior Rust Eng | Removed `list_projects().unwrap_or_default()`, propagate DB errors to UI |
| **TASK-R1.3B-PREP-03** | Persistent Metadata Safety & Startup Failure Handling | Critical | COMPLETED | Database Architect | In-memory store isolation, `reconnect_metadata_store`, fail-closed SQLite init in AppState |
| **TASK-R1.3B-PREP-04** | Adversarial Regression Suite & Workspace Test Verification | Critical | COMPLETED | QA Lead | 57 Rust tests + 16 frontend tests (73/73 PASS); `test_is_localforge_nginx_cmdline_exact_path_matching_and_substring_rejection`, `test_r1_3b_prep_04` |
| **TASK-R1.3B-GATE-01** | Direction A+ 4 High-Fidelity Visual Prototypes & PNGs | Critical | COMPLETED | Lead UI Designer | 4 1600x1000 HTML prototypes & rendered PNGs (Projects, Services, Storage, Inspector) in `docs/design/r1_3b/` |
| **TASK-R1.3B-GATE-02** | Data Capability Mapping & Log Safeguards Matrix | Critical | COMPLETED | Systems Architect | Metric-to-command mapping, backend availability audit, safe fallbacks, ring-buffer log specification |

### Subphase R1.3B: Brand Integration & Native Workspace Implementation (COMPLETED / AWAITING ACCEPTANCE)
| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R1.3B-B1** | Spatial Studio Brand Alignment & Direction A+ Monoliths | Critical | COMPLETED | Frontend Lead | Translucent vertical monoliths, specular rim, perspective floor reflection, 0/2/6 project behavior |
| **TASK-R1.3B-B2** | Projects Workspace Native View | High | COMPLETED | Frontend Lead | Real persistent metadata listing, search input, filter pills, safe lifecycle controls |
| **TASK-R1.3B-B3** | Services & Runtime Native View | Critical | COMPLETED | Systems Engineer | Nginx port 8080 process ownership verification, PHP-FPM socket pool, MariaDB engine, zero fabricated telemetry |
| **TASK-R1.3B-B4** | Storage & Volumes Native View | High | COMPLETED | Systems Engineer | APFS Tier-1 external volume, internal container, persistent UUIDs, disk usage gauges, VolumeBarrier guard |
| **TASK-R1.3B-B5** | Detailed Project Inspector Drawer | Critical | COMPLETED | Security Engineer | Slide-over drawer, brand identity, database deletion lock strictly enforced, SFTP dry-run only guard |
| **TASK-R1.3B-B6** | Welcome / First-Run Splash Experience | Medium | COMPLETED | Lead UI Designer | Official Apple squircle app icon, quickstart onboarding cards, non-intrusive dismiss logic |
| **TASK-R1.3B-REM** | Final Acceptance Remediation (P0-P2 Findings) | Critical | COMPLETED | Systems & QA Lead | 9 re-captured verified screenshots; safe domain routing modal & cURL; capacity integrity guard; UNKNOWN preservation; ImportProjectModal; 77/77 tests PASS |
| **TASK-R1.3B-CLOSE** | Final Acceptance Closeout (Findings 01-04) | Critical | COMPLETED | Principal Architect | Import error workflow & form state; production utility tests; live domain routing & Nginx isolation; genuine native macOS Tauri evidence (7 screenshots & zip); 78/78 tests PASS |
| **TASK-R1.3B-UAT** | UAT Failure Recovery & Native macOS Polish (Gates 2A–4) | Critical | COMPLETED | Lead Team | Dynamic PHP-FPM discovery & confirmed switcher; MariaDB safe lifecycle & datadir protection; routing diagnostics; squircle icon geometry; traffic lights clean; storage scroll & capacity gauge; non-layout-shifting HUD; 11 native Tauri screenshots; 84/84 tests PASS |
| **TASK-R1.3B-U5** | Gate U5 Runtime Readiness & Isolated Loopback Routing Architecture | Critical | COMPLETED (DESIGN) | Lead Team | PHP 8.4 keg-only isolation plan; MariaDB 12-point runbook; Shared Nginx multi-port architecture (18081..18999); 11 STRIDE threat models; 4 UX prototypes; 14-scenario acceptance matrix |
| **TASK-R1.3B-U5A** | Gate U5.A Architecture Safety Reconciliation & Implementation Readiness | Critical | COMPLETED | Lead Team | Reconciled MariaDB private staging runbook; RFC 6265 cookie & SOP boundaries; 4-stage PHP 8.4 plan; 5 explicit test statuses; implementation gate proposal (U5.B, U5.C, U5.D); `R1_3B_U5A_SECURITY_RECONCILIATION.md` |
| **TASK-R1.3B-U5B** | Gate U5.B Isolated Loopback Routing Engine & Branded Local Domains | Critical | COMPLETED | Lead Team | SQLite migration V2 (nullable loopback_port, partial UNIQUE index); PortAllocator (18081..18999, foreign PID inspection without killing); Multi-port Nginx (503 stopped, 502 unmounted, config rollback); ProjectRoutingEngine (Scenarios A-F); Desktop UI integration; 100% test coverage; `R1_3B_U5B_LOOPBACK_IMPLEMENTATION_REPORT.md` |
| **TASK-R1.3B-U5B1** | Gate U5.B1 Routing Security Hardening & Acceptance Closeout | Critical | COMPLETED | Lead Team | 7-step Nginx pre-signal identity check & TOCTOU defenses; `paths_match` macOS /private symlink normalization; complete elimination of unsafe 8080 fallback; Nginx shared port duplicate server name removal; SQLite V2 interrupted migration recovery & duplicate port detection; 100% tests PASS (14/14 integration, 16/16 runtime unit, 7/7 storage, 32/32 desktop); `R1_3B_U5B1_ROUTING_SECURITY_CLOSEOUT.md` |
| **TASK-R1.3B-U5B2** | Gate U5.B2 Final Process Safety & Routing Acceptance | Critical | COMPLETED | Lead Team | Eradication of `nginx -s stop` command; direct POSIX SIGQUIT signaling; verified process group teardown; macOS proctitle parsing support; multi-vector SQLite index definition integrity validation (`validate_loopback_port_index`); fail-closed defense against incompatible indexes; 6-layer browser readiness verification (`verify_browser_readiness`); Tauri IPC and UI gate in `App.tsx`; 100% tests PASS (66/66 Rust, 33/33 desktop); `R1_3B_U5B2_FINAL_SECURITY_ACCEPTANCE.md` |
| **TASK-R1.3B-PATCH** | Gate U5.B2 Final Acceptance Patch | Critical | COMPLETED | Lead Team | Canonical `signal_process_verified` with microsecond start time and ESRCH/EPERM handling; post-termination master dead check; zero-touch `/__localforge_health` endpoint and host mismatch rejection; generalized `check_listener_health(port)` multi-PID scrutiny; 100% tests PASS (69/69 Rust, 33/33 desktop); updated `R1_3B_U5B2_FINAL_SECURITY_ACCEPTANCE.md` |
| **TASK-R1.3B-U5E** | Gate U5.E Real Mac Activation Runbook Reconciliation | Critical | COMPLETED | Lead Team | Read-only audit of macOS M1, Schema V1 database, 2 project records; disposable drill proof of online atomic backup, migration, non-destructive quarantine and restore; dynamic process identity; DNS-safe slugification; `R1_3B_U5E_REAL_MAC_ACTIVATION_PREFLIGHT.md` |
| **TASK-R1.3B-U5F** | Gate U5.F Controlled Real Mac Routing Activation Proposal | Critical | PROPOSAL READY (HOLD) | Lead Team | Addressed all 5 PO findings; private backup directory (`0700`) & file (`0600`); single canonical Rust migration (`migrate_to_v2_loopback_ports()`); Host-header aware HTTP identity probes (`/__localforge_health`); honest stopped semantics for Projetest (PHP 8.4); reverse rollback ordering; execution held pending explicit authorization; `R1_3B_U5F_CONTROLLED_ACTIVATION_PROPOSAL.md` |
| **TASK-R1.3B-U5F-EXEC** | Gate U5.F Execution Blocker Remediation & CLI Migration Tool | Critical | COMPLETED (EXECUTION ON HOLD) | Lead Team | Implemented `localforge storage migrate-v2` with default read-only `--dry-run` (immutable URI, zero write lock) and gated `--execute`; clarified stopped Projetest HTTP 503 response; exact quarantine allowlist; execution held pending explicit authorization |

---

## Recovery Phase R0: Safety Stabilization & CI Repair (COMPLETED / CONDITIONALLY ACCEPTED)

| Task ID | Title | Priority | Status | Assigned Role | Deliverable |
|---|---|---|---|---|---|
| **TASK-R0-01** | Fail-Closed Deployment Engine | Critical | COMPLETED | Rust Security Auditor | `crates/deployment/src/sftp.rs` rejects non-dry-run with `DeploymentNotImplemented` |
| **TASK-R0-02** | MCP Human Approval Security Barrier | Critical | COMPLETED | Security Engineer | `tools/mcp-server/src/main.rs` eliminates `approval_token` exposure; ADR-0009 |
| **TASK-R0-03** | GitHub Actions CI & Clippy Repair | High | COMPLETED | DevOps / Release Eng | Fixed `clippy::useless_format` in `crates/runtime/src/php.rs`; CI Green (Run 39123438754) |
| **TASK-R0-04** | Project Memory Suite Reconciliation | Critical | COMPLETED | Principal Architect | Reconciled memory files and documentation suite |
| **TASK-R0-05** | Security & Safety Regression Test Suite | Critical | COMPLETED | QA Lead | 18/18 workspace tests PASS |
