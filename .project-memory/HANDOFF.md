# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F-M Final Pre-Execution Check Complete — Ready for Explicit Live Migration Decision**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `abc711684c7d0d00fca18c219660c679b32e652a`
- **Previous Milestone Status**:
  - **Gate U5.F-M Authorized SQLite V1 Backup Execution**: Verified V1 backup created and SHA-256 manifest recorded (Commit `abc7116`).
- **Completed in Gate U5.F-M Final Pre-Execution Check**:
  1. `Revalidated Backup`:
     - Path: `/Users/mehmetakar/Library/Application Support/LocalForge/backups/u5f_v1_backup_20261107_225212/localforge_v1_atomic_backup.db`
     - Permissions: directory `0700`, file `0600`, manifest `0600`.
     - Checksum: `2832b96f7a69fbb56d866cbe9bc259d379a6582a777bd45a333f161496f1e694` verified OK.
     - Integrity: `PRAGMA integrity_check;` returned `ok`.
  2. `Snapshot vs Backup Comparison`:
     - `Schema V1`: 100% match.
     - Project Records: 100% match on IDs, names, paths, UUIDs, docroots, local domains, and PHP versions.
     - `loopback_port`: absent on both (0 count). Zero material differences found.
  3. `Port Availability Recheck`:
     - Ports 18081 and 18082 confirmed completely free (0 listeners).
  4. `Trust Anchor Verification`:
     - Key ID: `po-ed25519-7374d1100bee`, fingerprint `7374d1100bee06a5364c0e193c485308065998ca65b3ac8822c25f8e52f42991` unchanged.
  5. `Token Generation Fixture Testing`:
     - Corrected token-generation script to read actual backup SHA-256 from manifest file.
     - Added unit test `test_external_openssl_python_token_signature_verification` in `crates/storage/src/approval.rs`.
     - Verified that canonical bytes generated via Python `json.dumps(payload, separators=(',', ':'))` signed via OpenSSL CLI (`openssl pkeyutl -sign -rawin`) verify cleanly in `localforge_storage::verify_and_consume_approval`.
  6. `Exact Operator Commands & Rollback Instructions`:
     - Formulated exact Python + OpenSSL script reading manifest file directly.
     - Formulated exact CLI migration command with `@/path/to/token.json` and backup path.
     - Documented both automated connection-isolated rollback and manual fallback procedure.
  7. `Safety Invariants Maintained`:
     - Live migration strictly NOT executed.
     - Zero real approval tokens minted; zero private key access.
     - Zero live SQLite file mutations; zero Nginx reload; zero PHP/MariaDB installs; zero main merges.
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
