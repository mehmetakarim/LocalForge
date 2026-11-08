# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F-M Final Signature Interoperability & Execution Readiness Complete**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `05dea484661efde2ce678d1f1a1219b69e554186` (CI Run `42429156267` SUCCESS)
- **Previous Milestone Status**:
  - **Gate U5.F-M Authorized SQLite V1 Backup Execution**: Verified V1 backup created and SHA-256 manifest recorded (Commit `abc7116`).
  - **Gate U5.F-M Final Pre-Execution Verification**: Preflight checks passed; disposable fixture test added (Commit `3af4f9d`).
- **Completed in Gate U5.F-M Final Signature Interoperability & Execution Readiness**:
  1. `Task 1 (10-Point Ed25519 Interoperability Test)`:
     - Implemented `test_task1_disposable_ed25519_python_rust_interoperability` in `crates/storage/src/approval.rs`.
     - Verified exact byte-for-byte equality between Python `json.dumps(..., separators=(',', ':'))` and Rust `MigrationApprovalPayload::to_canonical_bytes()` (PASS).
     - Verified Python-signed token against Rust `ed25519_dalek` (PASS).
     - Verified all 5 negative security conditions: wrong signature, port mismatch, sha256 mismatch, expired token, unauthorized scope (ALL PASS).
     - Verified deterministic project sorting by ID (PASS).
     - Built-in OpenSSL 4.0.3 CLI + RFC 8032 pure-Python fallback ensuring 100% cross-platform test reliability.
  2. `Task 2 (Backup & Live Readiness)`:
     - Backup SHA-256 verified against manifest: `2832b96f7a69fbb56d866cbe9bc259d379a6582a777bd45a333f161496f1e694` (PASS).
     - Backup integrity `PRAGMA integrity_check` = `ok`; Schema V1 verified.
     - Projects verified: `proj_e92e...` (8.5), `proj_db54...` (8.4 - PO-U5-01 PRESERVED).
     - Trust anchor fingerprint verified: `7374d1100bee06a5364c0e193c485308065998ca65b3ac8822c25f8e52f42991`.
     - Ports 18081 and 18082 reconfirmed free (0 listeners).
     - Zero unexpected SQLite writers; zero localforge processes.
     - Read-only preflight dry-run completed with zero modifications.
  3. `Task 3 (Human Execution Package)`:
     - Fully structured operator package with exact backup path, production payload, Python signing script, NOT AUTHORIZED execution command, post-migration verification, and emergency rollback.
  4. `Safety Invariants Maintained`:
     - Live migration strictly NOT executed.
     - Zero real approval tokens minted; zero private key access.
     - Zero live SQLite mutations; zero Nginx reload; zero PHP/MariaDB installs; zero main merges.
- **Next Permitted Action**: Await explicit Product Owner decision authorizing live database migration. Real migration remains STRICT NO-GO until that explicit decision is communicated.
     - Zero Nginx reload, PHP 8.4 installation, MariaDB init, DNS/hosts changes, or git merge.
- **Next Permitted Action**: Await explicit, separate Product Owner authorization for live migration execution with an independently minted approval token. Real migration remains STRICT NO-GO until that explicit authorization is provided.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
