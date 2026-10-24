# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.B2 Final Process Safety & Routing Acceptance — Completed / Awaiting PO Final Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `032ba2a036bb059902cdbdcf20b24508afce6b34`
- **CI Baseline**: Run `40100159014` — SUCCESS
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
- **Completed in Gate U5.B2 (Final Process Safety & Routing Acceptance)**:
  - `Finding 01 Nginx PID File TOCTOU Elimination (P0)`: Eradicated unsafe `nginx -s stop` command; direct POSIX `libc::kill(identity.pid, SIGQUIT)` with 100ms grace period and verified process group teardown; macOS `ngx_setproctitle` rewritten proctitle parsing support; adversarial test `test_adversarial_pid_file_replacement_after_verification_does_not_signal_replacement`.
  - `Finding 02 SQLite Index Definition Integrity Verification (P1)`: Multi-vector index validation (`validate_loopback_port_index`) inspecting columns, partial predicate (`WHERE loopback_port IS NOT NULL`), and uniqueness; fail-closed defense against incompatible indexes; comprehensive unit test suite in `localforge-storage`.
  - `Finding 03 Six-Layer Browser Readiness Verification (P1)`: 6 discrete layers probed (DNS, Port, Listener ownership, Nginx config, Project runtime readiness [FPM + socket], HTTP response); Tauri IPC command `verify_browser_readiness`; desktop UI gate in `App.tsx` preventing broken browser launches with Turkish error toasts; unit and integration tests.
  - `Test Verification`: 100% test coverage verified across Rust workspace (66/66 tests PASS) and Node desktop suite (33/33 tests PASS). Clippy clean (0 warnings, `-D warnings`). Rust fmt clean (0 diffs). Deliverable report: `docs/phases/R1_3B_U5B2_FINAL_SECURITY_ACCEPTANCE.md`.
- **Next Permitted Action**: Await Product Owner final acceptance of Gate U5.B2 before initiating Gate U5.C. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
