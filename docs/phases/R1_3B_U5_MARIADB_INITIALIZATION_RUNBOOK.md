# LocalForge — Gate U5 MariaDB Safe Initialization Runbook
## Storage Audit, Safety Classification & Approval-Gated Bootstrap Procedure (Reconciled)

**Author:** Principal Software Architect & Database Architect  
**Governance:** Gate U5.A — Architecture Safety Reconciliation & Implementation Readiness  
**Status:** DESIGN & READ-ONLY PREFLIGHT COMPLETED — EXECUTION ON HOLD (STRICT NO-GO)  
**Binding Decision:** `PO-U5-02` (MariaDB Initialization Hold)  
**Date:** 2026-10-22

---

## 1. Executive Summary & Binding Decision PO-U5-02

Per **Decision PO-U5-02**, the LocalForge MariaDB storage subsystem has been inspected in a strictly read-only manner. Although the runtime supervisor correctly reported that the data directory is uninitialized:
- **NO automatic disk initialization has been executed.**
- **NO system tables have been generated or modified.**
- **NO database credentials have been altered.**
- **NO foreign or system MariaDB processes have been touched.**
- **Execution of this runbook requires a separate, explicit Product Owner authorization checkpoint (Gate U5.D).**

> [!CRITICAL]
> **Empty Directory Observation $\neq$ Authorization to Initialize**  
> Observing that a directory on disk contains 0 files is purely an empirical physical fact. It does NOT constitute authorization to write to disk, bootstrap database files, or alter operating system state. Any write operation without explicit user authorization is strictly prohibited.

---

## 2. Non-Destructive Datadir Audit Findings

The audit conducted on **2026-10-22 at 23:10:20+03:00** gathered the following verifiable telemetry:

| Property | Observed State | Audit Evidence |
|---|---|---|
| **Base Directory** | `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb` | Exists on internal SSD |
| **Directory Contents** | Completely empty (only `.` and `..` entries) | Total size 64 bytes (`ls -la` exit 0) |
| **Data Subdirectory (`data/`)** | Does NOT exist | Verified: no `data/` or `mysql/` folder |
| **Run Subdirectory (`run/`)** | Does NOT exist | Verified: no socket or pid file |
| **Configuration (`my.cnf`)** | Does NOT exist | Verified |
| **Filesystem Type** | APFS (`/dev/disk3s5`), Journaled, Case-Insensitive | `df -h` -> `/System/Volumes/Data` |
| **Available Disk Capacity** | 11 GiB available (95% container capacity) | Verified via `df -h` |
| **Directory Ownership** | `mehmetakar:staff` (`drwxr-xr-x`, mode `0755`) | Verified non-root unprivileged ownership |
| **Symlink Status** | Real canonical directory (NOT a symlink) | Checked via `readlink` / `dunce::canonicalize` |
| **Installed Daemon Executable** | `/opt/homebrew/bin/mariadbd` (Ver `13.0.2-MariaDB` arm64) | `/opt/homebrew/bin/mariadbd --version` |
| **Bootstrap Utility** | `/opt/homebrew/bin/mariadb-install-db` | Exists and executable |
| **Port 3306 Occupancy** | **FREE** (No active listeners on 127.0.0.1:3306) | Verified via `lsof -iTCP:3306 -sTCP:LISTEN` |
| **Port 8080 Occupancy** | LocalForge Nginx master/worker (PID 39566/39567) | Verified via `lsof -iTCP:8080 -sTCP:LISTEN` |

---

## 3. Storage Safety Classification

Based on the empirical evidence gathered during the audit, the MariaDB storage subsystem is formally classified as:

### Classification: `VERIFIED EMPTY AND ELIGIBLE FOR INITIALIZATION (HELD)`

- **Observation**: The directory exists within the authorized user Application Support boundary, contains zero files or subdirectories, contains no partial system tables, has no active locks, and is wholly owned by the unprivileged user `mehmetakar`.
- **Constraint**: Because no explicit Product Owner command to initialize has been issued, the safety guard implemented in `crates/runtime/src/mariadb.rs` continues to fail closed:
  > *"MariaDB veri dizini başlatılmamış. Güvenlik politikası ve Product Owner direktifi gereği otomatik disk ilklendirme durdurulmuştur. Lütfen yetkilendirme talep ediniz."*

---

## 4. Reconciled 12-Point Approval-Gated Initialization Runbook

When the Product Owner explicitly authorizes initialization in a future gate (Gate U5.D), the procedure must strictly follow this reconciled runbook.

### Point 01: Preconditions & Revalidation Immediately Before Execution
Immediately prior to any disk mutation, LocalForge must revalidate:
1. Host user is unprivileged (`libc::geteuid() != 0`).
2. Target volume `/System/Volumes/Data` has at least 1.0 GiB free capacity.
3. Target directory `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data` does NOT already exist.
4. Port 3306 is free, or an unprivileged fallback port (e.g. 13306) is designated.
5. `/opt/homebrew/bin/mariadb-install-db` is verified for `arm64` binary architecture.

### Point 02: Private Staging Directory & Ownership Proof
To prevent leaving half-initialized or poisoned state inside the target location:
- LocalForge shall **NEVER initialize directly into the target production directory**.
- LocalForge creates a private, uniquely named staging directory on the **EXACT SAME filesystem**:
  ```
  STAGING_DIR = /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/staging_<uuid>
  ```
- **Ownership & Perm Proof**: Created with mode `0700` (`rwx------`). Inode and canonical path verified via `dunce::canonicalize`. UID checked against `geteuid()`. Symlinks are strictly rejected.

### Point 03: Same-Filesystem Invariant for Atomic Promotion
- The staging directory and the final target directory MUST reside on the exact same APFS filesystem mount point (`/System/Volumes/Data`).
- **Rationale**: POSIX `rename()` / `std::fs::rename()` is atomic **only** when both paths share the same filesystem. Cross-filesystem moves result in a non-atomic copy-and-delete sequence, which violates transaction safety.

### Point 04: Version-Specific Compatibility (MariaDB 13.0.2)
- Installed version: `13.0.2-MariaDB`.
- `mariadb-install-db` flags in version 13:
  - `--auth-root-authentication-method=socket`: Enforces peer credential authentication for `root@localhost`. Only the OS user `mehmetakar` communicating over the local Unix domain socket can connect as administrative root. Remote/TCP root connections are prohibited.
  - `--skip-test-db`: Prevents creating insecure world-writable `test` databases.
  - `--defaults-file`: Points to an isolated, temporary configuration file mode `0600`.

### Point 05: Secure Initial Authentication (Zero Intermediate Unsecured Window)
- **Elimination of Unsecured Window**: Traditional setups initialize root with an empty password, leaving a window of vulnerability before a hardening script runs.
- **LocalForge Mitigation**:
  1. Root is configured with `unix_socket` authentication (`auth_socket` / `auth_pam` on Darwin).
  2. Local TCP loopback requires dedicated service credentials (`localforge_admin`) created during bootstrap via `--init-file` with a high-entropy 256-bit cryptographic secret generated in memory.
  3. No open or passwordless root account is ever exposed, even momentarily.

### Point 06: Execution Inside Staging Directory
Execute `mariadb-install-db` strictly targeting `STAGING_DIR`:
```bash
/opt/homebrew/bin/mariadb-install-db \
  --defaults-file="/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/staging_<uuid>/bootstrap.cnf" \
  --datadir="/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/staging_<uuid>/data" \
  --auth-root-authentication-method=socket \
  --skip-test-db
```

### Point 07: Atomic Promotion Protocol
Only after `mariadb-install-db` exits with code 0 and the internal structure (`mysql/`, `performance_schema/`) passes integrity validation:
1. Verify target path `.../runtime/mariadb/data` still does not exist.
2. Atomically rename:
   ```rust
   std::fs::rename(staging_data_dir, target_data_dir)?;
   ```
3. Remove the staging metadata directory.

### Point 08: Non-Destructive Rollback & Quarantine (No Blind Deletion)
- **Rule**: Never execute blind `rm -rf` cleanup on failure!
- If initialization fails, exits non-zero, or crashes mid-way:
  1. The target `data/` directory was NEVER created or touched; it remains pristine.
  2. The staging directory is NOT deleted. It is quarantined:
     ```
     STAGING_DIR -> staging_<uuid>.failed_<timestamp>
     ```
  3. Staging directory permissions are restricted to `0000` or `0700` to prevent accidental access.
  4. Detailed stdout, stderr, and exit codes are written to a diagnostic log file for developer inspection.
  5. The runtime supervisor emits `RuntimeError::InitializationFailed` and halts cleanly.

### Point 09: Credential Storage & macOS Keychain Boundary
- **Architectural Policy (ADR-0004)**: Administrative database credentials must reside in the macOS Keychain (`security` CLI / Keychain Services API).
- **Honest Implementation Status**: In current Phase R1.2B/R1.3 crates, credential management resides in memory and structured configuration. Dedicated Keychain integration is scheduled for Phase 2 hardening. LocalForge does NOT claim active Keychain integration until verified code is compiled and tested.

### Point 10: Process Launch & Darwin Identity Verification
- MariaDB daemon `/opt/homebrew/bin/mariadbd` is spawned unprivileged with `--defaults-file=my.cnf`.
- PID, binary path, and launch timestamp are verified via `MacOSProcessManager` (`proc_pidinfo`).
- TCP port 3306 and Unix socket `run/mariadb.sock` (mode `0700`) are probed.

### Point 11: Database Deletion Lock Preservation
- The database deletion lock implemented in R1.2B (`DatabaseDeletionUnavailable`) remains **STRICTLY ENFORCED**.
- Initializing MariaDB does NOT grant or enable permission to delete user databases or wipe data directories.

### Point 12: Product Owner Checkpoint
```
[CHECKPOINT PO-AUTH-MARIADB-01]
Target: /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data
Action: Initialize MariaDB 13.0.2 system tables via atomic staging promotion
Status: STRICTLY HELD — REQUIRES SEPARATE GATE U5.D APPROVAL
```

---

## 5. Security & Architectural Invariants

1. **No Blind Cleanup**: Failed staging attempts are quarantined with timestamped suffixes for post-mortem forensics. Generic `rm -rf` is strictly forbidden.
2. **Zero Root Privilege**: Neither `sudo`, `setuid`, nor root socket ownership is permitted at any stage.
3. **No Foreign Daemon Interaction**: LocalForge manages only its dedicated instance and never interacts with Homebrew's global background services (`brew services start mariadb`).
4. **Execution Status**: This entire procedure remains **COMPLETELY UNEXECUTED** in Gate U5.A.
