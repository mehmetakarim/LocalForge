# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.1 — Real PHP-FPM & Nginx Runtime Integration Completed**
- **Active Git Branch**: `phase-r1/r1-1-runtime-implementation`
- **Target Remote**: `origin/phase-r1/r1-1-runtime-implementation`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: COMPLETED & FULLY VERIFIED (25/25 automated tests pass, zero Clippy warnings, live HTTP 200 FastCGI execution on external APFS volume).
- **Completed Tasks in R1.1**:
  - `TASK-R1.1-01`: Unprivileged PHP-FPM child process spawning, isolated pool configs, 104-byte Unix socket path limit mitigation.
  - `TASK-R1.1-02`: Shared unprivileged Nginx loopback gateway (`127.0.0.1:8080`), modular virtual host generation (`conf.d/{id}.conf`), atomic `nginx -t` validation before activation/reload.
  - `TASK-R1.1-03`: Darwin kernel verified process identity (`libc::proc_pidinfo` / `proc_bsdinfo` timestamp + binary canonical path), PID reuse race condition defense, graceful termination.
  - `TASK-R1.1-04`: End-to-end loopback HTTP 200 FastCGI verification, dynamic JSON response payload (`PHP 8.5.3`), multi-project isolation test (`test_r1_1_06_multi_project_isolation`).
- **Next Permitted Action**: Present R1.1 acceptance report to Product Owner and await explicit authorization for Phase R1.2 (MariaDB & Storage Resilience).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.2 (MariaDB) or Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
