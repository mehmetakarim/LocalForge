# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.E Activation Runbook Reconciled — Completed / Awaiting PO Explicit Activation Approval**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `0954d49d799015eb6f8df7cf2d5bcf0e976bcecf` (Code), `86cb93b2a0c49741e4da250495eb5a6e70908866` (Docs)
- **CI Baseline**: Run `40123012479` (Code & Tests) & Run `40140575584` (Preflight & Plan) — SUCCESS (100% Green on macOS Apple Silicon)
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
  - **Gate U5.A (Architecture Safety Reconciliation)**: COMPLETED (Commit `d96fd55`).
  - **Gate U5.B (Isolated Loopback Routing)**: COMPLETED (Commit `55a965b`).
  - **Gate U5.B1 (Routing Security Hardening)**: COMPLETED (Commit `032ba2a`, CI `40100159014`).
  - **Gate U5.B2 (Final Process Safety & Routing Acceptance)**: COMPLETED (Commit `55225d1`, CI `40114186966`).
  - **Gate U5.B2 Final Acceptance Patch**: COMPLETED & ACCEPTED (Commit `0954d49`, CI `40123012479`).
- **Completed in Gate U5.E (Real Mac Activation Preflight & Migration Plan)**:
  - `Read-Only Real Host Audit`: Verified macOS M1 Darwin 27.0.0, external APFS volume (`7E26530E-5179-414C-81B5-6FBDB429FC65`), Application Support directory, and Schema V1 database (12KB, WAL/SHM). Zero database writes performed.
  - `Project Records Verification`: Confirmed `r1_acceptance_project` (PHP 8.5, degraded) and `Projetest` (PHP 8.4 preserved, stopped).
  - `Daemon & Port Audit`: Verified Nginx PID 39566 (port 8080) and PHP-FPM PID 59625; identified ports 18097/18098 occupied by test instances; 916 candidate ports free in `18081..=18999`.
  - `DNS Resolution Audit`: Verified native RFC 6761 `.localhost` loopback resolution without `/etc/hosts`.
  - `SQLite WAL Backup Protocol`: Formulated online atomic `.backup` / `VACUUM INTO` procedure flushing WAL frames safely.
  - `Controlled Activation & Rollback Plan`: Formulated 4-checkpoint migration procedure (V2 migration, port assignment 18081/18082, dual-port vhost), Safari/Chrome acceptance matrix, fail-closed rollback protocol.
  - Deliverable report: `docs/phases/R1_3B_U5E_REAL_MAC_ACTIVATION_PREFLIGHT.md`.
- **Next Permitted Action**: Await Product Owner explicit activation approval of Gate U5.E before modifying `localforge.db`, assigning ports, or reloading Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
