# ADR-0012: Process Identity Verification, Lifecycle Safety & MariaDB Internal Storage Isolation

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Rust Engineer, Systems Architect, Security Auditor
- **Phase**: Recovery Phase R1.0 Architecture Refinement

---

## 1. Context & Architectural Deficiencies

During the R1.0 architecture review, critical runtime safety and storage isolation issues were identified:
1. **PID Reuse Risk**: Simply storing a `u32` PID in a file and later calling `kill(pid, SIGTERM)` is unsafe on macOS and Unix systems. If the managed daemon crashes and the OS reuses the PID for another process (e.g. an unrelated system or user app), LocalForge could inadvertently terminate an innocent process.
2. **Local Domain Resolution Confusion**: An HTTP request to `127.0.0.1:8080` with a `Host: project.test` header tests Nginx virtual-host dispatch, but does **not** prove that macOS or browser DNS resolves `project.test`. True browser resolution requires a dedicated resolver configuration.
3. **Database on External Storage Vulnerability**: Storing raw MariaDB InnoDB tables and redo logs on an external APFS drive risks catastrophic filesystem corruption if the drive is disconnected during an active write transaction.
4. **Shared vs Per-Project Lifecycle Coupling**: Nginx reverse proxy serves as a shared gateway, whereas PHP-FPM pools belong to individual projects. Stopping one project must never terminate Nginx or interrupt another project's pool.

---

## 2. Decision

We establish strict architectural policies for process lifecycle safety, domain resolution, and database storage:

### 2.1 Verified Process Identity & PID Reuse Protection
Before transmitting any signal (`SIGTERM`, `SIGKILL`, `SIGQUIT`) to a recorded PID:
1. **Executable Path Verification**: Query the operating system (via `proc_pidinfo` / `ps -p <pid> -o comm=`) to verify that the executable path matches the expected binary (e.g., `/opt/homebrew/opt/php@8.4/sbin/php-fpm` or LocalForge-managed Nginx).
2. **Process Start Timestamp Verification**: Record the process start time (`lstart` / elapsed ticks) alongside the PID at spawn time. If the current process with that PID has a start time later than recorded, it is a reused PID and MUST NOT be signaled.
3. **Process Group Ownership (PGID)**: Spawn daemons in a dedicated process group (`setpgid`). Bounded escalation sends `SIGTERM` to the process group, waits for a 5-second graceful exit window, and escalates to `SIGKILL` only if the verified process group remains active.

### 2.2 Shared Gateway vs Independent Pool Lifecycle
- **Shared Nginx Gateway**: A single, unprivileged Nginx master process runs under the user account on loopback port `8080`. It loads virtual host configurations dynamically from `.localforge/nginx/conf.d/*.conf`. Starting or stopping a project updates its virtual host and issues a safe `nginx -s reload`. Nginx itself remains running as long as at least one project is active.
- **Independent PHP-FPM Pools**: Each project manages its own independent PHP-FPM process and Unix domain socket (`.localforge/run/php-fpm.sock`). Stopping Project A terminates only Project A's PHP-FPM process group; Project B is completely unaffected.

### 2.3 Local Domain Resolution Architecture
- **Phase R1.1 Baseline**: Loopback access is guaranteed via `http://127.0.0.1:8080` (or `http://localhost:8080`) with Nginx Host-header dispatch.
- **macOS Resolver Configuration (Separate Capability)**: Browser-level `.test` resolution requires configuring macOS `/etc/resolver/test` pointing to a local loopback DNS responder (e.g. `dnsmasq` on `127.0.0.1:53`). Because modifying `/etc/resolver` requires human authorization, it is decoupled from core runtime execution and documented with explicit acceptance tests. No claim of zero-configuration `.test` browser support may be made until verified.

### 2.4 MariaDB Internal Storage Isolation & External Drive Safety
- **Default Storage Location**: MariaDB databases and InnoDB data files MUST reside on the reliable **internal SSD** within the user application data directory:
  `~/Library/Application Support/LocalForge/mariadb/data/`
- **Source vs Data Decoupling**: Project source code may reside on external APFS workspaces (`/Volumes/Mac Harici Disk/...`), while database files remain protected on internal storage.
- **External Database Storage Deferred**: Placing MariaDB data directly on external drives is deferred until volume identity, disconnect resilience, crash recovery, and automated transactional backups are implemented and proven in later phases.
- **Backup & Recovery Protocol**: MariaDB instances must support atomic `mysqldump` exports and structured snapshots to `.localforge/backups/` before any project migration or directory re-linking.

---

## 3. Consequences

- **Positive**: Prevents accidental termination of unrelated processes caused by OS PID recycling.
- **Positive**: Protects databases from corruption during unexpected external drive disconnections.
- **Positive**: Ensures project lifecycle isolation: one project failing does not bring down the entire local development stack.
- **Trade-off**: Requires running Nginx on port 8080 with port-specific URLs until resolver configuration is explicitly established.
