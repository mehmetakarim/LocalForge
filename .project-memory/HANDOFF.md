# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B Stage B Final Acceptance Remediation — Completed & Awaiting Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-brand-implementation`
- **Target Remote**: `origin/phase-r1/r1-3b-brand-implementation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **Phase R1.3B Brand Integration Stage A**: APPROVED by Product Owner (Baseline: `9c9d720b46c9676f5023f40f1660c92dc90dad5d`, CI: `39499020338`).
  - **Phase R1.3B Stage B Implementation**: COMPLETED on dedicated branch `phase-r1/r1-3b-brand-implementation`.
- **Completed Remediations in R1.3B Stage B**:
  - `Finding 1 (P0)`: Re-captured all 9 verified PNG screenshots via Playwright Chrome, resolving empty state bug caused by unhandled `fixture=default` alias in `visualFixtures.ts` and hardened `inspector` URL parameter. Packaged in `docs/design/r1_3b/localforge_r1_3b_stage_b_verified_screenshots.zip`.
  - `Finding 2 (P1)`: Added `default_server` loopback 404 in Nginx; implemented `check_domain_resolution` in Tauri; created `RoutingAlertModal.tsx` in desktop frontend with copyable cURL helper to prevent wrong-project routing.
  - `Finding 3 (P1)`: Removed `: 45;` fabricated capacity fallback in `StorageVolumes.tsx`; implemented `hasValidCapacity` check; unreadable/zero/disconnected volumes render `BİLİNMİYOR` / `KAPASİTE OKUNAMADI`.
  - `Finding 4 (P1)`: Refactored `ServicesRuntime.tsx` to explicitly preserve UNKNOWN for unprobed Nginx, PHP-FPM, and MariaDB rather than collapsing into false/stopped.
  - `Finding 5 (P1)`: Grounded operational claims in Deployments (`DeploymentNotImplemented` dry-run only), MCP (stdio ready), and Settings (dynamic ports/paths).
  - `Finding 6 (P2)`: Created `ImportProjectModal.tsx` and wired "Mevcut Projeyi Aç" in `WelcomeSplash.tsx` to invoke Tauri `import_project` command.
  - `Verification`: 58 Rust tests + 19 frontend automated tests (77/77 PASS); clean TypeScript build; zero clippy warnings; zero formatting diffs.
- **Next Permitted Action**: Submit R1.3B Stage B final acceptance remediation report with verified PNG screenshots and test results to Product Owner. Await visual & functional acceptance. Do NOT merge into `main`. Do NOT begin R1.3C.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-brand-implementation`; never merge into `main` without PO approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **Database Deletion Lock**: Database deletion remains strictly disabled in the UI with `DatabaseDeletionUnavailable` guard.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
