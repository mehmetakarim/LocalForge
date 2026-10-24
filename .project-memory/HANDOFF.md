# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.B2 Final Acceptance Patch — Completed / Awaiting PO Final Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `0954d49d799015eb6f8df7cf2d5bcf0e976bcecf`
- **CI Baseline**: Run `40123012479` — SUCCESS (100% Green on macOS Apple Silicon)
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
- **Completed in Gate U5.B2 Final Acceptance Patch**:
  - `Finding 01 PID Reuse and Signal Delivery Verification (P0)`: Canonical `signal_process_verified` abstraction in `ProcessManager` revalidating UID, path, and microsecond start time (`tv_usec`) immediately before `libc::kill`; clean handling of `libc::ESRCH` and `libc::EPERM`; bounded graceful shutdown (1s); worker group teardown via `terminate_process_group_verified`; post-termination master dead check before removing `nginx.pid`; adversarial tests covering process exit and process replacement.
  - `Finding 02 Project Identity & HTTP Readiness Verification (P1)`: Added zero-touch `/__localforge_health` endpoint to Nginx vhost returning `{"status":"ready","project_id":"<id>","port":<port>}` with `X-LocalForge-Project-Id` response header; two-stage readiness probe in `verify_browser_readiness`; rejection of Gateway Host Mismatch strings; acceptance of 4xx only after identity proven; complete removal of blind fallback retry with `Host: 127.0.0.1`.
  - `Finding 03 Strict Multi-PID Listener Ownership Verification (P1)`: Generalized `check_listener_health(port)` scrutinizing all listening PIDs for exact canonical binary, cmdline `-c`/`-p`, and start time; immediate `PortConflict` on foreign or mixed ownership; integrated into Layer 3; adversarial listener tests.
  - `Test Verification`: 100% test coverage verified across Rust workspace (69/69 tests PASS) and Node desktop suite (33/33 tests PASS). Clippy clean (0 warnings, `-D warnings`). Rust fmt clean (0 diffs). Deliverable report: `docs/phases/R1_3B_U5B2_FINAL_SECURITY_ACCEPTANCE.md`.
- **Next Permitted Action**: Await Product Owner final acceptance of Gate U5.B2 Patch before initiating Gate U5.C. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
