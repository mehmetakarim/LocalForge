# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B UAT Failure Recovery — Completed & Awaiting Acceptance**
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
- **Completed in R1.3B UAT Failure Recovery**:
  - `Gate 2A — PHP-FPM Compatibility Recovery`: Replaced hardcoded PHP version choices with dynamic runtime telemetry (`PhpVersionDetail`); verified binary (`fpm-fcgi`) and architecture (`arm64`); added project-level confirmed version switcher dialog; persisted safely to SQLite; preserved `Projetest` at PHP 8.4 intact; provided Turkish missing runtime guidance.
  - `Gate 2B — MariaDB Lifecycle Recovery`: Connected safe Start/Stop controls in `ServicesRuntime.tsx`; enforced safety guard refusing automatic datadir initialization on host Application Support storage without PO approval; displayed verified service state and datadir status honestly.
  - `Gate 2C — Browser Routing Diagnostics`: Enhanced `RoutingAlertModal.tsx` with a diagnostic pipeline matrix (Nginx listener, VHost configuration, PHP-FPM pool status, macOS DNS NXDOMAIN); provided copyable `/etc/hosts` and cURL commands; documented loopback port vs RFC 6761 options.
  - `Gate 3 — Native macOS Visual Recovery`: Generated official Apple squircle app icon geometry with transparent corners `(0,0)` and compiled native `icon.icns` via `iconutil`; eliminated duplicate HTML traffic lights; hid zoom controls when inspector is open; fixed storage viewport scroll hierarchy (`h-screen max-h-screen`, `justify-start min-h-0`); displayed verified available capacity gauge (`Boş: ... (%... dolu)`); converted `ErrorState` to non-layout-shifting floating HUD notification banner.
  - `Gate 4 — Verification Suite & Evidence Capture`: Captured 11 verified full-resolution screenshots from running native Cocoa Tauri app (`target/debug/localforge-desktop`) backed by real SQLite database (`localforge.db`) with 2 persistent projects on `/Volumes/Mac Harici Disk`; packaged into `docs/design/r1_3b/localforge_r1_3b_native_tauri_screenshots.zip`; 26/26 frontend tests PASS; 58/58 Rust workspace tests PASS (84/84 total); 0 clippy warnings; 0 format diffs; clean Vite build.
- **Next Permitted Action**: Submit R1.3B UAT Recovery Report to Product Owner. Await acceptance. Do NOT merge into `main`. Do NOT begin R1.3C.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
