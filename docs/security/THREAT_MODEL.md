# LocalForge — Threat Model & Security Architecture

This document establishes the threat model, attack vectors, trust boundaries, and mitigation strategies for LocalForge.

---

## 1. System Asset Classification

| Asset Category | Description | Sensitivity | Primary Protection |
|---|---|---|---|
| **User Code & Workspaces** | Local project files on internal and external SSDs | High | Strict path bounds, read/write permissions |
| **Project Manifests** | `.localforge/project.yaml` metadata | Low / Medium | Public schema, strict absence of secrets |
| **Deployment Credentials** | SFTP passwords, private SSH keys, passphrases | Critical | macOS Keychain, memory zeroing on drop |
| **Remote Production Servers** | Remote websites and databases deployed via SFTP | Critical | Checksum ledger, human authorization gate |
| **Local Runtime Daemons** | Nginx, PHP-FPM, MariaDB processes | Medium | Loopback binding, unprivileged user execution |
| **Local SQLite Metadata** | LocalForge system registry and health database | Medium | Internal SSD storage, WAL mode, permissions |

---

## 2. Threat Analysis by Vector

### 2.1 Threat Vector 1: Privilege Escalation & Host Compromise
- **Threat**: Malicious code or configuration exploit attempting to gain root privileges through development daemons or helper utilities.
- **Impact**: Complete compromise of developer workstation.
- **Mitigation**:
  1. LocalForge enforces a strict **Least Privilege** policy.
  2. Background daemons (Nginx, PHP-FPM, MariaDB) run under the non-privileged user UID.
  3. No suid-root binaries or arbitrary `sudo` commands are executed.
  4. Network services bind to unprivileged ports (>1024).

### 2.2 Threat Vector 2: Path Traversal & Unintended File Destruction
- **Threat**: Maliciously crafted manifest paths (e.g. `document_root: "../../../etc"`) or untrusted symlinks in imported projects attempting to read or overwrite system files.
- **Impact**: Arbitrary file read/write, host system compromise.
- **Mitigation**:
  1. All paths are resolved using canonical path normalization (`dunce::canonicalize`).
  2. Relative paths are strictly validated to ensure they remain inside the registered project root (`starts_with`).
  3. Symlinks pointing outside the project boundary are rejected with `Error::PathTraversalViolation`.
  4. LocalForge never deletes user projects automatically; project removal requires explicit confirmation.

### 2.3 Threat Vector 3: Accidental Secret Disclosure via Deployment
- **Threat**: Deploying `.env` files, `.git/` repositories, local database dumps, or private keys to public remote web servers during file sync.
- **Impact**: Production database credentials, API keys, and source code leaks.
- **Mitigation**:
  1. **Default-Deny Exclusion Filter**: Built-in engine blocks `.env*`, `.git/**`, `*.pem`, `*.key`, `id_*`, and internal metadata regardless of user config.
  2. **Deployment Preview**: Every transfer generates a clear file list.
  3. **Mandatory Human Approval**: No file transfer begins until the user clicks approve.

### 2.4 Threat Vector 4: AI Agent Prompt Injection & Autonomous Actions (MCP)
- **Threat**: An AI assistant reading an untrusted third-party codebase containing prompt injection instructions (e.g., in a `README.md` or comment) attempting to instruct the agent to deploy altered files or run destructive tools via MCP.
- **Impact**: Unauthorized code deployment, malicious server modification.
- **Mitigation**:
  1. **Read-Safe Diagnostic Boundary**: Diagnostic tools (`localforge_project_status`, `localforge_runtime_diagnostics`) are read-only.
  2. **Hard Human Approval Gate**: The deployment execution tool (`localforge_deploy_execute`) requires interactive confirmation in the desktop application or CLI terminal. The MCP server rejects automated deployment requests lacking a valid, user-signed confirmation token.
  3. **No Secret Return via MCP**: MCP tools never return Keychain passwords, private keys, or credentials in tool response payloads.

### 2.5 Threat Vector 5: External Volume Disconnect & File Recreation Hazard
- **Threat**: An external drive holding a project workspace is suddenly detached. Background processes or IDEs attempt to write to `/Volumes/ExternalSSD/...`, creating phantom directories on the internal drive and causing data desynchronization.
- **Impact**: Project state desynchronization, hidden disk space leakage.
- **Mitigation**:
  1. Persistent volume tracking via hardware/filesystem UUID.
  2. Automated disconnect barrier: if UUID is absent, the project enters `DISCONNECTED` state and all write attempts are immediately halted.
  3. LocalForge never calls directory creation functions (`fs::create_dir_all`) under `/Volumes/` without verifying volume mount integrity.

### 2.6 Threat Vector 6: Network Exposure & Public Interface Binding
- **Threat**: Local development servers binding to `0.0.0.0`, exposing in-development PHP applications and MariaDB databases to local Wi-Fi networks (e.g. coffee shops).
- **Impact**: Unauthorized access to development databases and web applications.
- **Mitigation**:
  1. All services bind strictly to loopback (`127.0.0.1` and `::1`).
  2. Binding to public interfaces is disabled by default and requires explicit configuration with a security warning.

### 2.7 Threat Vector 7: Supply Chain & Dependency Tampering
- **Threat**: Vulnerable or malicious dependencies introduced via Cargo or npm packages.
- **Impact**: Code execution during build or runtime.
- **Mitigation**:
  1. Lockfiles (`Cargo.lock`, `package-lock.json`) committed to Git.
  2. Automated dependency auditing via `cargo audit` and `npm audit`.
  3. Minimal dependency footprint: reject bloated or unmaintained libraries.
