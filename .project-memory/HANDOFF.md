# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Final Authorization & Recovery Safety Closeout Completed — Actual Activation Strictly on Hold / Awaiting PO Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `da26863dad9a7c9ddfc9bf3bc7f409b8cf53e56d`
- **Previous Milestone Status**:
  - **Gate U5.F Executor Safety Patch**: Implemented initial approval token and rollback connection isolation; conditionally accepted by PO.
- **Completed in Gate U5.F Final Authorization & Recovery Closeout**:
  1. `Finding 01 — Asymmetric Trust Separation`: Implemented Ed25519 verifier-only public key architecture in `crates/storage/src/approval.rs`. The runtime executor holds only `LOCALFORGE_PO_PUBLIC_KEY`. The private key remains strictly offline with the Product Owner. Agents cannot self-approve.
  2. `Finding 02 — Cross-Process Atomic Replay Protection`: Protected `consumed_approvals.jsonl` with exclusive POSIX `flock(fd, LOCK_EX)`. Atomically checks replay, validates non-corrupted lines, appends new record, and forces durability via `sync_all()`. Verified with 8 concurrent racing threads.
  3. `Finding 03 — Staged Fail-Closed SQLite Rollback`: Staged temporary restore file validated prior to live file modification; non-swallowing error handling (Projetest missing or mutated PHP 8.4 fails closed); dual-layer writer quiescence (`lsof -t` on DB/WAL/SHM + `pgrep`); 0700 quarantine preservation.
  4. `Finding 04 — Signed Snapshot Identity & Metadata Assertions`: Explicitly labeled `activation_snapshot_sha256`; bound rich project assertions (`id`, `name`, `path`, `volume_uuid`, `document_root`, `php_version`, `local_domain`, `assigned_loopback_port`); strict PHP 8.4 verification.
  5. Tested and verified: Real-host dry-run confirmed 0 modifications and valid WAL snapshot. Disposable drill completed with 100% precision. 20 storage unit tests, 15 runtime integration tests, 81 workspace library tests, 33 desktop frontend tests pass (114 total). Zero clippy warnings, zero fmt diffs.
- **Next Permitted Action**: Await explicit Product Owner acceptance and subsequent activation authorization specifically for Gate U5.F execution before making ANY persistent modifications to `localforge.db`, writing vhosts, or signaling Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
