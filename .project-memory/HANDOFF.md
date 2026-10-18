# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.1A — Targeted Acceptance Remediation Completed**
- **Active Git Branch**: `phase-r1/r1-1a-runtime-remediation`
- **Target Remote**: `origin/phase-r1/r1-1a-runtime-remediation`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: CONDITIONALLY ACCEPTED (subject to R1.1A acceptance remediation).
  - **Phase R1.1A**: COMPLETED & FULLY VERIFIED (26/26 automated tests pass across 8 crates, zero Clippy warnings, live HTTP 200 FastCGI verified on external APFS volume).
- **Completed Tasks in R1.1A**:
  - `TASK-R1.1A-01`: Eliminated false `RUNNING` status; separated infrastructure health from application routing status; 502/504 FastCGI gateway errors produce `Degraded`.
  - `TASK-R1.1A-02`: Hardened process identity and ownership verification; eliminated unsafe basename fallback; checked process UID; prevented signaling own process group.
  - `TASK-R1.1A-03`: Honest local URL reporting (`http://127.0.0.1:8080`); marked `.test` resolver as pending across CLI and Desktop UI; added 1-click curl copy action.
  - `TASK-R1.1A-04`: Added negative security tests and regression tests covering probe classification, dead sockets, and process spoofing.
- **Next Permitted Action**: Present R1.1A acceptance report to Product Owner and await explicit authorization for Phase R1.2 (MariaDB & Storage Resilience).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.2 (MariaDB) or Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
