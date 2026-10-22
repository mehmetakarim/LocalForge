# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5 Runtime Readiness & Isolated Loopback Routing Architecture — Design & Preflight Completed / Awaiting Implementation Approval**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
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
- **Completed in Gate U5 (Design & Read-Only Preflight)**:
  - `Decision PO-U5-01 Enforced`: `Projetest` kept at PHP 8.4 in SQLite (`localforge.db`). Audited host Homebrew `php@8.4` keg-only packaging at `/opt/homebrew/opt/php@8.4/` and proved zero-collision with global PHP 8.5.3 CLI/FPM. Created FastCGI multi-version isolation contract in `docs/phases/R1_3B_U5_RUNTIME_READINESS_REPORT.md`.
  - `Decision PO-U5-02 Enforced`: MariaDB datadir at `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb` audited (64B empty). Formulated 12-point approval-gated runbook with atomic directory layout, rollback, and PO checkpoint in `docs/phases/R1_3B_U5_MARIADB_INITIALIZATION_RUNBOOK.md`. Initialization strictly held.
  - `Decision PO-U5-03 Enforced`: Architected dedicated loopback port architecture (`18081..18999`) using Shared Multi-Port Nginx Master (Option A) in `docs/phases/R1_3B_U5_LOOPBACK_ROUTING_ARCHITECTURE.md`. Completely eliminates `/etc/hosts` and `.test` DNS dependencies.
  - `Security Threat Model Delivered`: 11 STRIDE threat vectors analyzed in `docs/phases/R1_3B_U5_SECURITY_THREAT_MODEL.md`.
  - `Acceptance Matrix Delivered`: 14 verification scenarios mapped in `docs/phases/R1_3B_U5_ACCEPTANCE_MATRIX.md`.
  - `4 High-Fidelity UX Prototypes Delivered`: HTML prototypes created in `docs/design/r1_3b/u5/` (`01_php_runtime_compatibility.html`, `02_mariadb_initialization_approval.html`, `03_loopback_routing_endpoint.html`, `04_runtime_troubleshooting.html`).
- **Next Permitted Action**: Await Product Owner review and authorization before any implementation. Do NOT begin implementation. Do NOT merge into `main`. Do NOT begin R1.3C.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
