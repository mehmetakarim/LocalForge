# LocalForge Architecture Overview

This document provides a comprehensive structural overview of the **LocalForge** system architecture.

---

## 1. System Topology

LocalForge is architected around a **Shared Rust Core** that powers three primary user interfaces: the Desktop Application (Tauri 2), the Command Line Interface (CLI), and the Model Context Protocol (MCP) Server for AI agent pairing.

```
+-----------------------------------------------------------------------------------+
|                                  USER SURFACES                                    |
|                                                                                   |
|  +------------------------+   +----------------------+   +---------------------+  |
|  |   Desktop App (GUI)    |   |    LocalForge CLI    |   |   LocalForge MCP    |  |
|  | Tauri 2 + React + Vite |   | (tools/localforge)   |   | (tools/mcp-server)  |  |
|  +------------------------+   +----------------------+   +---------------------+  |
+-----------------------------------------------------------------------------------+
                                         | (Typed IPC / IPC / StdIO)
                                         v
+-----------------------------------------------------------------------------------+
|                                SHARED RUST CORE                                   |
|                                                                                   |
|  +-----------------------+ +------------------------+ +------------------------+  |
|  |     crates/core       | | crates/project-manager | |     crates/runtime     |  |
|  | (types, errors, conf) | | (manifest, state, reg) | | (Nginx, PHP, MariaDB)  |  |
|  +-----------------------+ +------------------------+ +------------------------+  |
|                                                                                   |
|  +-----------------------+ +------------------------+ +------------------------+  |
|  |    crates/storage     | |   crates/deployment    | |    crates/platform     |  |
|  | (volume id, APFS, db) | | (ledger, sftp, diff)   | | (macOS, Launchd, key)  |  |
|  +-----------------------+ +------------------------+ +------------------------+  |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                             HOST & RUNTIME ENVIRONMENT                            |
|                                                                                   |
|  +--------------------+ +--------------------+ +-------------------------------+  |
|  | Internal SSD / APFS| | External SSD / APFS| | macOS Keychain / Launchd     |  |
|  | SQLite & App Data  | | Project Workspaces | | User unprivileged daemons     |  |
|  +--------------------+ +--------------------+ +-------------------------------+  |
+-----------------------------------------------------------------------------------+
```

---

## 2. Core Modules Breakdown

### 2.1 `crates/core`
- Common domain models, strongly typed ID wrappers (`ProjectId`, `RuntimeId`, `DeploymentId`), standard error types (`LocalForgeError`), unified logging, and configuration abstractions.

### 2.2 `crates/platform`
- Platform isolation layer.
- First-class implementation for **macOS Apple Silicon (`arm64`)**:
  - Volume detection & persistent identity (IOKit / DiskArbitration API bindings).
  - Secure credential storage (macOS Security Framework / Keychain).
  - Child process supervisory management without root privileges.
  - Network port availability check (loopback binding).
- Abstract traits for future Windows adapters (`CredentialStore`, `VolumeManager`, `ProcessSupervisor`).

### 2.3 `crates/storage`
- Volume identification via hardware UUID / filesystem UUID.
- Disconnect detection and safety barriers preventing erroneous filesystem recreation.
- Local SQLite metadata storage (using WAL mode and atomic transactions).
- Project root path canonicalization and symlink breakout validation.

### 2.4 `crates/project-manager`
- Discovery, registration, and status lifecycle of local PHP projects.
- Parses and serializes `.localforge/project.yaml` manifests.
- Multi-domain routing table maintenance.

### 2.5 `crates/runtime`
- Orchestrates PHP-FPM pools, Nginx server blocks, and MariaDB instances.
- Process supervisory tree, graceful reload/stop, port conflict resolution.
- Health-check probes (HTTP ping to FPM status and Nginx).

### 2.6 `crates/deployment`
- Git working-tree vs. last-deployed checksum ledger.
- Exclusion engine (default-deny for secrets, `.env`, `.git`).
- SFTP transfer engine with resume support, upload integrity validation (SHA-256), and deployment preview generation.

---

## 3. Storage Hierarchy & Volume Strategy

| Storage Category | Default Location | Persistence Policy |
|---|---|---|
| **Project Source** | Internal disk OR External SSD (`/Volumes/...`) | User owned, strictly preserved |
| **Project Manifest** | `<project-root>/.localforge/project.yaml` | Git-committable, non-sensitive |
| **LocalForge App Data** | `~/Library/Application Support/LocalForge/` | Internal disk, SQLite database |
| **Runtime Binaries** | Managed bundle or isolated user directory | Internal disk cache / external option |
| **MariaDB Data** | Internal disk (default) or explicit external mount | Internal SSD preferred for stability |
| **Secrets & Keys** | macOS Keychain | Hardware-backed, never in files |

---

## 4. Interfaces

1. **Desktop GUI (`apps/desktop`)**:
   - Tauri 2 wrapper with React 19 / Vite / Tailwind frontend.
   - Invokes Rust commands via Tauri `invoke` IPC with strictly typed inputs/outputs.
2. **CLI (`tools/cli`)**:
   - Built with `clap`. Directly links to `crates/*` for zero-overhead headless management.
3. **MCP Server (`tools/mcp-server`)**:
   - Implements Model Context Protocol over `stdio`. Exposes typed tools for Claude Code, Codex, and AI assistants.

---

## 5. Security & Isolation Boundaries

- **Least Privilege**: Services run under the unprivileged user account. Unprivileged ports (`127.0.0.1:8080`, `127.0.0.1:9000`, etc.) with local reverse proxying avoid `sudo`.
- **Path Sanitization**: Every path received from UI, CLI, or MCP is validated with `dunce::canonicalize` and checked for prefix containment against registered project roots.
- **Auditable Deployments**: File uploads to remote servers require an explicit user-approved deployment preview artifact.
