# LocalForge Information Architecture & Screen Navigation

**Document Version**: 1.0.0  
**Status**: APPROVED DESIGN BASELINE  
**Focus**: Structural Taxonomy, Screen Flows & Future Extensibility  

---

## 1. Global Navigation Taxonomy

LocalForge organizes developer workflows into seven primary modules accessible from the **Floating Vertical Navigation Dock**, accompanied by four explicitly reserved future extensibility modules:

```
+========================================================================================+
| PRIMARY MODULES (Active & Phase R1-R2 Targets)                                         |
+----------------------------------------------------------------------------------------+
| 1. [Studio]       Atmospheric spatial overview of all active projects & services       |
| 2. [Projects]     Workspace catalog, manifest editor, path management, git status     |
| 3. [Services]     Process supervisors: PHP-FPM pools, Nginx gateway, MariaDB daemon    |
| 4. [Storage]      APFS volume topology, UUID verification, mount health & free space   |
| 5. [Deployments]  Audited diffs, SHA-256 ledger, preview generator, approval barrier  |
| 6. [AI / MCP]     Model Context Protocol inspector, tool catalog, audit telemetry      |
| 7. [Settings]     Runtime discovery paths, unprivileged ports, macOS Keychain, themes  |
+========================================================================================+
| RESERVED EXTENSIBILITY MODULES (Future Roadmap — Clearly Labeled "Roadmap")            |
+----------------------------------------------------------------------------------------+
| 8. [Remote Servers]     SSH/SFTP server targets, remote specs, key authentication     |
| 9. [Website Monitor]    HTTP uptime probes, SSL certificates, latency telemetry        |
| 10.[Alerts & Incidents] Threshold triggers, crash reports, notification history        |
| 11.[Deployment Stream]  Multi-project release audit ledger & timeline                  |
+========================================================================================+
```

---

## 2. Screen & View Specifications (Dual-Mode Mapping)

Every primary module provides both a **Spatial Studio** macro view and a **Native Workspace** micro view:

### 2.1 Module 1: Studio (`/studio`)
- **Spatial Studio Mode (Default)**:
  - Atmospheric 3D stage featuring interactive spatial cards representing each active project and backing daemon.
  - Ambient glow represents system health (indigo = calm/healthy, amber = port conflict or degraded pool, crimson = process crash, magenta = unmounted external drive).
  - Quick action floating bar: Create Project, Quick Start All, Diagnostic Pulse.
- **Native Workspace Mode**:
  - High-density status matrix table displaying process IDs (PIDs), CPU %, RAM consumption, and active loopback virtual host bindings.

### 2.2 Module 2: Projects (`/projects`)
- **Spatial Studio Mode**:
  - Grid of spatial project blocks showing visual representations of document roots, PHP version badges (e.g. `8.4`, `8.3`), domain shortcuts (`project.test`), and external volume anchor badges.
- **Native Workspace Mode**:
  - Two-pane split view:
    - Left pane: Filterable project tree (by volume, PHP version, running status).
    - Right pane: Deep manifest editor (`.localforge/project.yaml`), document root picker, PHP-FPM pool tuning parameters, and environment variable preview (with secrets masked).

### 2.3 Module 3: Services (`/services`)
- **Spatial Studio Mode**:
  - Spatial architecture diagram showing the relationship between Nginx (port 8080/8443) -> Unix domain sockets -> PHP-FPM pools, and MariaDB (port 3306).
- **Native Workspace Mode**:
  - Dedicated tabbed service consoles:
    - Tab A: PHP-FPM Supervisor (pool configs, socket bindings, worker count).
    - Tab B: Nginx Reverse Proxy (generated `nginx.conf` preview, virtual hosts, reload logs).
    - Tab C: MariaDB (socket path, data directory on external/internal drive, process health).
    - Tab D: Live System Log Stream with search, regex filtering, and copy options.

### 2.4 Module 4: Storage (`/storage`)
- **Spatial Studio Mode**:
  - 3D visual disks showing internal SSD vs external APFS drives (e.g. `/Volumes/Mac Harici Disk`).
  - Spatial capacity gauges showing free space, total size, and volume mount status.
- **Native Workspace Mode**:
  - Detailed volume inventory:
    - APFS UUID, BSD disk node (`/dev/disk3s1`), filesystem flags.
    - Safety barrier status: Volume Disconnect Guard active.
    - List of projects hosted on each volume with migration tools.

### 2.5 Module 5: Deployments (`/deployments`)
- **Spatial Studio Mode**:
  - Visual release staging card displaying local project -> remote staging/production target.
  - Preview diff card with added/modified/deleted file count indicators.
  - Prominent **Fail-Closed Security Notice**: Real remote file upload is safely locked in this release; only dry-run cryptographic simulation is permitted.
- **Native Workspace Mode**:
  - Cryptographic deployment diff explorer with tree and unified diff views.
  - SHA-256 ledger viewer (`.localforge/deployments/<target>.json`).
  - Secret exclusion validator proving `.env`, `.git/`, and private keys are excluded.

### 2.6 Module 6: AI / MCP (`/mcp`)
- **Spatial Studio Mode**:
  - Model Context Protocol status orb showing active agent connection state.
  - Security perimeter visual: Human Authorization Boundary active.
- **Native Workspace Mode**:
  - MCP tool registry viewer (`localforge_list_projects`, `localforge_project_status`, `localforge_runtime_diagnostics`, `localforge_deploy_preview`, `localforge_deploy_execute`).
  - Audit log recording every tool invocation requested by Claude Code, Codex, or Antigravity with timestamps, arguments, and outcomes.

### 2.7 Module 7: Settings (`/settings`)
- **Unified Workspace Layout**:
  - Homebrew PHP discovery directories (`/opt/homebrew/opt`, `/usr/local/opt`).
  - Port configurations (Nginx loopback, MariaDB unprivileged port).
  - macOS Keychain integration status.
  - Appearance preferences (Dark / Light / Match macOS).
  - Telemetry & Security policies.

---

## 3. Structural Screen Wireframe (Spatial Studio Master)

```
+----------------------------------------------------------------------------------------------------+
| (•)(•)(•)  LocalForge   [ Spatial Studio | Workspace ]                         Mac Harici Disk  931G |
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|  [DOCK]   +-------------------------------------------------------------------------------------+  |
|  +----+   |  ACTIVE WORKSPACES (3)                                           [+ New Project]    |  |
|  | ◉  |   |                                                                                     |  |
|  | ❖  |   |  +--------------------------+  +--------------------------+  +--------------------+  |  |
|  | ⚙  |   |  | StepperSkip Store        |  | LocalForge Core          |  | Client CRM API     |  |  |
|  | ⊞  |   |  | ● RUNNING   PHP 8.4      |  | ○ STOPPED    PHP 8.3     |  | ⚠ DEGRADED         |  |  |
|  | ⎋  |   |  | http://stepperskip.test  |  | http://localforge.test   |  | Port conflict 8080 |  |  |
|  | ✦  |   |  | ⛁ Mac Harici Disk        |  | ⛁ Mac Harici Disk        |  | ⛁ Macintosh HD     |  |  |
|  | ⚙  |   |  | [Open Browser] [Logs]    |  | [Start] [Settings]       |  | [Diagnose Conflict]|  |  |
|  +----+   |  +--------------------------+  +--------------------------+  +--------------------+  |  |
|           |                                                                                     |  |
|           |  RUNTIME SERVICES                                                                   |  |
|           |  +------------------+  +------------------+  +------------------+                   |  |
|           |  | PHP-FPM (3 Pools)|  | Nginx Gateway    |  | MariaDB 11.4     |                   |  |
|           |  | ● Active (3 PIDs)|  | ● Loopback :8080 |  | ● Port 3306      |                   |  |
|           |  +------------------+  +------------------+  +------------------+                   |  |
|           +-------------------------------------------------------------------------------------+  |
|                                                                                                    |
|                       [ ✦ Ask LocalForge AI or type a command... (⌘K) ]                            |
+----------------------------------------------------------------------------------------------------+
```

---

## 4. Keyboard Navigation & Accessibility Model

- `Cmd+1`: Switch to Studio
- `Cmd+2`: Switch to Projects
- `Cmd+3`: Switch to Services
- `Cmd+4`: Switch to Storage
- `Cmd+5`: Switch to Deployments
- `Cmd+6`: Switch to AI / MCP
- `Cmd+,`: Open Settings
- `Cmd+K`: Focus Floating AI Command Capsule
- `Cmd+R`: Refresh live system telemetry
- `Cmd+Shift+S`: Toggle Spatial Studio Mode
- `Cmd+Shift+W`: Toggle Native Workspace Mode
- `Escape`: Close modals, dismiss command bar, deselect spatial card
