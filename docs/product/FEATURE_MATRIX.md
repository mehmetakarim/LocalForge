# LocalForge — Feature Matrix & Competitive Comparison

This document charts the feature roadmap across phases and compares LocalForge against existing developer tools.

---

## 1. Feature Phase Matrix

| Feature Domain | Feature Description | Phase | Core Dependency | Interface Availability |
|---|---|---|---|---|
| **Storage** | Project Creation & Registration | Phase 1 | `crates/project-manager` | GUI, CLI |
| **Storage** | External Volume UUID Tracking | Phase 1 | `crates/storage`, `crates/platform` | GUI, CLI, MCP |
| **Storage** | Disconnect Barrier & Recovery | Phase 1 | `crates/storage` | GUI, CLI, MCP |
| **Storage** | SQLite Local Metadata Index | Phase 1 | `crates/storage` | Core (Internal) |
| **Runtime** | Isolated PHP-FPM Multi-Version Pools | Phase 2 | `crates/runtime` | GUI, CLI, MCP |
| **Runtime** | Loopback Nginx Virtual Hosts | Phase 2 | `crates/runtime` | GUI, CLI |
| **Runtime** | User-Level MariaDB Management | Phase 2 | `crates/runtime` | GUI, CLI |
| **Runtime** | Port Conflict Detection & Health Probes | Phase 2 | `crates/runtime` | GUI, CLI, MCP |
| **Deployment** | Git-Aware Checksum Ledger (SHA-256) | Phase 3 | `crates/deployment` | GUI, CLI, MCP |
| **Deployment** | Default-Deny Secret Filter | Phase 3 | `crates/deployment` | GUI, CLI, MCP |
| **Deployment** | Interactive Deployment Preview | Phase 3 | `crates/deployment` | GUI, CLI, MCP (Read-only) |
| **Deployment** | SFTP Transfer Engine with Resume | Phase 3 | `crates/deployment` | GUI, CLI |
| **Interfaces** | Headless CLI Utility | Phase 3 | `tools/cli` | Terminal |
| **Interfaces** | Model Context Protocol (MCP) Server | Phase 3 | `tools/mcp-server` | AI Assistants |
| **Security** | macOS Keychain Integration | Phase 1 | `crates/platform` | GUI, CLI, Core |

---

## 2. Competitive Landscape Comparison

| Capability | LocalForge | XAMPP / MAMP | Laravel Herd | Docker Desktop |
|---|---|---|---|---|
| **Core Architecture** | Tauri 2 + Rust | C/C++ + Cocoa/Qt | Swift / macOS Native | Go / Hypervisor (VM) |
| **Idle Memory Footprint** | **< 50 MB** | ~150–250 MB | ~80–120 MB | **> 2000 MB (2+ GB)** |
| **Open Source Core** | **Yes (Apache/MIT)** | XAMPP: GPL / MAMP: Proprietary | Proprietary (Paid Pro) | Mixed / Enterprise License |
| **Concurrent Multi-PHP** | **Yes (Per-project FPM)** | Cumbersome / Requires Pro | Yes | Yes (via containers) |
| **External SSD First-Class** | **Yes (UUID tracking + safety barrier)** | Fragile (crashes / path lost) | Basic | Slow I/O virtualization |
| **Smart Audited Deployment** | **Yes (Checksum ledger + SFTP)** | No (None) | No (None) | No (Requires CI/CD) |
| **Model Context Protocol (MCP)** | **Yes (Native Built-in)** | No | No | No (Requires third-party wrappers) |
| **Administrative Rights (`sudo`)** | **None Required** | Often requires root for 80/443 | Requires helper tool | Requires root/privileged daemon |
| **Cold Startup Speed** | **< 1.5s** | ~5–10s | ~2–3s | ~15–30s |
| **Platform Scope** | macOS first -> Windows | macOS / Windows | macOS first (Windows beta) | macOS / Windows / Linux |
