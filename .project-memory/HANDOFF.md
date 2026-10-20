# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B Stage B Brand Identity & Native Workspace Implementation — Completed & Awaiting Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-brand-implementation`
- **Target Remote**: `origin/phase-r1/r1-3b-brand-implementation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **Phase R1.3B Brand Integration Stage A**: APPROVED by Product Owner (Baseline: `9c9d720b46c9676f5023f40f1660c92dc90dad5d`, CI: `39499020338`).
  - **Phase R1.3B Stage B Implementation**: COMPLETED on dedicated branch `phase-r1/r1-3b-brand-implementation`.
- **Completed Deliverables in R1.3B Stage B**:
  - `B1`: Spatial Studio Brand Alignment (Direction A+ 3D monoliths with 8° tilt, specular rim, perspective floor reflection, 0/2/6 project count behavior, bottom AI command capsule).
  - `B2`: Projects Workspace (Real persistent metadata listing, search input, status filter pills, safe lifecycle controls).
  - `B3`: Services & Runtime (Nginx port 8080 process ownership verification, PHP-FPM socket pool, MariaDB engine, grounded process identity panel).
  - `B4`: Storage & Volumes (APFS Tier-1 `/Volumes/Mac Harici Disk`, internal container, persistent UUIDs, disk usage gauges, VolumeBarrier guard).
  - `B5`: Detailed Project Inspector (Slide-over drawer, project metadata, database deletion strictly disabled with lock guard, SFTP dry-run only guard).
  - `B6`: Welcome / First-Run Experience (Official Apple squircle app icon, quickstart onboarding cards, dismissible onboarding).
  - `Icons`: Official macOS app icons generated into `apps/desktop/src-tauri/icons/` (`icon.icns`, `icon.ico`, etc.).
  - `Verification`: 57 Rust tests + 16 frontend automated tests (73/73 PASS); clean TypeScript build; 9 verified high-resolution PNG screenshots captured and packaged into `docs/design/r1_3b/localforge_r1_3b_stage_b_verified_screenshots.zip`.
- **Next Permitted Action**: Submit R1.3B Stage B implementation report with verified PNG screenshots and test results to Product Owner. Await visual & functional acceptance. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-brand-implementation`; never merge into `main` without PO approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **Database Deletion Lock**: Database deletion remains strictly disabled in the UI with `DatabaseDeletionUnavailable` guard.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
