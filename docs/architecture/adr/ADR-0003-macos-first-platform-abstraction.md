# ADR-0003: macOS-First Platform Abstraction Architecture

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, macOS Systems Engineer, Security Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

LocalForge's initial target platform is macOS Apple Silicon (`arm64`). macOS possesses distinct operating system characteristics:
- Hardware-backed Keychain for credentials.
- DiskArbitration and IOKit frameworks for volume lifecycle and hardware identification.
- APFS filesystem semantics (snapshots, fast clones, extended attributes).
- Launchd user daemons for background service management.
- Strict sandboxing and System Integrity Protection (SIP).

Future platform plans include Windows. If macOS-specific APIs are mixed into core domain code, porting to Windows will require massive rewrites. Conversely, an overly generic cross-platform abstraction created too early might degrade native macOS performance.

---

## 2. Decision

We establish a dedicated platform abstraction crate (`crates/platform`) with explicit Rust traits:
1. `VolumeManager`: Hardware/volume UUID detection, mount point polling, and event notifications.
2. `CredentialStore`: Secure key/token storage and retrieval.
3. `ProcessSupervisor`: Child process monitoring, signal dispatch (SIGTERM, SIGKILL), and exit status collection.
4. `NetworkInspector`: Loopback port availability and bind conflict checking.

**Implementation Priority**:
- **Tier 1 (Immediate)**: `crates/platform/src/macos/` utilizing native Apple frameworks (`security-framework`, `disk-arbitration`, `iokit-sys`, `nix`).
- **Tier 2 (Future / Stubbed)**: `crates/platform/src/windows/` designed via traits for future implementation using DPAPI / Windows Credential Manager and Win32 process APIs.

---

## 3. Alternatives Considered

### Alternative A: Using Generic Multi-Platform Crates Without Abstraction
- *Pros*: Faster initial assembly.
- *Cons*: Generic crates frequently lack deep macOS external disk arbitration and fail to surface hardware volume UUIDs, leading to fragile volume tracking.

### Alternative B: Cross-Platform Implementation in Parallel
- *Pros*: Early Windows support.
- *Cons*: Dilutes engineering focus and delays macOS stabilization. Violates product roadmap sequencing.

---

## 4. Consequences & Risks

### Positive
- Deep, first-class native macOS Apple Silicon optimization.
- Secure, hardware-backed Keychain storage for deployment passwords and tokens.
- Clear compiler-enforced trait boundaries for future platform adapters.

### Negative / Risks
- Requires direct C/Objective-C/CoreFoundation FFI bindings where pure Rust crates do not exist.

---

## 5. Verification Status

- Host platform verified: macOS 27.0 arm64 Darwin Kernel.
- Abstraction traits scheduled for Phase 1 (TASK-102 & TASK-103).
