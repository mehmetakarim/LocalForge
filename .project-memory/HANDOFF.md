# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.A Architecture Safety Reconciliation & Implementation Readiness — Completed / Awaiting Implementation Approval**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `4e2d762a5c3b4bd7437e33fb37a821b64c4c5c3d`
- **CI Baseline**: Run `39848607260` — SUCCESS
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **Phase R1.3B Brand Integration Stage A**: APPROVED by Product Owner (Baseline: `9c9d720b46c9676f5023f40f1660c92dc90dad5d`, CI: `39499020338`).
  - **Phase R1.3B Stage B Implementation**: COMPLETED on dedicated branch `phase-r1/r1-3b-brand-implementation`.
  - **Phase R1.3B Final Acceptance Remediation**: COMPLETED (Commit `47fb5461557a1c2ceae90cb3d3259ced0b57c14f`, CI `39561526282` Green).
  - **Phase R1.3B Final Acceptance Closeout**: COMPLETED (Commit `f6302479efcbebf907837f176d68962acf208e4f`).
  - **Phase R1.3B UAT Failure Recovery (Gates 2A–4)**: COMPLETED (Commit `8516983`).
  - **Gate U5 (Design & Read-Only Preflight)**: COMPLETED (Commit `4e2d762`).
- **Completed in Gate U5.A (Architecture Safety Reconciliation)**:
  - `Task 1 (MariaDB Runbook Security Correction)`: Updated `R1_3B_U5_MARIADB_INITIALIZATION_RUNBOOK.md` with private staging directory (`staging_<uuid>`) on `/System/Volumes/Data`, socket peercred authentication (`--auth-root-authentication-method=socket`), atomic same-filesystem `rename()` promotion, quarantined non-destructive isolation on failure (`.failed_<timestamp>`), no blind `rm -rf`, and preservation of database deletion lock.
  - `Task 2 (Loopback Routing Security Reconciliation)`: Updated `R1_3B_U5_LOOPBACK_ROUTING_ARCHITECTURE.md` and `SECURITY_THREAT_MODEL.md` analyzing all 14 factors. Clarified that loopback ports provide a routing/usability boundary and SOP origin partition, but NOT an OS container. Mitigated RFC 6265 cookie sharing via unique session names (`LF_SESS_<id>`), isolated session directories, and `127.0.0.X` IP alias options. Decoupled SQLite port allocation from OS port ownership with fail-closed TOCTOU conflict resolution.
  - `Task 3 (PHP 8.4 Installation Plan Hardening)`: Updated `R1_3B_U5_RUNTIME_READINESS_REPORT.md` replacing absolute guarantees with 4 testable preconditions (PATH, sbin, opt binary, config). Separated 4 sequential lifecycle stages with failure handling, removed blanket `rm -rf`, and preserved `Projetest` at PHP 8.4.
  - `Task 4 (Honest Acceptance Classification)`: Reconciled `R1_3B_U5_ACCEPTANCE_MATRIX.md` across 5 explicit statuses: `DESIGN REVIEWED`, `PREFLIGHT VERIFIED`, `IMPLEMENTED & TESTED`, `NOT RUN`, and `BLOCKED`. Explicitly listed what remains untested on the actual PO Mac.
  - `Task 5 (Implementation Gate Proposal & Deliverable)`: Created `R1_3B_U5A_SECURITY_RECONCILIATION.md` detailing corrected assumptions, security findings, and specifying 3 decoupled implementation packages: Gate U5.B (Loopback Routing Engine), Gate U5.C (PHP 8.4 Provisioning), and Gate U5.D (MariaDB Staging Initialization) with exact scopes, preconditions, prohibited operations, test matrices, rollback strategies, and PO checkpoints.
- **Next Permitted Action**: Await Product Owner review and authorization before any implementation. Do NOT begin implementation. Do NOT merge into `main`. Do NOT begin R1.3C.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
