# LocalForge — Gate U5 MariaDB Safe Initialization Runbook
## Storage Audit, Safety Classification & Approval-Gated Bootstrap Procedure

**Author:** Principal Software Architect & Database Architect  
**Governance:** Gate U5 — Runtime Readiness & Isolated Loopback Routing Architecture  
**Status:** DESIGN & READ-ONLY PREFLIGHT COMPLETED — EXECUTION ON HOLD  
**Binding Decision:** `PO-U5-02` (MariaDB Initialization Hold)  
**Date:** 2026-10-22

---

## 1. Executive Summary & Binding Decision PO-U5-02

Per **Decision PO-U5-02**, the LocalForge MariaDB storage subsystem has been thoroughly audited in a strictly read-only manner. Although the runtime supervisor correctly reported that the data directory is uninitialized:
- **NO automatic disk initialization has been executed.**
- **NO system tables have been generated or modified.**
- **NO database credentials have been altered.**
- **NO foreign or system MariaDB processes have been touched.**
- **Execution of this runbook requires a separate, explicit Product Owner authorization checkpoint.**

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

- **Rationale**: The directory exists within the authorized user Application Support boundary, contains zero files or subdirectories, contains no partial system tables, has no active locks, and is wholly owned by the unprivileged user `mehmetakar`.
- **Constraint**: Because no explicit Product Owner command to initialize has been issued, the safety guard implemented in `crates/runtime/src/mariadb.rs` continues to fail closed:
  > *"MariaDB veri dizini başlatılmamış. Güvenlik politikası ve Product Owner direktifi gereği otomatik disk ilklendirme durdurulmuştur. Lütfen yetkilendirme talep ediniz."*

---

## 4. 12-Point Approval-Gated Initialization Runbook

When the Product Owner explicitly authorizes initialization, the procedure must strictly follow this reviewable runbook.

### Point 01: Preconditions & Environment Validation
1. Verify host user is unprivileged (UID != 0).
2. Verify target volume `/System/Volumes/Data` has at least 500 MB free capacity (currently 11 GiB).
3. Verify `/opt/homebrew/bin/mariadb-install-db` executable SHA-256 and `arm64` signature.
4. Verify port 3306 is available or LocalForge dynamic fallback port (e.g. 13306) is available.

### Point 02: Target Datadir Identity Lock
- Canonical Base Path: `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb`
- Canonical Data Path: `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data`
- Canonical Run Path: `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/run`
- Canonical Socket: `/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/run/mariadb.sock`

### Point 03: Symlink & Path Traversal Barrier
- Before touching any file, `dunce::canonicalize()` is executed on `base_dir`.
- If `base_dir` resolves outside the user's home directory or points to a removable volume, the operation fails closed immediately.

### Point 04: Directory Creation & Atomic Permissions
- Create `data/` and `run/` with strict POSIX permissions:
  - Directories: Mode `0700` (`rwx------`) — inaccessible to other local users.
  - Configuration (`my.cnf`): Mode `0600` (`rw-------`).

### Point 05: Isolated Configuration Generation (`my.cnf`)
Generate an isolated configuration bounded to loopback only:
```ini
[mysqld]
datadir = /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data
socket = /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/run/mariadb.sock
pid-file = /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/run/mariadb.pid
port = 3306
bind-address = 127.0.0.1
skip-name-resolve
default_storage_engine = InnoDB
innodb_buffer_pool_size = 64M
innodb_file_per_table = 1
character-set-server = utf8mb4
collation-server = utf8mb4_unicode_ci

[client]
socket = /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/run/mariadb.sock
```

### Point 06: Atomic Bootstrap Command Execution
Execute `mariadb-install-db` in non-root mode:
```bash
/opt/homebrew/bin/mariadb-install-db \
  --defaults-file="/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/my.cnf" \
  --datadir="/Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data" \
  --auth-root-authentication-method=normal \
  --skip-test-db
```

### Point 07: Atomicity & Partial-Failure Rollback
- If `mariadb-install-db` exits with non-zero or the `mysql/` system table directory is not created:
  1. Record diagnostic output to session log.
  2. Clean up any partially written files in `data/` to prevent a poisoned state.
  3. Emit typed error `RuntimeError::ConfigGenerationFailed`.
  4. Never leave corrupted or half-initialized tables on disk.

### Point 08: Process Launch & Ownership Verification
- Launch `/opt/homebrew/bin/mariadbd --defaults-file=...` under unprivileged user.
- Capture Child PID and start timestamp via `MacOSProcessManager`.
- Record verified `ProcessIdentity` in `MariaDbSupervisor` memory registry.

### Point 09: Socket Readiness & Port Health Probe
- Poll Unix domain socket `mariadb.sock` using `is_unix_socket_alive()` (bounded to 15s timeout).
- Verify TCP loopback port 3306 is accepting connections via `std::net::TcpStream::connect("127.0.0.1:3306")`.

### Point 10: User & Privilege Hardening
- Verify that standard root user is restricted strictly to local socket connections.
- Ensure no test databases (`test`, `test_%`) are present.
- Store database administrative secrets strictly in the macOS Keychain (`security` CLI) per ADR-0004.

### Point 11: Data Preservation Guarantee
- MariaDB data remains strictly on internal SSD storage (`~/Library/Application Support/LocalForge/`), completely immune to external APFS volume unmounts or disconnects.
- Project repositories on `/Volumes/Mac Harici Disk` connect to the internal database engine via Unix domain socket or loopback TCP.

### Point 12: Explicit Product Owner Approval Checkpoint
```
[CHECKPOINT PO-AUTH-MARIADB-01]
Target: /Users/mehmetakar/Library/Application Support/LocalForge/runtime/mariadb/data
Action: Initialize MariaDB 13.0.2 system tables
Status: AWAITING EXPLICIT USER / PRODUCT OWNER INSTRUCTION
```

---

## 5. Security & Architectural Invariants

1. **Database Deletion Disabled**: The safety lock on database deletion (`DatabaseDeletionUnavailable`) remains active in the UI and backend.
2. **Zero Root Privilege**: Neither `sudo`, `setuid`, nor root socket ownership is permitted at any stage.
3. **No Foreign Daemons**: LocalForge manages only its dedicated instance and never interacts with Homebrew's global background services (`brew services start mariadb`).
