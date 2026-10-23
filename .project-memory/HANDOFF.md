# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.B Isolated Loopback Routing & Branded Local Domains — Completed / Awaiting PO Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `d96fd550f6cb5f1bbe9f0a19c62b6e65619a569d`
- **CI Baseline**: Run `39852172536` — SUCCESS
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
- **Completed in Gate U5.B (Loopback Routing & Branded Domains)**:
  - `Database Schema V2`: Added nullable `loopback_port INTEGER` to `projects` table with partial unique index `idx_projects_loopback_port WHERE loopback_port IS NOT NULL`. Added backwards-compatible schema inspector and tested rollback mechanism.
  - `PortAllocator`: Bounded to unprivileged range `18081..18999`, thread-safe mutex, non-destructive foreign PID collision detection via `lsof` without killing foreign processes.
  - `Multi-Port Nginx Gateway`: Dynamic multi-port virtual hosts declaring dedicated port, shared gateway port, preferred `<slug>.localforge`, RFC 6761 `<slug>.localforge.localhost`, and legacy domain. Enforced HTTP 503 for stopped projects, HTTP 502 for unmounted drives. Config rollback to `.conf.bak` on invalid `nginx -t`. Clean shutdown via `nginx -s stop` and auto-stop on supervisor drop.
  - `Hybrid Domain Engine`: `ProjectRoutingEngine` with `slugify()` and Scenarios A through F typed profiles with Turkish guidance.
  - `Desktop UI & Tauri IPC`: `ProjectInspector` displays Preferred Domain, Direct Loopback with 1-click copy, Localhost candidate, Legacy domain, and gateway port. `SpatialCard` displays `<slug>.localforge` and dedicated port. `App.tsx` directly opens dedicated loopback endpoint without blocking modal if domain resolution fails. `RoutingAlertModal` provides 1-click direct loopback button.
  - `Test Verification`: 100% test coverage verified across Rust workspace and Node/Desktop suite (30/30 frontend tests PASS). Deliverable report: `docs/phases/R1_3B_U5B_LOOPBACK_IMPLEMENTATION_REPORT.md`.
- **Next Permitted Action**: Await Product Owner review and acceptance of Gate U5.B before initiating Gate U5.C. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
