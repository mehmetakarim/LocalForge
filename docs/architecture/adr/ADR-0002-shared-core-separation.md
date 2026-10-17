# ADR-0002: Shared Core Separation Across Desktop, CLI, and MCP

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Senior Rust Engineer, Systems Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

LocalForge serves users across three primary execution contexts:
1. **Desktop GUI**: Visual management of projects, runtimes, domains, and disk usage.
2. **LocalForge CLI**: Headless terminal automation, CI/CD usage, and scripted workflows.
3. **Model Context Protocol (MCP) Server**: Direct AI assistant interaction (Claude Code, Codex, Antigravity).

If business rules (e.g. project manifest validation, runtime lifecycle controls, deployment checksums) are implemented inside the desktop UI or duplicated across interfaces, discrepancies and race conditions will inevitably emerge.

---

## 2. Decision

We mandate a **Single Shared Core** in Rust. The user interfaces (Desktop, CLI, MCP Server) are strictly thin presentation and transport adaptors that invoke typed service contracts provided by shared library crates:

```text
               +-------------------------------------------+
               |             PRESENTATION / API            |
               |                                           |
               | [apps/desktop]  [tools/cli]  [tools/mcp]  |
               +-------------------------------------------+
                                     |
                                     v
               +-------------------------------------------+
               |              SHARED RUST CORE             |
               |                                           |
               |   crates/project-manager  crates/runtime  |
               |   crates/storage          crates/platform |
               |   crates/deployment       crates/core     |
               +-------------------------------------------+
```

1. **No UI-Side Business Logic**: The React frontend must never maintain business state independently of the Rust core.
2. **Explicit Service Contracts**: Every operation is defined via typed structs, enums, and `Result<T, LocalForgeError>`.
3. **Transport Independence**: The core engine knows nothing about Tauri IPC, Clap CLI arguments, or MCP JSON-RPC protocols.

---

## 3. Alternatives Considered

### Alternative A: Desktop App as Primary Server (Daemon)
- *Pros*: Single running process.
- *Cons*: CLI and MCP cannot function when the desktop app is closed; desktop crashes would disrupt background tasks.

### Alternative B: Independent Codebases
- *Pros*: Rapid independent UI development.
- *Cons*: Severe logic drift, inconsistent deployment diffing, and multiple bugs. Explicitly prohibited.

---

## 4. Consequences & Risks

### Positive
- Guaranteed uniform behavior across Desktop, CLI, and MCP.
- High testability: core crates can be unit-tested without launching WebKit or UI mock engines.
- AI agents via MCP execute the exact same battle-tested code paths as human GUI clicks.

### Negative / Risks
- Requires disciplined API versioning between core crates and presentation layers.

---

## 5. Verification Status

- Architectural boundary confirmed in `docs/architecture.md` and enforced in `AGENTS.md`.
- Implementation structure scheduled for Phase 1 (TASK-102).
