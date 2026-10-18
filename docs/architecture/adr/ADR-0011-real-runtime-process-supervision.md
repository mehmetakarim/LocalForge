# ADR-0011: Real Runtime Process Supervision & Non-Root Lifecycle Architecture

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Rust Engineer, Systems Engineer, Security Auditor
- **Phase**: Recovery Phase R1 Baseline

---

## 1. Context

During Phase 2, LocalForge implemented configuration file generators for PHP-FPM pools and Nginx virtual hosts. However, as identified in Independent Technical Audit A1:
1. Generating configuration strings is not equivalent to managing live, healthy operating system processes.
2. The UI previously simulated "Running" status based on database state rather than real active process telemetry.
3. Managing development servers without `sudo` on macOS requires strict port and socket discipline to prevent permission violations and port conflicts.

---

## 2. Decision

We establish an active, unprivileged **Runtime Process Supervision Architecture** centered in `crates/runtime`:

### 2.1 Unprivileged Child Process Spawning
- All runtime daemons (PHP-FPM master, Nginx worker, MariaDB server) execute strictly under the **current unprivileged user space** (UID > 500 on macOS).
- **No Sudo**: Root privileges are strictly forbidden. Development servers bind exclusively to unprivileged ports (>1024, e.g. 8080, 8443, 3306) on the local loopback interface (`127.0.0.1`).
- FastCGI communication between Nginx and PHP-FPM occurs over isolated Unix domain sockets (`.localforge/run/php-fpm.sock`) created with `0660` user permissions.

### 2.2 Tokio Async Supervisory Tree & Verified Process Identity
- Process management logic resides in `crates/runtime::supervisor` implementing a shared `ProcessSupervisor` trait.
- Processes are spawned via `tokio::process::Command` in a dedicated process group (`setpgid`).
- Active child processes are registered in an in-memory process registry tracking:
  - Process ID (PID)
  - Process Group ID (PGID)
  - Canonical executable binary path
  - Launch timestamp (start ticks) for PID reuse verification
  - Bound socket path / port
  - Health probe status

### 2.3 Graceful Termination, PID Reuse Protection & Orphan Prevention
- **PID Reuse Defense**: Before signaling any PID, the supervisor queries the operating system (`proc_pidinfo` / `ps`) to confirm that the running process's binary path and start timestamp match the recorded launch data. Unverified or reused PIDs are NEVER signaled.
- **Shared Gateway vs Independent Pool**: A single unprivileged Nginx master process acts as the shared loopback gateway. Stopping Project A terminates only Project A's PHP-FPM process group, leaving the shared Nginx proxy and Project B intact.
- **Bounded Escalation**: Stopping a project sends `SIGTERM` to the verified process group (`nix::sys::signal::killpg`), granting a 5-second graceful exit window for inflight FastCGI requests. If processes remain active after 5 seconds, `SIGKILL` is issued only to the verified group.
- **Stale Socket & Startup Reconciliation**: Stale `.sock` and `.pid` files are inspected, validated against active processes, and cleanly removed prior to spawning new daemons. Upon application restart, existing PIDs are reconciled.

### 2.4 External Volume Disconnect Safety & Storage Policy
- In alignment with ADR-0004 and ADR-0012, if an external workspace volume is unmounted, the supervisor immediately aborts operations, issues SIGTERM to affected child processes, and flags the project as `DISCONNECTED`.
- The supervisor is strictly prohibited from writing fallback logs or runtime state to the internal disk when an external target is unavailable.
- MariaDB databases reside strictly in internal application data storage (`~/Library/Application Support/LocalForge/mariadb/`) by default, decoupled from external project source code.

---

## 3. Consequences

- **Positive**: Eliminates security risks associated with `sudo` and root daemons.
- **Positive**: Guarantees truthful runtime state in the UI based on real OS process and socket probes.
- **Positive**: Cleanly handles concurrent multi-version PHP projects without inter-project interference.
- **Trade-off**: High unprivileged ports (e.g. `:8080`) require port numbers in URLs unless a local loopback resolver (such as dnsmasq or pfctl redirect) is configured separately.
