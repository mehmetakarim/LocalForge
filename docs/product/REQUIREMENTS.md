# LocalForge — Requirements Specification

This document details the functional, nonfunctional, security, and environmental requirements for LocalForge.

---

## 1. Functional Requirements (FR)

### 1.1 Project & Workspace Management
- **FR-101 (Project Registration)**: The system must allow users to register existing PHP projects or create new empty projects on both internal and external storage volumes.
- **FR-102 (Portable Manifest)**: Every project must maintain a versioned `.localforge/project.yaml` manifest containing runtime configurations and domain routing metadata without secrets.
- **FR-103 (External Storage UUID Tracking)**: The system must associate registered projects with the underlying hardware/filesystem UUID of their storage volume to support dynamic mount path changes on macOS.
- **FR-104 (Disconnect Barrier)**: If a project's storage volume is unmounted, the system must transition the project status to `DISCONNECTED` and prohibit any automatic directory recreation on the host disk.
- **FR-105 (Project Metadata Fast Query)**: Project state, recent health metrics, and volume bindings must be indexed in a local SQLite database for sub-millisecond querying.

### 1.2 Local Runtime Management
- **FR-201 (Concurrent PHP Versions)**: The system must allow different registered projects to execute against different PHP versions (e.g., PHP 8.1 and 8.3) concurrently without global version conflicts.
- **FR-202 (Isolated FastCGI Pools)**: Each project must bind to an isolated Unix domain socket or loopback port managed by a dedicated PHP-FPM process pool.
- **FR-203 (Nginx Reverse Proxy)**: The system must manage a user-level Nginx reverse proxy that routes incoming HTTP requests on loopback addresses to the appropriate project FastCGI pool based on hostname (`.test`).
- **FR-204 (MariaDB Management)**: The system must manage a user-level MariaDB instance storing databases safely on internal storage by default.
- **FR-205 (Diagnostic Health Probes)**: The system must run periodic health checks on active runtime processes, collect localized logs, and detect port binding conflicts.

### 1.3 Smart Audited Deployment
- **FR-301 (Deployment Ledger)**: The system must track deployed files independently from Git using an audited local ledger recording per-file SHA-256 checksums and commit SHAs.
- **FR-302 (Change Diffing)**: The system must detect modified, added, deleted, and untracked files ready for deployment.
- **FR-303 (Default-Deny Exclusion)**: The deployment engine must strictly exclude sensitive files (`.env*`, `.git/**`, SSH keys, credentials) by default.
- **FR-304 (Deployment Preview & Human Gate)**: The system must generate a human-readable deployment preview and require explicit user authorization before initiating remote uploads.
- **FR-305 (SFTP File Transfer)**: The system must upload files over SFTP with support for transfer resumption and integrity verification.

### 1.4 AI & Interface Extensibility
- **FR-401 (Shared Core)**: Desktop GUI, CLI, and MCP Server must utilize the exact same Rust core crates.
- **FR-402 (MCP Tooling)**: The MCP server must expose typed tools (`localforge_list_projects`, `localforge_project_status`, `localforge_runtime_diagnostics`, `localforge_deploy_preview`, etc.).
- **FR-403 (MCP Human Gate Invariant)**: The MCP server must never execute production deployments or destructive local operations autonomously without explicit user approval.

---

## 2. Non-Functional Requirements (NFR)

### 2.1 Performance & Resource Footprint
- **NFR-001 (Memory Consumption)**: The Desktop application must consume less than 60 MB RAM in idle state.
- **NFR-002 (Startup Latency)**: Cold startup time on Apple Silicon Macs must be under 1.5 seconds.
- **NFR-003 (I/O Efficiency)**: File scanning on external APFS drives must use asynchronous I/O and respect disk read/write bandwidth.

### 2.2 Security & Isolation
- **NFR-101 (Least Privilege)**: All processes (LocalForge, Nginx, PHP-FPM, MariaDB) must run under the unprivileged local developer user account without requiring `sudo`.
- **NFR-102 (Network Boundary)**: All local development services must bind strictly to loopback (`127.0.0.1` / `::1`) by default.
- **NFR-103 (Credential Security)**: SFTP passwords, SSH passphrases, and API tokens must be stored in the macOS Keychain. Plain-text secret storage in configuration files is prohibited.
- **NFR-104 (Path Traversal Protection)**: All path inputs from GUI, CLI, or MCP must be canonicalized and checked against allowed project root boundaries.

### 2.3 Reliability & Maintainability
- **NFR-201 (Disaster Resilience)**: Sudden physical disconnection of an external drive must never cause a crash or corrupt the central SQLite database.
- **NFR-202 (Atomic State Mutations)**: Configuration files and deployment manifests must be written atomically via temporary files and rename operations.
- **NFR-203 (Test Coverage)**: Core domain logic must maintain >80% automated unit test coverage.
