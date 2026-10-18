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

### 2.2 Tokio Async Supervisory Tree
- Process management logic resides in `crates/runtime::supervisor` implementing a shared `ProcessSupervisor` trait.
- Processes are spawned via `tokio::process::Command` with standard IO captured for structured log streaming.
- Active child processes are registered in an in-memory process registry tracking:
  - Process ID (PID)
  - Process group ID (PGID)
  - Bound socket path / port
  - Start timestamp and health telemetry

### 2.3 Graceful Termination & Orphan Prevention
- Stopping a project sends `SIGTERM` to the process group (`nix::sys::signal::killpg`), granting a 5-second graceful shutdown window for inflight PHP requests to complete.
- If a process fails to exit after the graceful window, `SIGKILL` is issued.
- Stale socket files and PID locks are cleaned up automatically upon exit.
- Process exit events are actively monitored to prevent zombie or detached orphan processes.

### 2.4 External Volume Disconnect Safety
- In alignment with ADR-0004, if an external workspace volume is unmounted, the supervisor immediately aborts operations, issues SIGTERM to affected child processes, and flags the project as `DISCONNECTED`.
- The supervisor is strictly prohibited from writing fallback logs or runtime state to the internal disk when an external target is unavailable.

---

## 3. Consequences

- **Positive**: Eliminates security risks associated with `sudo` and root daemons.
- **Positive**: Guarantees truthful runtime state in the UI based on real OS process and socket probes.
- **Positive**: Cleanly handles concurrent multi-version PHP projects without inter-project interference.
- **Trade-off**: High unprivileged ports (e.g. `:8080`) require port numbers in URLs unless a local loopback resolver (such as dnsmasq or pfctl redirect) is configured separately.
