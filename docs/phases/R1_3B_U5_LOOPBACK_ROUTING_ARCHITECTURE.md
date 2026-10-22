# LocalForge — Gate U5 Isolated Loopback Routing Architecture
## Dedicated Port Allocation, Multi-Port Nginx Gateway & Security Reconciliation

**Author:** Principal Software Architect & Systems Engineer  
**Governance:** Gate U5.A — Architecture Safety Reconciliation & Implementation Readiness  
**Status:** ARCHITECTURAL DESIGN RECONCILED — IMPLEMENTATION ON HOLD (STRICT NO-GO)  
**Binding Decision:** `PO-U5-03` (Isolated Loopback Routing)  
**Date:** 2026-10-22

---

## 1. Architectural Objective & Core Problem Statement

### 1.1 The User Experience Goal
The primary objective defined by the Product Owner in **Decision PO-U5-03** is:

$$\text{Select Project} \longrightarrow \text{Start Project} \longrightarrow \text{Open in Browser} \longrightarrow \text{Correct Project Opens Immediately}$$

The local development studio must provide an out-of-the-box browser experience that **never requires**:
- Manual terminal intervention (`sudo vi /etc/hosts`).
- Modifying system-wide DNS configurations (`/etc/resolver/test`).
- Third-party DNS proxy daemons (`dnsmasq`, `Acrylic`, etc.).
- Elevating LocalForge processes to `root`.

### 1.2 The Root Cause of `.test` Domain Failure
In Phase R1.1 through R1.3B, LocalForge routed all projects through a single shared Nginx port (`127.0.0.1:8080`), multiplexing virtual hosts based on the HTTP `Host:` header (e.g., `Host: projetest.test`).
- While this works for HTTP clients with manual headers (`curl -H "Host: projetest.test" http://127.0.0.1:8080/`), standard macOS browsers (Safari, Chrome, Arc) rely on the operating system's `mDNSResponder` resolver.
- RFC 6761 reserves `.test` for testing, but **does not specify** automatic resolution to `127.0.0.1`.
- Consequently, typing `http://projetest.test:8080/` in a browser triggers an immediate `NXDOMAIN` (Name Not Resolved) DNS failure.

### 1.3 The Solution: Deterministic Isolated Loopback Ports
By allocating each project an isolated loopback port on `127.0.0.1` (e.g. `127.0.0.1:18081`, `127.0.0.1:18082`), the endpoint is resolved directly by the IP protocol stack with **zero DNS lookup**:
- Project A (`r1_acceptance_project`) $\longrightarrow$ `http://127.0.0.1:18081/`
- Project B (`Projetest`) $\longrightarrow$ `http://127.0.0.1:18082/`

---

## 2. Fundamental Boundary Clarification: Routing vs Security Sandbox

> [!WARNING]
> **Loopback Ports Provide a Routing & Usability Boundary, NOT a Complete Security Sandbox**  
> Assigning separate TCP ports on `127.0.0.1` resolves local DNS and HTTP routing issues, but does NOT create an airtight operating system sandbox or total browser partition. Developers and security auditors must understand the exact boundaries below.

### 2.1 Browser Same-Origin Policy (SOP) vs RFC 6265 Cookie Scoping
- **Same-Origin Policy (SOP)**: In modern browsers, an Origin is defined as a tuple of `(Protocol, Host, Port)`.
  - `http://127.0.0.1:18081` and `http://127.0.0.1:18082` are **DIFFERENT ORIGINS**.
  - Cross-origin DOM access, `localStorage`, `sessionStorage`, `IndexedDB`, and `fetch()` requests are subject to strict CORS controls. Project A cannot directly inspect Project B's DOM or storage via client script.
- **RFC 6265 Cookie Scoping (The Port Blind Spot)**:
  - Standard HTTP cookies **IGNORE PORT NUMBERS** by default (RFC 6265 Section 8.5).
  - A cookie set by `http://127.0.0.1:18081/` with default domain scoping is attached by the browser to subsequent requests to `http://127.0.0.1:18082/`!
  - If both projects run default PHP session configurations (`session.name = PHPSESSID`), Project A and Project B will overwrite each other's session IDs, causing random logouts and session corruption.

### 2.2 Reconciled Mitigation Strategy for Cookie & Session Isolation
1. **Per-Project Session Cookie Names**: LocalForge injects a project-specific session name into each FastCGI pool configuration:
   ```ini
   php_value[session.name] = LF_SESS_proj_db5472bb
   ```
2. **Per-Project Session Save Path**: FastCGI pools use isolated session storage directories:
   ```ini
   php_value[session.save_path] = "/Users/mehmetakar/Library/Application Support/LocalForge/runtime/sessions/proj_db5472bb"
   ```
3. **Loopback IP Alias Partitioning (Recommended Architectural Enhancement)**:
   - macOS loopback interface `lo0` routes the entire `127.0.0.0/8` block (16.7 million IPs) to localhost without any configuration.
   - Assigning `127.0.0.2:18081` to Project A and `127.0.0.3:18082` to Project B partitions the host domain itself, completely eliminating RFC 6265 cookie collisions across projects.

---

## 3. Comprehensive Analysis of the 14 Technical & Security Factors

### 1. Host-Header Validation & Port Ownership
- **Problem**: In a shared port setup, spoofing `Host:` can misroute requests.
- **Reconciliation**: In the multi-port setup, each `server` block binds to a specific `listen 127.0.0.1:<port> default_server;`. Even if a client sends an arbitrary or spoofed `Host` header, Nginx serves strictly the document root and FastCGI socket associated with that specific port.

### 2. Cookie Sharing Between Different Ports on `127.0.0.1`
- **Problem**: As established in RFC 6265, cookies do not isolate by port.
- **Reconciliation**: Mitigated via unique `session.name`, isolated `session.save_path`, and optional distinct `127.0.0.X` loopback IP assignments.

### 3. Browser Same-Origin Policy (SOP) vs Cookie Scoping
- **Reconciliation**: SOP protects DOM, `localStorage`, `IndexedDB`, and CacheStorage across distinct ports. Cookie scoping does not; cookie isolation must be handled explicitly at the application/runtime configuration layer.

### 4. Session Namespace Isolation
- **Reconciliation**: PHP sessions are physically separated on disk in Application Support. Permissions on session directories are `0700`, preventing cross-project inspection or session hijacking.

### 5. Port Allocation Races with Unrelated Local Processes (TOCTOU)
- **Problem**: Time-of-Check to Time-of-Use (TOCTOU). Probing `is_port_available(port)` in Rust and then launching Nginx leaves a time window where an unrelated local daemon (e.g. Docker, Node.js) could bind the port.
- **Reconciliation**: SQLite port reservation does NOT equal operating system port ownership! LocalForge must treat port allocation as an atomic database reservation, but handle OS bind failures gracefully at Nginx startup:
  - If Nginx fails to bind with `EADDRINUSE`, LocalForge detects the failure, diagnoses the occupying PID via Darwin `lsof`, notifies the user, and reallocates an alternative port without terminating foreign processes.

### 6. Port Persistence and Conflict Recovery
- **Reconciliation**: A project's `allocated_port` is stored in SQLite (`projects.allocated_port`). Upon host restart, LocalForge attempts to reuse the same port to preserve browser bookmarks and application configurations. If occupied, the system provides a non-destructive conflict recovery flow.

### 7. Safe Behavior When the Assigned Port Cannot Be Bound
- **Reconciliation**: Fail-closed! If port 18081 is blocked:
  - Nginx configuration generation rolls back.
  - Project status transitions to `PortConflict`.
  - The UI presents an actionable modal explaining the conflict with the foreign PID and executable path, allowing the user to click "Reassign Port" (e.g. to 18083).

### 8. Nginx Configuration Regeneration and Rollback
- **Problem**: A corrupt Nginx configuration could take down all running projects.
- **Reconciliation**: Configuration updates must be atomic:
  1. Write candidate configuration to temporary file `nginx.conf.candidate_<uuid>`.
  2. Validate candidate syntax: `nginx -t -c nginx.conf.candidate_<uuid>`.
  3. If validation passes, atomically rename over `nginx.conf` and issue `nginx -s reload`.
  4. If validation fails, discard candidate, retain active running configuration, log error, and alert user.

### 9. Unknown or Stopped Project Behavior
- **Problem**: What happens if a user navigates to a port of a stopped project?
- **Reconciliation**:
  - If Project B is stopped, its `server` block in Nginx does NOT route to Project A or default server.
  - Nginx returns an honest `HTTP 503 Service Unavailable` with a dedicated LocalForge maintenance page: *"LocalForge: Proje başlatılmamış veya durdurulmuş (Project Stopped)"*.
  - Wrong-project routing is strictly prohibited under all circumstances.

### 10. Laravel, WordPress, CodeIgniter and PHP Application Compatibility
- **Problem**: Web applications frequently construct absolute URLs:
  - **WordPress**: Stores `siteurl` and `home` options in the database (e.g. `http://127.0.0.1:18081`). If the port changes unpredictably, CSS/JS assets fail to load.
  - **Laravel**: Relies on `APP_URL=http://127.0.0.1:18081`. The `route()` and `asset()` helpers depend on this configuration.
  - **CodeIgniter**: Uses `$config['base_url']`.
- **Reconciliation**:
  - Ports must be **persistent** in SQLite to avoid breaking hardcoded application URLs.
  - FastCGI parameters must accurately convey port information:
    ```nginx
    fastcgi_param SERVER_PORT $server_port;
    fastcgi_param HTTP_HOST $http_host;
    fastcgi_param SERVER_NAME $server_name;
    ```

### 11. Absolute Redirects & Location Headers
- **Problem**: When PHP sends `header("Location: /dashboard")`, Nginx or PHP might rewrite it to `http://127.0.0.1/dashboard` (omitting the non-standard port 18081).
- **Reconciliation**: Enforce in Nginx:
  ```nginx
  port_in_redirect on;
  server_name_in_redirect off;
  ```
  This ensures that relative 301/302 redirects preserve the client's actual connected port.

### 12. WebSockets & HTTP/HTTPS Constraints
- **Problem**: LocalForge uses unencrypted HTTP on loopback. Modern browser features (Service Workers, Web Crypto API, Clipboard API) require a "Secure Context".
- **Reconciliation**:
  - The W3C Secure Contexts specification and all major browsers (Chromium, Safari WebKit, Firefox) explicitly designate `http://127.0.0.1:*` and `http://localhost:*` as **Potentially Trustworthy / Secure Contexts**.
  - Local developers can use Service Workers and Web APIs on `127.0.0.1:18081` without generating or trusting self-signed SSL certificates.
  - WebSocket connections (`ws://127.0.0.1:18081/ws`) are fully supported by Nginx via `proxy_set_header Upgrade $http_upgrade;`.

### 13. Existing `.test` Domains & Backward Compatibility
- **Reconciliation**:
  - The `local_domain` column (e.g. `projetest.test`) is NOT deleted.
  - Nginx `server` blocks declare:
    ```nginx
    listen 127.0.0.1:18082;
    server_name projetest.test 127.0.0.1;
    ```
  - If a user configures `/etc/hosts` or a local DNS proxy, `.test` domains continue to function identically.

### 14. APFS Unmount & Stale FastCGI Socket Defenses
- **Problem**: If `/Volumes/Mac Harici Disk` is disconnected while Nginx is running, requests to the docroot could hang or crash.
- **Reconciliation**:
  - All FastCGI Unix domain sockets and PID files reside on internal SSD (`Application Support/LocalForge/runtime/run/`).
  - If the external disk is unmounted, Nginx cannot access the document root.
  - LocalForge's `VolumeBarrier` trips immediately upon APFS container status changes, transitions the project to `VolumeDisconnected`, and Nginx returns `HTTP 502 Bad Gateway / Volume Disconnected` instead of corrupting data or attempting fallback writes to the internal drive.

---

## 4. SQLite Schema & Rust Model Migration Design (For Future Implementation)

```sql
-- Planned schema migration for Gate U5.B (SUBJECT TO PO APPROVAL)
ALTER TABLE projects ADD COLUMN loopback_port INTEGER UNIQUE;
```

Allocation Range: `18081` to `18999`.

---

## 5. Summary: Security Boundaries of Loopback Routing

| Layer | Boundary Provided | Security Guarantee |
|---|---|---|
| **Operating System** | Unprivileged user process | Cannot bind ports < 1024 without root; no privilege escalation |
| **Network Interface** | `127.0.0.1` (loopback only) | Immune to external LAN/WAN eavesdropping or network scans |
| **HTTP Routing** | Dedicated TCP Port | Physical port-to-FastCGI pool mapping; no Host-header confusion |
| **Browser DOM / Storage** | Same-Origin Policy (SOP) | Separate Origins; cross-project script access blocked |
| **Browser Cookies** | Shared `127.0.0.1` domain | **NOT isolated by default**; requires custom session names/paths |
| **Filesystem / OS User** | Shared Unix user `mehmetakar` | **NOT an OS sandbox**; malicious PHP code can read user files if not restricted by `open_basedir` |

**Conclusion**: Dedicated loopback ports solve local URL routing, browser reachability, and origin isolation with zero DNS configuration. They are a routing architecture and usability standard, not an operating system container.
