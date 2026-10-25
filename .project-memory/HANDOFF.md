# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Final Executor Safety Patch Completed — Actual Activation Strictly on Hold / Awaiting PO Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `4d2bac7c14401903c399f7201ec60cea88acc70d`
- **Previous Milestone Status**:
  - **Gate U5.F Execution Blocker Remediation**: CLI migration entrypoint implemented; safety patch applied for authorization bypass, rollback scope isolation, and WAL snapshot preflight.
- **Completed in Gate U5.F Final Executor Safety Patch**:
  1. `P0-01 — Approval Authorization Bypass Remediation`: Implemented `crates/storage/src/approval.rs`. Replaced arbitrary string acceptance with RFC 2104 / RFC 4231 HMAC-SHA256 signature verification over canonical payload binding `token_id`, `database_path`, `database_sha256`, `scope` (`migrate-v2-loopback-ports`), and exact `projects` (`proj_e92e56cd...: 18081`, `proj_db5472bb...: 18082`). Implemented persistent replay ledger (`consumed_approvals.jsonl` at 0600 permissions). Zero hardcoded secrets (reads `LOCALFORGE_PO_APPROVAL_KEY`, fails closed if missing).
  2. `P0-02 — SQLite Rollback With Open Connection Remediation`: Implemented `crates/storage/src/rollback.rs`. Divided execution and rollback into distinct ownership scopes (`MetadataStore` connection is dropped before rollback begins). Verifies writer quiescence immediately prior to restoration (`verify_writer_quiescence`), failing closed if active SQLite processes exist. Quarantines failed live state into `quarantine_<timestamp>/` (0700) using exact allowlist (`localforge.db`, `localforge.db-wal`, `localforge.db-shm`). Restores backup atomically (0600) and independently verifies integrity and records. Simulated failure injection tested across multiple failure points.
  3. `P1-01 — WAL-Aware Read-Only Preflight Remediation`: Implemented `crates/storage/src/snapshot.rs`. Uses SQLite Online Backup API against live database into an isolated temporary file to obtain a consistent snapshot without forcing dirty WAL flushes or mutating live WAL/SHM file metadata. Runs preflight schema and project record inspection against the snapshot.
  4. Tested and verified: Real-host dry-run confirmed 0 modifications and valid WAL snapshot. 18 storage unit tests, 15 runtime integration tests, 79 workspace library tests, 33 desktop frontend tests pass (112 total). Zero clippy warnings, zero fmt diffs.
- **Next Permitted Action**: Await explicit Product Owner acceptance and subsequent activation authorization specifically for Gate U5.F execution before making ANY persistent modifications to `localforge.db`, writing vhosts, or signaling Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
