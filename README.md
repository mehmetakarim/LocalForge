# LocalForge

[![License: Apache-2.0 / MIT](https://img.shields.io/badge/License-Apache_2.0_OR_MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20Apple%20Silicon-lightgrey.svg)](docs/architecture/adr/ADR-0003-macos-first-platform-abstraction.md)
[![Phase](https://img.shields.io/badge/phase-0%20Foundation-orange.svg)](docs/phases/PHASE_0_FOUNDATION.md)

> **An open-source, AI-ready local development environment that makes PHP project management, external workspace storage, and secure deployment simple.**

---

## 🌟 Overview

LocalForge is modern developer tooling built for PHP engineers, digital agencies, and independent creators who require native macOS performance, seamless work from external SSDs, and AI-assisted workflows without subscription lock-in or fragile global runtime mutations.

LocalForge combines:
- **Tauri 2 & Rust Core**: Blazing fast, minimal memory footprint, and native macOS security integration.
- **External Storage as a First-Class Citizen**: Work effortlessly across internal SSDs and high-speed external volumes (`/Volumes/...`) with robust disconnect guards and volume UUID tracking.
- **Project-Aware Runtimes**: Isolated PHP-FPM pools, Nginx virtual hosts, and MariaDB instances without conflicting global packages.
- **Audited Smart Deployment**: Git-aware file change diffs, checksum ledgers, and secure SFTP deployment previews with human authorization gates.
- **AI-Native & MCP Integration**: Native Model Context Protocol (MCP) server sharing the same trusted Rust core as the Desktop UI and CLI.

---

## 🧱 Technology Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Desktop Shell** | [Tauri 2](https://v2.tauri.app/) | Native webview, low memory (<50MB vs >400MB Electron), strict IPC security |
| **Frontend UI** | React 19 + TypeScript + Tailwind CSS | High productivity, accessible component patterns (shadcn/ui-compatible) |
| **Shared Core** | Rust (Modular Cargo Workspace) | High memory safety, fast process supervisory controls, cross-platform architecture |
| **Local Metadata** | SQLite (WAL mode) | Embedded, zero-maintenance, ACID transactional reliability |
| **Web Server** | Nginx | High-performance reverse proxy and static/fastcgi gateway |
| **PHP Engine** | PHP-FPM | FastCGI Process Manager supporting multiple concurrent versions |
| **Database** | MariaDB | MySQL-compatible relational database engine |
| **AI Protocol** | Model Context Protocol (MCP) | Direct standard integration with Claude Code, OpenAI Codex, and Antigravity |
| **Credentials** | macOS Keychain | Hardware-backed, secure key and token storage |

---

## 📐 Project Structure

```text
LocalForge/
├── apps/
│   └── desktop/            # Tauri 2 frontend & desktop application
├── crates/
│   ├── core/               # Shared domain primitives, types, errors, configuration
│   ├── platform/           # macOS Apple Silicon platform abstraction (Keychain, volumes)
│   ├── storage/            # External volume detection, path sanitization, SQLite store
│   ├── project-manager/    # Project manifest, registry, local domain routing
│   ├── runtime/            # Process supervisors for Nginx, PHP-FPM, MariaDB
│   └── deployment/         # Checksum ledger, SFTP client, diff analyzer
├── tools/
│   ├── cli/                # LocalForge command-line utility
│   └── mcp-server/         # Model Context Protocol stdio server
├── docs/
│   ├── architecture/       # System diagrams, specifications, and ADRs (ADR-0001+)
│   ├── product/            # Vision, Requirements, Feature Matrix, Non-Goals
│   ├── security/           # Threat Model, Security Policy
│   └── phases/             # Phase roadmaps, acceptance criteria, and closure reports
├── .project-memory/        # Cross-session AI & human canonical memory system
├── AGENTS.md               # Agent rules, multi-agent protocol, and phase boundaries
├── CLAUDE.md               # Claude Code orientation guide
├── brain.md                # Central project index & status dashboard
└── CONTRIBUTING.md         # Engineering guidelines & contributor instructions
```

---

## 🚦 Project Phases & Status

LocalForge follows a strict, verifiable phased engineering discipline:

| Phase | Description | Status |
|---|---|---|
| **Phase 0** | **Foundation & Project Memory** | **IN PROGRESS (Active)** |
| **Phase 1** | Desktop Foundation & Workspace Storage | *Planned (Pending PO Approval)* |
| **Phase 2** | Local Runtime Engine (PHP/Nginx/MariaDB) | *Planned* |
| **Phase 3** | Smart Deployment & CLI / MCP Integration | *Planned* |
| **Phase 4** | Hardening, Release & Open-Source Distribution | *Planned* |

See [`docs/phases/PHASE_0_FOUNDATION.md`](docs/phases/PHASE_0_FOUNDATION.md) for full details.

---

## 🔒 Security Baseline

- **No Root Required**: All local services run under the local developer user on unprivileged ports.
- **Default-Deny Deployment**: Sensitive files (`.env*`, `.git/`, SSH keys) are never deployed to remote servers.
- **Human Approval Gates**: Destructive operations or remote file synchronization always require explicit human confirmation.
- Read [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md) for complete details.

---

## 📄 License

LocalForge will be distributed under the dual **Apache-2.0 or MIT** license. See the upcoming `LICENSE` file for details.
