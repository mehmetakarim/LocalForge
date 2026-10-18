# LocalForge — Decisions Index

This index tracks all Architecture Decision Records (ADRs) established for the LocalForge project. Detailed decision rationales, alternatives, consequences, and approval statuses are recorded in [`docs/architecture/adr/`](../docs/architecture/adr/).

---

## Architecture Decision Records (ADRs)

| ADR ID | Title | Status | Date | Primary Driver |
|---|---|---|---|---|
| [**ADR-0001**](../docs/architecture/adr/ADR-0001-tauri-2-rust-modular-workspace.md) | Tauri 2 & Modular Rust Workspace | ACCEPTED | 2026-10-18 | Native macOS performance, low memory footprint (<50MB vs >400MB Electron), strict IPC security. |
| [**ADR-0002**](../docs/architecture/adr/ADR-0002-shared-core-separation.md) | Shared Core Separation for Desktop, CLI, and MCP | ACCEPTED | 2026-10-18 | Single source of truth across all user interfaces; prevents logic duplication. |
| [**ADR-0003**](../docs/architecture/adr/ADR-0003-macos-first-platform-abstraction.md) | macOS-First Platform Abstraction (`arm64`) | ACCEPTED | 2026-10-18 | Apple Silicon tier-1 optimization, native Keychain, and IOKit volume identification with trait decoupling for future Windows support. |
| [**ADR-0004**](../docs/architecture/adr/ADR-0004-external-workspace-storage-model.md) | External Workspace Storage & Volume Disconnect Safety | ACCEPTED | 2026-10-18 | Hardware/UUID volume tracking, graceful disconnect handling, and preventing unintended internal disk file recreation. |
| [**ADR-0005**](../docs/architecture/adr/ADR-0005-runtime-distribution-strategy.md) | Runtime Distribution & Isolated FPM Pools Strategy | ACCEPTED | 2026-10-18 | Hybrid strategy: Homebrew discovery as bootstrap with managed sandboxed binaries as the target architecture. |
| [**ADR-0006**](../docs/architecture/adr/ADR-0006-secure-deployment-architecture.md) | Git-Aware Checksum Deployment Ledger & Security | ACCEPTED | 2026-10-18 | Independent deployment ledger instead of raw git diff; default-deny sensitive files; human confirmation required. |
| [**ADR-0007**](../docs/architecture/adr/ADR-0007-project-manifest-metadata-model.md) | Project Manifest (`project.yaml`) & SQLite Architecture | ACCEPTED | 2026-10-18 | Portable, version-controlled `.localforge/project.yaml` without secrets; fast local queries via internal SQLite store. |
| [**ADR-0008**](../docs/architecture/adr/ADR-0008-project-memory-and-phase-governance.md) | File-Based Project Memory & Sequential Phase Governance | ACCEPTED | 2026-10-18 | Cross-session continuity for Claude Code, Codex, and multi-agent teams with strict write isolation and honest verification. |
| [**ADR-0009**](../docs/architecture/adr/ADR-0009-application-mediated-human-approval.md) | Application-Mediated Human Approval & AI Agent Authorization Boundary | ACCEPTED | 2026-10-18 | Prevents AI agents from self-authorizing remote mutations; mandates out-of-band application-mediated approval. |
| [**ADR-0010**](../docs/architecture/adr/ADR-0010-spatial-studio-design-system.md) | Spatial Studio Design System Architecture & Dual-Mode UI | ACCEPTED | 2026-10-18 | Atmospheric 2.5D spatial depth, floating vertical dock, AI command capsule, and dual Spatial Studio/Native Workspace modes. |
| [**ADR-0011**](../docs/architecture/adr/ADR-0011-real-runtime-process-supervision.md) | Real Runtime Process Supervision & Non-Root Lifecycle Architecture | ACCEPTED | 2026-10-18 | Unprivileged child process spawning, async tokio supervision tree, FastCGI unix sockets, and orphan prevention. |
| [**ADR-0012**](../docs/architecture/adr/ADR-0012-process-identity-verification-and-storage-isolation.md) | Process Identity Verification, Lifecycle Safety & MariaDB Internal Storage Isolation | ACCEPTED | 2026-10-18 | Verified process identity preventing PID reuse kills; shared Nginx vs independent FPM pools; internal MariaDB storage isolation. |

---

## Status Definitions
- **PROPOSED**: Under engineering or product owner review; feedback actively requested.
- **ACCEPTED**: Approved architectural baseline; binding on subsequent development phases.
- **SUPERSEDED**: Replaced by a newer decision record (referenced explicitly).
- **REJECTED**: Evaluated and discarded with documented rationale.
