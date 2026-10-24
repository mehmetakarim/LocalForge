# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.B1 Routing Security Hardening & Acceptance Closeout — Completed / Awaiting PO Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `55a965b7d3c9bbce975795181aa3f51bd10fd2e1`
- **CI Baseline**: Run `39917796258` — SUCCESS
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
- **Completed in Gate U5.B1 (Routing Security Hardening & Acceptance Closeout)**:
  - `B1-01 Nginx Process Identity Safety`: 7-step pre-signal verification in `NginxSupervisor::stop()`. Validates executable path, command-line arguments referencing LocalForge `conf_dir`, and PID reuse start-time before issuing signals. Implemented `paths_match` handling macOS `/private` symlink divergence. Added adversarial PID replacement unit tests.
  - `B1-02 Removal of Unsafe Port Fallback`: Missing `loopback_port` is an explicit `UNASSIGNED` state across Rust IPC (`service.rs`, `routing.rs`) and UI (`browserRouting.ts`, `App.tsx`, `SpatialCard`, `ProjectInspector`, `RoutingAlertModal`). Browser launching and direct endpoints are strictly blocked on unassigned or out-of-range ports. Range `18081..=18999` strictly enforced.
  - `B1-03 Nginx Shared-Port Host Isolation`: Dedicated port includes default server 404 for Host header mismatch. Shared 8080 block matches ONLY project-specific domains, eliminating duplicate `localhost`/`127.0.0.1` server name warnings. `validate_configuration()` fails closed on conflicting server names.
  - `B1-04 SQLite V2 Migration Fault Tolerance`: Checks both column and unique index (`has_unique_index()`). Recovers cleanly from interrupted migrations. Detects pre-existing duplicate ports and fails closed. Bounds `18081..=18999` enforced in DB queries.
  - `Test Verification`: 100% test coverage verified across Rust workspace (14/14 integration, 16/16 runtime unit, 7/7 storage, 5/5 platform, 5/5 deployment, 3/3 MCP, 2/2 desktop) and Node desktop suite (32/32 tests PASS). Clippy clean (0 warnings). Rust fmt clean (0 diffs). Deliverable report: `docs/phases/R1_3B_U5B1_ROUTING_SECURITY_CLOSEOUT.md`.
- **Next Permitted Action**: Await Product Owner review and acceptance of Gate U5.B1 before initiating Gate U5.C. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
