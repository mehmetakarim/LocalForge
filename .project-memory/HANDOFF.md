# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.3A-V Stage B — Spatial Studio Visual Recovery Completed**
- **Active Git Branch**: `phase-r1/r1-3a-v-spatial-visual-implementation`
- **Target Remote**: `origin/phase-r1/r1-3a-v-spatial-visual-implementation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: COMPLETED & VERIFIED (Faithful reproduction of visual truth reference, vertical monoliths, floor reflections, Project Inspector drawer, zero fabricated data, 5 scenario screenshots captured).
- **Completed Tasks in R1.3A-V Stage B**:
  - `TASK-R1.3A-V-01`: Stage A High-Fidelity Direction A+ Proposal & PO Approval.
  - `TASK-R1.3A-V-02`: Vertical glass monoliths (`SpatialCard.tsx`) with `-webkit-box-reflect` and 3D perspective stage (`StudioOverview.tsx`).
  - `TASK-R1.3A-V-03`: Slide-over Project Inspector drawer (`ProjectInspector.tsx`) housing all technical telemetry and diagnostic actions.
  - `TASK-R1.3A-V-04`: Zero fabricated production data guarantee in `App.tsx` (genuine empty states, honest `UNKNOWN`/`UNAVAILABLE` status tokens).
  - `TASK-R1.3A-V-05`: High-resolution screen captures across 0, 2, 6 projects, inspector and responsive mode (`docs/screenshots/spatial_studio_v_*.png`).
- **Next Permitted Action**: Submit R1.3A-V Stage B Acceptance Report to Product Owner. Await explicit authorization before proceeding to Phase R1.3B.

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
