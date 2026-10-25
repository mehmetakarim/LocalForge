# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Execution Tooling Verified — Actual Activation Strictly on Hold / Awaiting PO Explicit Activation Authorization**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `66303e42907675f2e41f8809f73dd31dd59f3ca9`
- **Previous Milestone Status**:
  - **Gate U5.F Final Execution Proposal**: Corrected proposal accepted; execution held for blocker remediation.
- **Completed in Gate U5.F Execution Blocker Remediation**:
  1. `P0 — Missing Production Migration Entrypoint`: Implemented `localforge storage migrate-v2` in `tools/cli/src/main.rs`. Early intercept before `open_metadata_store()` prevents implicit write locks or auto-migrations. Default mode is read-only `--dry-run` opening `file:<path>?immutable=1` with zero write locks and zero WAL side effects. Gated live mode `--execute` strictly requires `--approval-token` and `--backup-path`, verifies SQLite quiescence, executes `migrate_to_v2_loopback_ports()` and `update_project_loopback_port()`, preserves Projetest PHP 8.4, and provides automated rollback into timestamped `quarantine_<timestamp>/` (`chmod 700`).
  2. `P1 — Stopped Project Semantics`: Validated that stopped projects produce a single server block with `return 503 ...` without health endpoints. Clarified that `Projetest` returns `HTTP 503 Service Unavailable` for both `/__localforge_health` and `/` while stopped.
  3. `P1 — Backup and Quarantine Allowlist`: Replaced wildcard matching with exact allowlist: `localforge.db`, `localforge.db-wal`, `localforge.db-shm` with UID 501 check and symlink rejection.
  4. Tested and verified on real host (dry-run, 0 modifications made) and disposable copies (full migration and rollback verified). All 69 Rust tests, 15 runtime integration tests, and 33 desktop tests pass with 0 clippy warnings.
- **Next Permitted Action**: Await explicit Product Owner activation authorization specifically for Gate U5.F execution before making ANY persistent modifications to `localforge.db`, writing vhosts, or signaling Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
