# LocalForge — Recovery Phase R1: Runtime Integration & Process Supervision Plan

**Document Version**: 1.0.0  
**Phase**: Recovery Phase R1 (Next Engineering Sprint)  
**Status**: READY FOR IMPLEMENTATION REVIEW  
**Deciders**: Principal Rust Engineer, Systems Engineer, QA Lead  

---

## 1. Phase Objective & Boundary

Recovery Phase R0 stabilized safety boundaries and repaired GitHub Actions CI.  
**Phase R1** bridges configuration generation to **real, active process supervision** on macOS Apple Silicon.

### Explicit Phase Boundaries:
- **In Scope**:
  - Live PHP-FPM process supervision via `std::process::Command` / `tokio::process::Command` under unprivileged user permissions.
  - Live loopback Nginx master process orchestration with generated project configs.
  - Real HTTP loopback verification (`http://<project>.test:8080`).
  - Active PID tracking, socket validation, and safe SIGTERM/SIGKILL termination.
  - Staged, isolated user-level MariaDB lifecycle management.
  - Integration with the Spatial Studio and Native Workspace UI.
- **Out of Scope (Deferred to R2 / Subsequent Phases)**:
  - Production remote SFTP file uploads (remains fail-closed with dry-run simulation only).
  - Background Launchd daemons surviving app shutdown.
  - Root `/etc/hosts` automated modification (loopback operates via `127.0.0.1:8080` or `.test` dnsmasq/resolver).

---

## 2. Primary Acceptance Scenario 1: End-to-End PHP Runtime Flow

This scenario must execute and pass completely on live macOS Apple Silicon hardware before Phase R1 can be declared complete:

```
[ Step 1: Volume Selection ]
   External APFS volume (/Volumes/Mac Harici Disk) verified via UUID.

[ Step 2: Project Creation ]
   Directory created at /Volumes/.../test-project/
   .localforge/project.yaml and public/index.php initialized.

[ Step 3: Process Launch (PHP-FPM) ]
   PHP-FPM binary discovered at Homebrew path.
   Dedicated pool conf written to .localforge/run/fpm.conf.
   Spawned as child process; PID stored; socket verified at .localforge/run/php-fpm.sock.

[ Step 4: Process Launch (Nginx Loopback) ]
   Nginx spawned on unprivileged port 8080 with fastcgi_pass pointing to unix socket.
   Port conflict probe validates port 8080 is bound by LocalForge.

[ Step 5: Live HTTP Verification ]
   HTTP GET request to http://127.0.0.1:8080 with Host: test-project.test.
   Assert HTTP status 200 OK and PHP response payload.

[ Step 6: Real-time UI Telemetry ]
   UI updates status from STARTING -> RUNNING.
   Actual PID, memory usage, and socket status displayed.

[ Step 7: Clean Process Termination ]
   User or test triggers "Stop Project".
   PHP-FPM received SIGTERM; Nginx reloaded or stopped.
   Child processes exit cleanly; zero zombie/orphan processes left running.
   UI updates status immediately to STOPPED.
```

---

## 3. Secondary Acceptance Scenario 2: Isolated User-Level MariaDB

MariaDB integration is staged as an independent, testable milestone:
1. `mysql_install_db` initializes an isolated datadir inside `.localforge/data/mariadb/` on the configured project storage volume.
2. `mariadbd` is spawned with:
   - `--port=3306` (or next free unprivileged port if 3306 is occupied).
   - `--socket=.localforge/run/mariadb.sock`.
   - `--datadir=.localforge/data/mariadb/`.
   - `--skip-networking=0` or restricted to `bind-address=127.0.0.1`.
3. Ping MariaDB socket with `mysqladmin ping --socket=...` to verify readiness.
4. Clean shutdown via `mysqladmin shutdown` or SIGTERM.

---

## 4. Failure Handling & Recovery Specifications

### 4.1 Port Conflict Resolution
- If port 8080 or 3306 is bound by another application (e.g. Docker, MAMP, Homebrew service):
  - Do NOT silently fail or crash.
  - The runtime probe detects the conflicting PID via `lsof -i :8080` or socket bind trial.
  - UI displays status as **`DEGRADED (Port 8080 Conflict)`** with an explanation and recommended next free port (e.g. `8081`).

### 4.2 Process Crash & Orphan Cleanup
- If a child process crashes unexpectedly (SIGSEGV, out of memory):
  - The supervisor detects child exit via async process wait loop.
  - Cleans up stale `.sock` and `.pid` files.
  - Flags project status as **`FAILED (Exit code N)`**.
  - Records stderr diagnostics to `.localforge/logs/error.log`.

### 4.3 Sudden External Volume Disconnect
- If `/Volumes/...` is disconnected while processes are running:
  - macOS DiskArbitration callback triggers `VolumeBarrier`.
  - Immediately signals SIGTERM to child processes.
  - Marks project as **`DISCONNECTED`**.
  - Refuses to write new logs or state files to `/Volumes/...` to prevent recreating dummy directories on the root mount.

---

## 5. Architectural Traits & Shared Core Contracts

In accordance with ADR-0002, process supervision logic lives exclusively in `crates/runtime`:

```rust
// Contract in crates/runtime/src/supervisor.rs
pub trait ProcessSupervisor: Send + Sync {
    async fn start(&self, project: &ProjectRecord) -> Result<RunningProcessInfo>;
    async fn stop(&self, project_id: &ProjectId) -> Result<()>;
    async fn health_check(&self, project: &ProjectRecord) -> HealthStatus;
}
```

The Tauri desktop application, CLI, and MCP server all invoke this exact trait.
