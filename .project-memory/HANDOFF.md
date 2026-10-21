# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B Final Acceptance Closeout — Completed & Awaiting Sign-Off**
- **Active Git Branch**: `phase-r1/r1-3b-brand-implementation`
- **Target Remote**: `origin/phase-r1/r1-3b-brand-implementation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **Phase R1.3B Brand Integration Stage A**: APPROVED by Product Owner (Baseline: `9c9d720b46c9676f5023f40f1660c92dc90dad5d`, CI: `39499020338`).
  - **Phase R1.3B Stage B Implementation**: COMPLETED on dedicated branch `phase-r1/r1-3b-brand-implementation`.
  - **Phase R1.3B Final Acceptance Remediation**: COMPLETED (Commit `47fb5461557a1c2ceae90cb3d3259ced0b57c14f`, CI `39561526282` Green).
- **Completed in R1.3B Final Acceptance Closeout**:
  - `Finding 01 — Import Error Propagation`: `ImportProjectModal.tsx` catches backend import errors, displays error message in modal, keeps entered project folder path intact for correction, and only resets/closes modal on confirmed success. Added automated regression test in `r1_3b_remediation.test.mjs`.
  - `Finding 02 — Production-Code Test Coverage`: Extracted production utilities `storageCapacity.ts`, `servicesTelemetry.ts`, `browserRouting.ts`; updated production screens to consume them; eliminated duplicated logic in tests; 20/20 desktop tests directly verify production modules.
  - `Finding 03 — Domain Routing Acceptance`: Verified live Nginx `default_server` 404 response on unknown host/IP; verified project isolation (Project A -> Project A only, Project B -> Project B only); verified via live Darwin test `test_r1_3b_live_domain_routing_acceptance_and_isolation` (PASS in 2.20s).
  - `Finding 04 — Genuine Native macOS Tauri Application Evidence`: Built native binary (`target/debug/localforge-desktop`), launched with live SQLite database (`localforge.db`) containing persistent projects `r1_acceptance_project` and `Projetest` on `/Volumes/Mac Harici Disk`. Captured 7 verified 1600×1000 screenshots (Spatial Studio 2 projects, Project Inspector open, Projects Workspace, Services & Runtime, Storage & Volumes, Import Modal, Welcome Splash) in `docs/design/r1_3b/native_tauri_verified/` and packaged into `localforge_r1_3b_native_tauri_screenshots.zip`. OCR verified live persistent data.
  - `Full Verification`: 58/58 Rust workspace tests PASS; 20/20 frontend tests PASS (78/78 total); zero clippy warnings; zero format diffs; clean Vite build.
- **Next Permitted Action**: Submit R1.3B Final Acceptance Closeout report to Product Owner. Await final sign-off. Do NOT merge into `main`. Do NOT begin R1.3C.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-brand-implementation`; never merge into `main` without PO approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **Database Deletion Lock**: Database deletion remains strictly disabled in the UI with `DatabaseDeletionUnavailable` guard.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
