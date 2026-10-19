# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.3A-V1 — Final Functional Acceptance Fixes Completed**
- **Active Git Branch**: `phase-r1/r1-3a-v1-functional-acceptance-fixes`
- **Target Remote**: `origin/phase-r1/r1-3a-v1-functional-acceptance-fixes`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: COMPLETED (Direction A+ visual baseline).
  - **Phase R1.3A-V1**: COMPLETED & VERIFIED (Production fixture isolation, verified health messaging, synchronization failure propagation, and >6 project spatial pagination).
- **Completed Tasks in R1.3A-V1**:
  - `TASK-R1.3A-V1-01`: Isolated fixtures into `visualFixtures.ts` with strict production exclusion gate (`import.meta.env.PROD`).
  - `TASK-R1.3A-V1-02`: Created `computeHeroHealthContent` checking volumes, Nginx 8080, MariaDB, probe failures, and real FPM pool state.
  - `TASK-R1.3A-V1-03`: Propagated sync errors in `sync_projects` and updated failed project states to `Unknown` in SQLite.
  - `TASK-R1.3A-V1-04`: Multi-project spatial pagination for >6 projects with truthful total project count and keyboard arrow navigation.
  - `TASK-R1.3A-V1-05`: Verified with 50 Rust tests and 11 frontend automated regression tests (61/61 PASS).
- **Next Permitted Action**: Submit R1.3A-V1 Acceptance Report to Product Owner. Await explicit authorization before proceeding to Phase R1.3B.

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
