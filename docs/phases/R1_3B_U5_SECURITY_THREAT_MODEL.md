# LocalForge — Gate U5 Security Threat Model
## Isolated Loopback Routing, Process Lifecycle & Storage Security Analysis

**Author:** Principal Software Architect & Security Engineer  
**Governance:** Gate U5 — Runtime Readiness & Isolated Loopback Routing Architecture  
**Status:** SECURITY ASSESSMENT COMPLETED  
**Scope:** PHP-FPM, Nginx Multi-Port Loopback, MariaDB Unprivileged Isolation, Tauri IPC  
**Date:** 2026-10-22

---

## 1. Threat Modeling Methodology & Trust Boundaries

This threat model applies the STRIDE methodology to analyze the security implications of moving to a dedicated loopback port architecture (`127.0.0.1:18081..18999`), managing multi-version PHP-FPM FastCGI pools, and maintaining an isolated MariaDB database engine under unprivileged macOS user execution.

### 1.1 Identified Trust Boundaries
1. **User Space vs Kernel**: All LocalForge processes run strictly unprivileged as UID `mehmetakar` (non-root). No sudo, setuid, or root daemons.
2. **Local Loopback vs Remote Network**: All network listeners (`Nginx`, `MariaDB`, `PHP-FPM FastCGI`) bind strictly to `127.0.0.1` or Unix domain sockets. Zero listening on `0.0.0.0`, LAN, or WAN interfaces.
3. **Inter-Project Boundary**: Project A (`/scratch/r1_acceptance_project`) and Project B (`/Volumes/.../Projetest`) must be strictly isolated from one another in document root, PHP session storage, FastCGI worker processes, and database access.
4. **Desktop IPC Boundary**: Frontend UI communicates with the Rust core strictly via typed Tauri IPC commands with fail-closed parameter validation.

---

## 2. Threat Analysis & Mitigation Matrix

### THREAT 01: Host-Header Spoofing & Cache Poisoning
- **Vector**: An attacker or rogue script sends an HTTP request with an arbitrary `Host:` header (e.g., `Host: malicious.com` or `Host: internal-admin.test`).
- **Impact in Shared Single-Port Architecture**: In a shared port setup (8080), a spoofed Host header could misroute requests or poison application URL generators.
- **Mitigation in Dedicated Loopback Architecture**:
  - Each project listens on a physically distinct TCP port (`18081`, `18082`).
  - Nginx routes requests based strictly on the bound port: even if `Host: malicious.com` is transmitted on port `18082`, Nginx serves **only** Project B's root directory and FastCGI pool.
  - Furthermore, `fastcgi_param SERVER_PORT` is hardcoded to the assigned port, preventing header-based port spoofing.
- **Residual Risk**: Minimal / Negligible.

---

### THREAT 02: Local Port Hijacking by Foreign Processes
- **Vector**: An unrelated local process (e.g. Node.js app, Docker container, or malicious background script) binds to LocalForge's assigned project port before LocalForge starts.
- **Impact**: Traffic intended for LocalForge could be intercepted or served by an unauthorized process.
- **Mitigation**:
  - `is_port_available()` probes the socket before starting Nginx.
  - If occupied, LocalForge executes Darwin `lsof` / `proc_pidinfo` to inspect the occupying PID and executable identity.
  - If the occupant is NOT a verified LocalForge Nginx master process, LocalForge enters a fail-closed `PORT ÇAKIŞMASI` state.
  - **Invariant**: LocalForge **never** kills foreign processes; it alerts the user with PID diagnostics and reassigns an alternative free port upon confirmation.
- **Residual Risk**: Low (managed fail-closed).

---

### THREAT 03: PID-Reuse and Process Spoofing Attacks
- **Vector**: An operating system recycles a terminated PID. If LocalForge sends signals (`SIGTERM`, `SIGKILL`) based purely on numeric PID, an innocent application could be terminated.
- **Mitigation**:
  - LocalForge's `MacOSProcessManager` enforces verified `ProcessIdentity`:
    $$\text{ProcessIdentity} = (\text{PID}, \text{Canonical Executable Path}, \text{Start Time Seconds}, \text{Start Time Microseconds})$$
  - Before any process inspection or signaling, LocalForge verifies kernel start timestamps via `proc_pidinfo`. If the launch timestamp or binary path has changed, signaling fails closed immediately.
- **Residual Risk**: Negligible.

---

### THREAT 04: Symlink Attacks & Directory Traversal Across Volumes
- **Vector**: A project repository contains a malicious symbolic link (`public/secrets -> /etc/passwd` or `public/db -> ~/.localforge/localforge.db`).
- **Mitigation**:
  - Nginx configuration enables strict path confinement.
  - FastCGI `SCRIPT_FILENAME` parameters are canonicalized by `dunce::canonicalize()` before execution.
  - LocalForge Storage `sanitize_project_path()` rejects relative path escapes (`../`), null bytes, and non-canonical references.
  - Symlinks pointing outside the project document root are strictly rejected or fail with HTTP 403/404.
- **Residual Risk**: Low.

---

### THREAT 05: Port Allocation Race Conditions During Concurrent Startup
- **Vector**: Two projects are created or started simultaneously in the UI, competing for the same free port.
- **Mitigation**:
  - Port allocation in `MetadataStore` is protected by a SQLite transaction and a global Rust `Mutex`.
  - The `allocated_port` column has a `UNIQUE` database constraint: any collision triggers a database constraint error, causing the allocator to retry with the next free port atomically.
- **Residual Risk**: Zero (guaranteed atomic).

---

### THREAT 06: Tenant Cross-Contamination & Shared FastCGI Sockets
- **Vector**: Multiple projects communicate with the same PHP-FPM worker pool, allowing Project A to read Project B's memory, OpCache, or session files.
- **Mitigation**:
  - Every project receives an independent FastCGI pool configuration (`~/.localforge/run/<project-id>.sock`).
  - Worker processes run in separate process groups.
  - Dedicated `session.save_path` is configured per pool.
  - `open_basedir` can be restricted to the project root directory.
- **Residual Risk**: Negligible.

---

### THREAT 07: Path Traversal Out of Project Document Root
- **Vector**: A malicious HTTP request (`GET /../../../../etc/passwd HTTP/1.1`) attempts to bypass the web root.
- **Mitigation**:
  - Nginx normalizes URI paths before evaluation (`try_files $uri $uri/ /index.php?$query_string;`).
  - FastCGI SAPI rejects script filenames that do not resolve inside the configured document root.
- **Residual Risk**: Zero (standard web server baseline).

---

### THREAT 08: Unauthorized Remote Network Exposure (LAN / Public IP)
- **Vector**: A development server accidentally listens on `0.0.0.0`, exposing internal projects or unauthenticated MariaDB instances to local Wi-Fi or internet attackers.
- **Mitigation**:
  - Nginx configurations strictly use `listen 127.0.0.1:<port>;` (never bare `listen <port>;` which defaults to `0.0.0.0`).
  - MariaDB configuration enforces `bind-address = 127.0.0.1` and `skip-name-resolve`.
  - PHP-FPM pools communicate strictly over local Unix domain sockets with POSIX permissions `0660`.
- **Residual Risk**: Zero (strictly bounded to local loopback).

---

### THREAT 09: Stale Endpoints & Orphan Socket Hijacking
- **Vector**: LocalForge crashes or is forcefully terminated; stale Unix domain sockets (`.sock`) or PID files linger in `~/.localforge/run/`.
- **Mitigation**:
  - `is_unix_socket_alive()` sends an active probe down the socket. If the socket file exists but refuses connections, it is classified as dead and safely unlinked.
  - PID files are validated against kernel launch timestamps before trusting.
- **Residual Risk**: Negligible.

---

### THREAT 10: Browser Cookie Leakage Across Loopback Ports
- **Vector**: Because standard browser cookie scoping treats `127.0.0.1:18081` and `127.0.0.1:18082` as sharing the same host domain (`127.0.0.1`), cookies set without path scoping might be readable or overwritten across projects.
- **Mitigation & Developer Guidance**:
  - LocalForge configures distinct session cookie names per project pool (e.g. `PHPSESSID_proj_...`).
  - PHP session storage directories are isolated per project pool (`php_admin_value[session.save_path]`).
  - Developers are guided to use project-specific session names or test via separate browser profiles / private windows if strict cookie isolation is required.
- **Residual Risk**: Low (documented browser architectural reality).

---

### THREAT 11: Remote SFTP & Database Deletion Safety Invariants
- **Vector**: Compromised AI model or malicious tool call initiates file upload to external server or deletes MariaDB databases.
- **Mitigation**:
  - **SFTP Invariant**: Live deployment execution remains permanently disabled (`DeploymentNotImplemented`). Dry-run previews only.
  - **Database Invariant**: Database deletion remains permanently disabled with `DatabaseDeletionUnavailable`.
  - **No Root Execution**: All processes execute under the standard unprivileged user.
- **Residual Risk**: Zero (permanent fail-closed code barriers).

---

## 3. Threat Model Conclusion
The isolated loopback port architecture (`Option A`) significantly strengthens LocalForge's security posture by eliminating Host-header reliance and tenant cross-contamination, while strictly preserving all established Phase R0 through R1.3B security boundaries.
