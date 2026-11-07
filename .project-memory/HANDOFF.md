# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F-M Authorized SQLite V1 Backup Complete — Live Migration Awaiting Separate Product Owner Authorization**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `1b78a6129226445961e51f756a40c6203e384709`
- **Previous Milestone Status**:
  - **Gate U5.F Trust Anchor Enrollment Closeout**: Enrolled trust anchor verified, threat boundary documented (Commit `1b78a61`).
- **Completed in Gate U5.F-M Authorized SQLite V1 Backup Execution**:
  1. `Quiescence & Source Preflight`:
     - Verified source database `/Users/mehmetakar/Library/Application Support/LocalForge/localforge.db` (12288 bytes, UID 501 `mehmetakar`).
     - Verified 0 open file handles (`lsof`) and 0 running `localforge` processes (`pgrep`).
     - Verified live schema: Schema V1 (`loopback_port` absent), integrity `ok`, exactly 2 projects (Projetest PHP 8.4 preserved, r1_acceptance_project PHP 8.5).
  2. `SQLite Online Backup Creation`:
     - Created unique backup directory: `/Users/mehmetakar/Library/Application Support/LocalForge/backups/u5f_v1_backup_20261107_225212/` (`0700`, UID 501).
     - Executed SQLite Online Backup API via `sqlite3 -readonly ... ".backup ..."` creating `localforge_v1_atomic_backup.db` (`0600`, UID 501, 12288 bytes).
  3. `Independent Backup Validation`:
     - `PRAGMA integrity_check;` equals `ok`.
     - Exactly 2 projects present; Project IDs and metadata match snapshot 100%.
     - Projetest PHP version equals `8.4` (PO-U5-01 PRESERVED).
     - `loopback_port` column absent (clean Schema V1).
  4. `Manifest Generation & Validation`:
     - Calculated SHA-256: `2832b96f7a69fbb56d866cbe9bc259d379a6582a777bd45a333f161496f1e694`.
     - Created `manifest.sha256` (`0600`, UID 501); validated with `shasum -a 256 -c`.
  5. `Safety Invariants Strictly Maintained`:
     - Live source database: 0 writes, timestamp and size unchanged.
     - Active WAL/SHM files: 100% preserved.
     - PO private signing key: Never accessed or read.
     - Zero approval tokens minted; zero `--execute` invocations; zero port allocations.
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
