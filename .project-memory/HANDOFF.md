# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.2B — Final Database Safety Gate Completed**
- **Active Git Branch**: `phase-r1/r1-2b-final-database-safety`
- **Target Remote**: `origin/phase-r1/r1-2b-final-database-safety`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: ACCEPTED by Product Owner.
  - **Phase R1.1A**: ACCEPTED by Product Owner.
  - **Phase R1.2**: CONDITIONALLY ACCEPTED.
  - **Phase R1.2A**: COMPLETED & VERIFIED.
  - **Phase R1.2B**: COMPLETED & FULLY VERIFIED (40/40 automated tests pass across 8 crates, zero Clippy warnings, 13 MariaDB integration tests pass).
- **Completed Tasks in R1.2B**:
  - `TASK-R1.2B-01`: Deletion request and approval decision separation (`DatabaseDeletionRequest`), self-authorization via `db_name` and `request_id` strictly eliminated, deletion explicitly disabled in Phase R1.2B pending UI gate (`DatabaseDeletionUnavailable`).
  - `TASK-R1.2B-02`: Fail-closed database restore without `if let Ok(...)`, non-empty destination rejection, and diagnostic partial restore tracking with DDL implicit commit documentation.
  - `TASK-R1.2B-03`: Darwin Application Support resolution via `$HOME` and `libc::getpwuid(libc::getuid())`; eliminated all hardcoded `/Users/mehmetakar` fallbacks across runtime, desktop app, CLI, and MCP server.
  - `TASK-R1.2B-04`: 13 MariaDB integration tests and 40 workspace tests executed and verified.
- **Next Permitted Action**: Present R1.2B final safety report to Product Owner and await explicit authorization for Phase R1.3 (Spatial Studio UI).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage (`Library/Application Support/LocalForge/mariadb/`); user project files stay on external storage.
