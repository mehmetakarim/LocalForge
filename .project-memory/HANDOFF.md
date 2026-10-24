# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Final Execution Proposal Corrected — Proposal Ready / Execution Strictly on Hold / Awaiting PO Explicit Activation Authorization**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `a2efcbb302c0ba30b2872b946ce7c4dcf43d6e79`
- **CI Baseline**: Run `40141502541` — SUCCESS (100% Green on macOS Apple Silicon)
- **Previous Milestone Status**:
  - **Gate U5.B2 Final Acceptance Patch**: COMPLETED & ACCEPTED (Commit `0954d49`, CI `40123012479`).
  - **Gate U5.E Activation Runbook Reconciliation**: COMPLETED & ACCEPTED (Commit `a2efcbb`, CI `40141502541`).
- **Completed in Gate U5.F (Proposal Correction)**:
  - Addressed all 5 Product Owner findings without mutating the real host environment:
    1. `Finding 01 (Backup Command)`: Canonical Application Support directory resolution, UID 501 verification, private backup dir (`mkdir -m 700`), atomic `.backup` file (`0600`), SHA-256 manifest (`0600`), and fail-closed stop conditions.
    2. `Finding 02 (Single Canonical Migration)`: Eliminated manual SQL scripts; standardized strictly on `MetadataStore::migrate_to_v2_loopback_ports()`, schema version 2 tracking, and `update_project_loopback_port()`.
    3. `Finding 03 (Host-Header Aware Probes)`: Specified explicit Host headers for `/__localforge_health` (`r1-acceptance-project.localforge` and `projetest.localforge`) and verified returned project IDs.
    4. `Finding 04 (Stopped Project Semantics)`: Formally established that `Projetest` remains in `stopped` state on dedicated port `18082` with legitimate 503 response until PHP 8.4 is provisioned.
    5. `Finding 05 (Rollback & Process Safety)`: Designated Antigravity Lead Agent as single operator (UID 501); quiescence checks; full Nginx config backup; reverse rollback ordering; dynamic kernel-verified process signaling; foreign daemons preserved.
  - Deliverable document: `docs/phases/R1_3B_U5F_CONTROLLED_ACTIVATION_PROPOSAL.md`.
- **Next Permitted Action**: Await explicit Product Owner activation authorization specifically for Gate U5.F execution before making ANY persistent modifications to `localforge.db`, writing vhosts, or signaling Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
