# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.2 — MariaDB Integration, Storage Resilience & Multi-Project Verification Completed**
- **Active Git Branch**: `phase-r1/r1-2-database-and-resilience`
- **Target Remote**: `origin/phase-r1/r1-2-database-and-resilience`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Recovery Phase R1.0**: ACCEPTED by Product Owner.
  - **Phase R1.1**: ACCEPTED by Product Owner.
  - **Phase R1.1A**: ACCEPTED by Product Owner.
  - **Phase R1.2**: COMPLETED & FULLY VERIFIED (32/32 automated tests pass across 8 crates, zero Clippy warnings, live PHP PDO FastCGI on external APFS volume verified, 6 MariaDB integration tests pass).
- **Completed Tasks in R1.2**:
  - `TASK-R1.2-01`: Implemented `MariaDbSupervisor` with Darwin process identity and unprivileged Homebrew `mariadbd` lifecycle.
  - `TASK-R1.2-02`: Implemented internal datadir isolation (`~/Library/Application Support/LocalForge/mariadb/data/`) and private Unix socket.
  - `TASK-R1.2-03`: Implemented authenticated SQL health check (`SELECT 1;`) with latency measurement; never report RUNNING solely because port is open.
  - `TASK-R1.2-04`: Implemented database/user provisioning, strict identifier validation, and drop confirmation token guard.
  - `TASK-R1.2-05`: Implemented database backup (`mariadb-dump`) with `0o600` permissions and database restore.
  - `TASK-R1.2-06`: Verified end-to-end PHP PDO connectivity on external APFS storage with data persistence across PHP restart.
  - `TASK-R1.2-07`: Verified multi-project concurrency on shared Nginx with independent FastCGI sockets and database users.
  - `TASK-R1.2-08`: Verified `VolumeBarrier` disconnect guard and zero fallback writes to internal disk.
- **Next Permitted Action**: Present R1.2 acceptance report to Product Owner and await explicit authorization for Phase R1.3 (Spatial Studio UI).

---

## 2. Invariants & Guardrails

1. **Strict Phase Gate Discipline**: Do NOT begin Phase R1.3 (Spatial Studio UI) without explicit Product Owner approval.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `DeploymentNotImplemented`.
3. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
4. **Verified Process Identity**: All process terminations enforce Darwin kernel start timestamp and binary path matching before signaling.
5. **No False Claims**: Browser `.test` resolution remains explicitly marked as pending dedicated network resolver configuration; loopback access verified via Host header at `127.0.0.1:8080`.
6. **Internal DB Decoupling**: Database files stay on internal APFS storage; user project files stay on external storage.
