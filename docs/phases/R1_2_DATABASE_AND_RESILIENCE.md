# LocalForge — Phase R1.2: MariaDB, Storage Resilience & Multi-Project Verification

**Phase**: Recovery Phase R1.2  
**Status**: SPECIFIED / AWAITING R1.1 COMPLETION  
**Prerequisite**: Formal Product Owner Approval of Phase R1.1  
**Deciders**: Principal Rust Engineer, Database Architect, Systems Engineer  

---

## 1. Explicit Scope

Phase R1.2 introduces isolated user-level MariaDB database management and rigorously validates multi-project concurrency and external APFS storage resilience.

### In Scope for R1.2:
1. **MariaDB Internal Storage Isolation**:
   - Initialize and supervise a user-level `mariadbd` instance under unprivileged permissions.
   - Default database storage strictly located in internal application storage:
     `~/Library/Application Support/LocalForge/mariadb/data/`.
   - Prevent raw InnoDB database storage on external drives until transactional safety is established.
   - Health probes via `mysqladmin ping --socket=...`.
2. **Multi-Project Concurrency**:
   - Concurrently run two separate projects with different PHP versions (e.g. PHP 8.4 and PHP 8.3).
   - Independent FastCGI sockets (`.localforge/run/php-fpm-84.sock` vs `.localforge/run/php-fpm-83.sock`).
   - Unified Nginx gateway routing requests by Host header to the correct pool without collision.
3. **External Storage Resilience & Disconnect Guard**:
   - Verify volume disconnect behavior: when an external APFS drive disappears, `VolumeBarrier` signals immediate graceful shutdown of child processes and refuses fallback writes.
4. **Database Backup & Recovery**:
   - Provide structured dump/restore commands to `.localforge/backups/`.

---

## 2. Test Matrix for R1.2

| Test ID | Level | Target | Test Procedure | Acceptance Threshold |
|---|---|---|---|---|
| **TEST-R1.2-01** | Integration | `crates/runtime::mariadb` | Initialize internal datadir via `mysql_install_db`, spawn `mariadbd` on port 3306, verify socket. | Socket ping `mysqld is alive`, zero root priv |
| **TEST-R1.2-02** | Concurrency | `crates/runtime::supervisor`| Run Project A (PHP 8.4) and Project B (PHP 8.3) concurrently; query both via HTTP loopback. | Both return 200 OK with distinct PHP versions |
| **TEST-R1.2-03** | Resilience | `crates/platform::volume` | Simulate external volume unmount; verify FPM stops, project marked DISCONNECTED, 0 internal writes. | Volume barrier blocks write; graceful shutdown |
| **TEST-R1.2-04** | Recovery | `crates/runtime::mariadb` | Simulate sudden SIGKILL to MariaDB; restart supervisor; verify InnoDB crash recovery and clean startup. | Database recovers without corruption |
| **TEST-R1.2-05** | Backup | `crates/runtime::mariadb` | Execute snapshot dump to `.localforge/backups/`; verify SQL archive integrity and restore feasibility. | Valid SQL dump file created |

---

## 3. Acceptance Scenario & Exit Criteria

1. **Database Readiness**: MariaDB responds to queries over local socket and loopback port 3306 from local PHP scripts.
2. **Dual-Project Concurrency**: Two active projects run side-by-side with independent sockets and PIDs.
3. **Storage Decoupling**: Project source files on `/Volumes/Mac Harici Disk/...` connect seamlessly to databases stored safely on the internal disk.
4. **Zero State Pollution**: Disconnecting the external drive leaves internal storage clean and undamaged.

---

## 4. Evidence & Handoff Requirements

- Git commit on branch `phase-r1/r1-2-database-and-resilience`.
- Verification logs showing concurrent FastCGI routing and MariaDB socket pings.
- Product Owner approval required before proceeding to R1.3.
