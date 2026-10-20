# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **R1.3B Preparation — Completed & Reliability Review / Visual Proposals Awaiting Approval**
- **Active Git Branch**: `phase-r1/r1-3b-preparation`
- **Target Remote**: `origin/phase-r1/r1-3b-preparation`
- **Previous Milestone Status**:
  - **Phase R1.2B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V Stage A**: ACCEPTED by Product Owner (Direction A+ Approved).
  - **Phase R1.3A-V Stage B**: ACCEPTED by Product Owner.
  - **Phase R1.3A-V1 Final Acceptance Patch**: ACCEPTED by Product Owner.
  - **R1.3B Preparation**: COMPLETED & VERIFIED (Nginx process ownership via `find_listening_pids` and sysctl cmdline config inspection, SQLite persistent metadata recovery guard & dynamic service reconnection, metadata query error propagation, 55 Rust + 16 frontend tests pass, database deletion strictly excluded from R1.3B).
- **Completed Tasks in R1.3B Preparation**:
  - `TASK-R1.3B-PREP-01`: Verified Nginx ownership against listening PIDs, legitimate binary paths, and config directory matches; reports port conflicts for unverified occupants or unrelated listeners. Adversarial test verified.
  - `TASK-R1.3B-PREP-02`: Removed `list_projects().unwrap_or_default()`, propagating database errors explicitly as UNKNOWN or error responses.
  - `TASK-R1.3B-PREP-03`: Removed silent `open_in_memory()` fallback; isolated temporary store; dynamic reconnection of `ProjectService` and `ProjectRuntimeService` to recovered persistent database without stale references. Recovery test verified.
  - `TASK-R1.3B-PREP-04`: Verified with 55 Rust tests and 16 frontend automated regression tests (71/71 PASS).
- **Next Permitted Action**: Submit reliability findings, adversarial test results, scope lock, and high-fidelity Direction A+ visual proposals for the 4 core views to Product Owner. Await explicit review and authorization before proceeding with R1.3B code implementation.

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
