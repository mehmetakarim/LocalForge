# Phase 1 — Desktop Foundation & Workspace Storage (PLAN ONLY)

- **Phase ID**: `PHASE-1`
- **Phase Title**: Desktop Foundation & Workspace Storage
- **Status**: **COMPLETED (All automated tests pass, zero warnings)**
- **Prerequisites**: Phase 0 Formal Approval (Achieved).
- **Authorized Git Branch**: `phase-1/desktop-foundation`

---

## 1. Phase 1 Objectives & Scope

Phase 1 establishes the operational core and desktop application shell for LocalForge:
1. **Desktop Application Scaffolding**: Setup Tauri 2 shell with a React 19 + TypeScript + Vite + Tailwind CSS frontend.
2. **Modular Cargo Workspace**: Initialize root `Cargo.toml` and foundational core crates:
   - `crates/core`: Common domain types, IDs, error handling (`LocalForgeError`).
   - `crates/platform`: macOS platform bindings (IOKit, DiskArbitration, Keychain).
   - `crates/storage`: APFS volume identification, disconnect barrier, SQLite database.
   - `crates/project-manager`: Manifest parser (`project.yaml`) and project registry.
3. **External Volume Management**:
   - Resolve and record persistent volume UUIDs.
   - Monitor volume mount/unmount events via macOS DiskArbitration framework.
   - Enforce the directory creation barrier when external volumes are detached.
4. **Local Metadata Storage**:
   - Initialize internal SQLite database (`~/Library/Application Support/LocalForge/localforge.db`) in WAL mode.
   - Implement project indexing and fast search.
5. **Desktop UI Baseline**:
   - Project list view displaying registered projects, document roots, and runtime labels.
   - Storage health indicator showing connected volume status and disk space.

---

## 2. Planned Work Packages & Task Breakdown

### TASK-101: Tauri 2 & React Frontend Initialization
- Setup `apps/desktop` with `pnpm create tauri-app` or manual clean scaffolding.
- Configure Tailwind CSS, dark/light theme tokens, and accessible component layout.
- Ensure macOS window styling with native titlebar integration.

### TASK-102: Cargo Workspace & Core Crates
- Create root `Cargo.toml` with workspace members.
- Implement strongly typed domain primitives: `ProjectId`, `VolumeId`, `PathBuf` wrappers.
- Implement unified error enum `LocalForgeError` using `thiserror`.

### TASK-103: macOS APFS Volume Detection & Safety Guards
- Bind macOS `DiskArbitration` and `IOKit` APIs in `crates/platform/src/macos/`.
- Query volume UUID and mount paths.
- Implement the disconnect guard preventing file recreation under `/Volumes/`.

### TASK-104: Project Manifest & SQLite Metadata Store
- Define `serde`-serializable structs for `.localforge/project.yaml`.
- Setup `rusqlite` database schema with migrations.
- Implement two-way sync: manifest on disk -> SQLite index.

### TASK-105: UI Project List & Drive Status Indicator
- React components: `ProjectCard`, `DriveStatusBadge`, `ImportProjectModal`.
- Tauri IPC commands linking UI to Rust `crates/storage` and `crates/project-manager`.

### TASK-106: Automated Test Suite & Acceptance Verification
- Unit tests for manifest serialization, path canonicalization, and UUID resolution.
- Verification matrix update and Phase 1 closure review.

---

## 3. Planned Acceptance Criteria

- [ ] Desktop application launches on Apple Silicon with < 50MB idle memory footprint.
- [ ] User can register a project located on an external APFS drive (`/Volumes/...`).
- [ ] Disconnecting the external drive transitions project to `DISCONNECTED` within 1 second.
- [ ] No directory or file is recreated on the internal disk while the external drive is detached.
- [ ] Reconnecting the drive restores project status to `CONNECTED` automatically.
- [ ] SQLite database stores metadata reliably on internal disk with WAL mode enabled.
- [ ] All automated unit tests in `crates/*` pass with zero warnings.

---

## 4. Execution Guard

Do not create files in `apps/desktop/` or `crates/*` until the Product Owner confirms acceptance of Phase 0.
