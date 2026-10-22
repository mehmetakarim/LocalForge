# LocalForge — Gate U5 Security Threat Model
## Isolated Loopback Routing, Process Lifecycle & Storage Security Analysis (Reconciled)

**Author:** Principal Software Architect & Security Engineer  
**Governance:** Gate U5.A — Architecture Safety Reconciliation & Implementation Readiness  
**Status:** RECONCILED SECURITY ASSESSMENT — IMPLEMENTATION ON HOLD (STRICT NO-GO)  
**Scope:** PHP-FPM, Nginx Multi-Port Loopback, MariaDB Unprivileged Isolation, Tauri IPC  
**Date:** 2026-10-22

---

## 1. Threat Modeling Methodology & Trust Boundaries

This threat model applies the STRIDE methodology to analyze the security implications of moving to a dedicated loopback port architecture (`127.0.0.1:18081..18999`), managing multi-version PHP-FPM FastCGI pools, and maintaining an isolated MariaDB database engine under unprivileged macOS user execution.

### 1.1 Identified Trust Boundaries
1. **User Space vs Kernel**: All LocalForge processes run strictly unprivileged as UID `mehmetakar` (non-root). No sudo, setuid, or root daemons.
2. **Local Loopback vs Remote Network**: All network listeners (`Nginx`, `MariaDB`, `PHP-FPM FastCGI`) bind strictly to `127.0.0.1` or Unix domain sockets. Zero listening on `0.0.0.0`, LAN, or WAN interfaces.
3. **Inter-Project Boundary**: Project A (`/scratch/r1_acceptance_project`) and Project B (`/Volumes/.../Projetest`) must be isolated from one another in document root, PHP session storage, FastCGI worker processes, and database access.
4. **Desktop IPC Boundary**: Frontend UI communicates with the Rust core strictly via typed Tauri IPC commands with fail-closed parameter validation.
5. **Browser Sandbox & Origin Boundary**: Same-Origin Policy separates DOM and Web Storage across ports, but RFC 6265 cookie scoping treats all ports on `127.0.0.1` as a shared namespace unless mitigated.

---

## 2. Reconciled Threat Analysis & Mitigation Matrix

### THREAT 01: Host-Header Spoofing & Cache Poisoning
- **Vector**: An attacker or rogue script sends an HTTP request with an arbitrary `Host:` header (e.g., `Host: malicious.com` or `Host: internal-admin.test`).
- **Impact in Shared Single-Port Architecture**: In a shared port setup (8080), a spoofed Host header could misroute requests or poison application URL generators.
- **Mitigation in Dedicated Loopback Architecture**:
  - Each project listens on a physically distinct TCP port (`18081`, `18082`).
  - Nginx routes requests based strictly on the bound port: even if `Host: malicious.com` is transmitted on port `18082`, Nginx serves **only** Project B's root directory and FastCGI pool.
  - Furthermore, `fastcgi_param SERVER_PORT` is set to `$server_port` and passed accurately.
- **Residual Risk**: Low. Application-level URL helpers that trust unvalidated `X-Forwarded-Host` headers could still misgenerate URLs if misconfigured by the user.

---

### THREAT 02: Cookie Namespace Collision Across Loopback Ports (RFC 6265)
- **Vector**: Project A (`127.0.0.1:18081`) sets a session cookie (e.g. `PHPSESSID=session_a`). The user opens Project B (`127.0.0.1:18082`). Because RFC 6265 cookies ignore port numbers, the browser transmits `session_a` to Project B, overwriting or colliding with Project B's session.
- **Impact**: Random session dropouts, user logout loops, or accidental session state leakage between local development projects.
- **Mitigation**:
  - Pool-specific session names: `php_value[session.name] = LF_SESS_<project_id>`.
  - Pool-specific session paths: `php_value[session.save_path] = ~/Library/.../sessions/<project_id>`.
  - Architecture option for Gate U5.B: Allocate distinct loopback aliases (`127.0.0.2:18081`, `127.0.0.3:18082`) to enforce both port and host-level cookie isolation.
- **Residual Risk**: Low to Medium (managed via pool configuration; developers configuring custom cookie names at framework level must adhere to project prefixes).

---

### THREAT 03: Local Port Hijacking by Foreign Processes
- **Vector**: An unrelated local process (e.g. Node.js app, Docker container, or malicious background script) binds to LocalForge's assigned project port before LocalForge starts.
- **Impact**: Traffic intended for LocalForge could be intercepted or served by an unauthorized process.
- **Mitigation**:
  - `is_port_available()` probes the socket before starting Nginx.
  - If occupied, LocalForge executes Darwin `lsof` / `proc_pidinfo` to inspect the occupying PID and executable identity.
  - If the occupant is NOT a verified LocalForge Nginx master process, LocalForge enters a fail-closed `PORT ÇAKIŞMASI` state.
  - **Invariant**: LocalForge **never** kills foreign processes; it alerts the user with PID diagnostics and reassigns an alternative free port upon confirmation.
- **Residual Risk**: Low (managed fail-closed).

---

### THREAT 04: Operating System TOCTOU Port Allocation Race
- **Vector**: LocalForge verifies in SQLite and via socket probe that port 18081 is free. Between the probe and Nginx's `bind()` syscall, an external local process binds to 18081.
- **Impact**: Nginx fails to start or reload, causing a gateway disruption.
- **Mitigation**:
  - SQLite port allocation and OS port ownership are strictly decoupled.
  - LocalForge does not assume a port is owned until Nginx successfully executes `bind()` and `listen()`.
  - Candidate Nginx configuration is validated (`nginx -t`). If Nginx reports `bind() to 127.0.0.1:18081 failed (48: Address already in use)`, LocalForge aborts reload, retains previous running configuration, flags the port conflict, and offers a one-click port reassignment.
- **Residual Risk**: Low (handled via fail-closed configuration rollback).

---

### THREAT 05: PID-Reuse and Process Spoofing Attacks
- **Vector**: The operating system recycles a terminated PID. If LocalForge sends signals (`SIGTERM`, `SIGKILL`) based purely on numeric PID, an innocent application could be terminated.
- **Mitigation**:
  - LocalForge's `MacOSProcessManager` enforces verified `ProcessIdentity`:
    $$\text{ProcessIdentity} = (\text{PID}, \text{Canonical Executable Path}, \text{Start Time Seconds}, \text{Start Time Microseconds})$$
  - Before any process inspection or signaling, LocalForge verifies kernel start timestamps via `proc_pidinfo`. If the launch timestamp or binary path has changed, signaling fails closed immediately.
- **Residual Risk**: Negligible (guaranteed by Darwin kernel process telemetry).

---

### THREAT 06: Symlink Attacks & Directory Traversal Across Volumes
- **Vector**: A project repository contains a malicious symbolic link (`public/secrets -> /etc/passwd` or `public/db -> ~/.localforge/localforge.db`).
- **Mitigation**:
  - Nginx configuration enables strict path confinement.
  - FastCGI `SCRIPT_FILENAME` parameters are canonicalized by `dunce::canonicalize()` before execution.
  - LocalForge Storage `sanitize_project_path()` rejects relative path escapes (`../`), null bytes, and non-canonical references.
  - Symlinks pointing outside the project document root fail with HTTP 403/404.
- **Residual Risk**: Low (standard web server filesystem boundaries).

---

### THREAT 07: Tenant Cross-Contamination & Shared FastCGI Sockets
- **Vector**: Multiple projects communicate with the same PHP-FPM worker pool, allowing Project A to read Project B's memory, OpCache, or session files.
- **Mitigation**:
  - Every project receives an independent FastCGI pool configuration (`~/Library/.../run/<project-id>.sock`).
  - Worker processes run in separate process groups.
  - Dedicated `session.save_path` is configured per pool.
  - `open_basedir` can be restricted to the project root directory.
- **Residual Risk**: Low (PHP processes run under same OS user `mehmetakar`, so this is a process-level boundary, not a virtualization container).

---

### THREAT 08: Unauthorized Remote Network Exposure (LAN / Public IP)
- **Vector**: A development server accidentally listens on `0.0.0.0`, exposing internal projects or unauthenticated MariaDB instances to local Wi-Fi or internet attackers.
- **Mitigation**:
  - Nginx configuration generator hardcodes `listen 127.0.0.1:<port>;` (never `listen <port>;` or `listen 0.0.0.0:<port>;`).
  - MariaDB configuration specifies `bind-address = 127.0.0.1` and `skip-name-resolve`.
  - PHP-FPM pools communicate strictly over local Unix domain sockets mode `0700`.
- **Residual Risk**: Low.

---

### THREAT 09: External Drive Disconnect & Zombie Socket Leakage
- **Vector**: `/Volumes/Mac Harici Disk` is disconnected while Nginx and PHP-FPM pools are active.
- **Mitigation**:
  - FastCGI Unix domain sockets reside on internal SSD (`Application Support/.../run/`), preventing socket destruction on volume unmount.
  - `VolumeBarrier` detects unmount, sets project state to `VolumeDisconnected`, and Nginx returns HTTP 502 without attempting to write to the root disk.
- **Residual Risk**: Low (fail-closed behavior verified).

---

### THREAT 10: Premature or Corrupted MariaDB Datadir Mutation
- **Vector**: Automatic or unvalidated database bootstrap creates corrupted or unauthenticated system tables on disk.
- **Mitigation**:
  - Automatic initialization is completely blocked (`Decision PO-U5-02`).
  - Reconciled runbook mandates private staging directory (`staging_<uuid>`) on the same filesystem.
  - Root authentication uses socket peercred (`--auth-root-authentication-method=socket`), preventing an unsecured administrative window.
  - Atomic rename promotion over destination data directory.
  - Failed initialization preserves quarantined directory for forensics; generic `rm -rf` is strictly forbidden.
- **Residual Risk**: Low (governed by Gate U5.D checkpoint).

---

## 3. Summary of Honest Residual Risk Posture

Separate loopback ports provide an exceptional developer experience and solve local URL routing, but they are:
1. **NOT an OS virtualization boundary** (Docker/VM sandbox). Malicious PHP code executing on the machine runs with the user's OS permissions.
2. **NOT a complete cookie partition by default** without custom session naming or loopback IP alias partitioning.
3. **Subject to TOCTOU race conditions** at OS bind time, which LocalForge mitigates via fail-closed error handling and atomic Nginx configuration rollbacks.
