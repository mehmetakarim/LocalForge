# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.3A — Spatial Studio Foundation & Application Shell Completed**
- **Active Git Branch**: `phase-r1/r1-3a-spatial-studio-foundation`
- **Target Remote**: `origin/phase-r1/r1-3a-spatial-studio-foundation`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: ACCEPTED by Product Owner.
  - **Phase R1.1A**: ACCEPTED by Product Owner.
  - **Phase R1.2**: ACCEPTED by Product Owner.
  - **Phase R1.2A**: ACCEPTED by Product Owner.
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A**: COMPLETED & VERIFIED (Spatial Studio design system, floating dock, 2.5D cards, Cmd+K capsule, embedded dist frontend).
- **Completed Tasks in R1.3A**:
  - `TASK-R1.3A-01`: Spatial Studio design system tokens and atmospheric void styling (`index.css`).
  - `TASK-R1.3A-02`: 2.5D `SpatialCard` with micro-tilt, live metrics, status halos, and curl command helper.
  - `TASK-R1.3A-03`: `FloatingDock` navigation (`⌘1..7`) and `AICommandCapsule` modal dialog (`⌘K`).
  - `TASK-R1.3A-04`: Embedded production frontend distribution (`devUrl: null`), eliminating sandbox socket network issues.
  - `TASK-R1.3A-05`: Verified real macOS window captures (`docs/screenshots/spatial_studio_final.png`, `spatial_studio_cmd_k.png`).
- **Next Permitted Action**: Present R1.3A visual acceptance report to Product Owner and await authorization for Phase R1.3B (Native Workspace & Detailed Inspector).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
