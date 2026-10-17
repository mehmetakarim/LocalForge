# ADR-0007: Project Manifest Schema & SQLite Metadata Architecture

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Senior Rust Engineer, Backend Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

LocalForge needs to persist project configuration and metadata. Requirements dictate:
1. **Portability**: A project moved to another machine or shared with a colleague via Git should retain its configuration (PHP version, document root, local domain pattern).
2. **Speed**: Querying 50+ projects in the Desktop GUI or CLI must be instantaneous without scanning and parsing disk files repeatedly.
3. **Security**: Secrets (database credentials, SFTP passwords, private keys) must never be committed to Git.

---

## 2. Decision

We implement a **Dual-Layer Metadata Architecture**:

### Layer 1: Portable Project Manifest (`.localforge/project.yaml`)
- Placed directly inside the project root directory.
- Versioned via schema header (`version: 1`).
- Defines project properties:
  ```yaml
  version: 1
  project:
    id: "proj_01j7..." # Unique KSUID/UUID
    name: "StepperSkip"
    type: "php"
    document_root: "public" # Relative to project root
  runtime:
    php_version: "8.2"
    web_server: "nginx"
    extensions:
      - "curl"
      - "pdo_mysql"
  routing:
    local_domain: "stepperskip.test"
    https: false
  storage:
    volume_uuid: "E289C823-7F01-..." # Hardware/filesystem UUID
  ```
- **Invariant**: **NO SECRETS** are permitted in this file.

### Layer 2: Fast Local SQLite Store (`localforge.db`)
- Located in `~/Library/Application Support/LocalForge/localforge.db` (internal SSD).
- Configured with `PRAGMA journal_mode = WAL;` and foreign key enforcement.
- Indexes all registered projects, volume mount histories, deployment ledgers, and runtime statuses.
- Provides sub-millisecond query response for GUI and CLI.
- Re-indexes or synchronizes with `project.yaml` when changes are detected.

---

## 3. Alternatives Considered

### Alternative A: SQLite Database Only (No `project.yaml`)
- *Rejected*: Projects would not be portable; sharing a project with a team member would lose all LocalForge runtime and domain settings.

### Alternative B: Storing Secrets in `project.yaml` with Encryption
- *Rejected*: Key management for encrypted Git files is complex, error-prone, and presents severe accidental leak risks. Secrets belong strictly in the OS Keychain.

---

## 4. Consequences & Risks

### Positive
- Projects are self-describing and portable across machines.
- Instantaneous UI listing via SQLite index.
- Absolute separation between public project config and private secrets.

### Negative / Risks
- Requires a two-way synchronization strategy if `project.yaml` is modified externally via text editor (addressed by watching file modification times).

---

## 5. Verification Status

- Schema design established. Implementation scheduled for Phase 1 (TASK-104).
