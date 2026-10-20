# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B Visual Design Gate & Narrow Reliability Review — Completed & Awaiting Approval**
- **Active Git Branch**: `phase-r1/r1-3b-preparation`
- **Target Remote**: `origin/phase-r1/r1-3b-preparation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **R1.3B Preparation**: CONDITIONALLY ACCEPTED by Product Owner.
  - **R1.3B Visual Design Gate**: COMPLETED (Exact path matching in `is_localforge_nginx_cmdline`, substring spoofing rejection, fail-closed multi-listener conflict handling, 4 high-fidelity visual prototypes & rendered PNGs in `docs/design/r1_3b/`, full data capability mapping, 57 Rust + 16 frontend tests pass).
- **Completed Tasks in R1.3B Gate**:
  - `TASK-R1.3B-PREP-01`: Exact canonical path matching for Nginx config and prefix, elimination of substring matching, fail-closed multi-listener conflict handling.
  - `TASK-R1.3B-PREP-04`: Verified with 57 Rust tests and 16 frontend automated tests (73/73 PASS).
  - `TASK-R1.3B-GATE-01`: 4 1600x1000 Direction A+ HTML prototypes and rendered PNG screenshots in `docs/design/r1_3b/`.
  - `TASK-R1.3B-GATE-02`: Comprehensive Data Capability Mapping and bounded log specification.
- **Next Permitted Action**: Submit Visual Design Gate report with PNG screenshots and data capability mapping to Product Owner. Await explicit review and selection before proceeding with R1.3B code implementation.

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
