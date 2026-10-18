# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.2A — Database Security Remediation Completed**
- **Active Git Branch**: `phase-r1/r1-2a-database-security`
- **Target Remote**: `origin/phase-r1/r1-2a-database-security`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: ACCEPTED by Product Owner.
  - **Phase R1.1A**: ACCEPTED by Product Owner.
  - **Phase R1.2**: CONDITIONALLY ACCEPTED (Functional progress accepted; security remediation authorized as R1.2A).
  - **Phase R1.2A**: COMPLETED & FULLY VERIFIED (37/37 automated tests pass across 8 crates, zero Clippy warnings, 11 MariaDB integration & adversarial security tests pass).
- **Completed Tasks in R1.2A**:
  - `TASK-R1.2A-01`: SQL credential escaping (`escape_sql_string_literal`), null byte rejection (`\0`), length constraints, and password redaction from stderr logs.
  - `TASK-R1.2A-02`: Secure atomic backup streaming with `0o600` creation permissions, symlink defense, overwrite protection, and non-empty SQL header verification.
  - `TASK-R1.2A-03`: Strict elimination of deletion self-authorization (`db_name == approval_token` fails closed) and single-use cryptographic UUIDv4 token with 60s TTL.
  - `TASK-R1.2A-04`: Safe restore with non-empty database protection (`information_schema.tables > 0`), streaming stdin, and header integrity checks.
  - `TASK-R1.2A-05`: Datadir ownership validation (`uid == geteuid()`), unknown existing directory guard, and graceful shutdown (`mysqladmin` & `SIGTERM` before `SIGKILL`).
  - `TASK-R1.2A-06`: Comprehensive adversarial security test suite (5 new tests in `mariadb_integration_test.rs`).
- **Next Permitted Action**: Present R1.2A acceptance report to Product Owner and await explicit authorization for Phase R1.3 (Spatial Studio UI).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage; user project files stay on external storage.
