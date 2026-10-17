# LocalForge — Product Vision & Strategy

> **"An open-source, AI-ready local development environment that makes PHP project management, external workspace storage, and secure deployment simple."**

---

## 1. Product Purpose & Problem Statement

For over two decades, web developers have relied on local stack managers like XAMPP, MAMP, and WampServer to run PHP, Apache/Nginx, and MySQL on their computers. In recent years, Docker-based alternatives emerged.

However, modern developers face three acute friction points:
1. **The External Storage Breakdown**: Modern laptops (especially Apple Silicon Macs) have fast but limited internal storage. Developers store multiple massive agency projects on high-speed external SSDs. Legacy tools fail catastrophically when external volumes are unmounted, reconnect at new mount paths, or experience transient disconnects.
2. **Resource Exhaustion & Complexity**: Docker Desktop consumes multiple gigabytes of RAM, drains battery life, and suffers from filesystem virtualization overhead when accessing external disks.
3. **The Disconnect from Modern Workflows**: Legacy tools have zero concept of Git-aware deployment, cannot talk to AI coding assistants (Claude Code, OpenAI Codex), lack Model Context Protocol (MCP) support, and do not track what files actually need deployment.

**LocalForge is built from the ground up to solve these challenges.**

---

## 2. Target Users

1. **Freelance & Agency PHP Developers**: Building custom WordPress, Laravel, Symfony, or bespoke PHP applications across dozens of active client projects.
2. **MacBook Power Users Working with External SSDs**: Engineers who keep their client workspaces on portable APFS external drives.
3. **AI-First Developers**: Programmers pairing with AI tools (Claude Code, Antigravity, Cursor, Codex) who want their assistant to safely query runtime diagnostics, inspect local environments, and prepare deployment previews via MCP.
4. **DevOps & Technical Project Managers**: Teams needing audited, Git-aware deployment previews to avoid uploading unvetted files or secrets to production.

---

## 3. Product Principles

1. **Open-Source Core**: LocalForge is committed to a permanently free open-source core under permissive licensing (Apache-2.0 / MIT).
2. **Free Essential Development**: Basic local PHP development, database management, and domain routing will never require a paid license or cloud account.
3. **Zero-Config Developer Experience**: Sane defaults that work out of the box without requiring manual editing of complex configuration files.
4. **AI-Ready Architecture**: Deep, native integration with AI coding assistants via typed Model Context Protocol (MCP) tools.
5. **Project-Aware Runtime Management**: Projects declare their PHP version and environment independently; switching one project's runtime never breaks other active projects.
6. **External Storage First-Class Support**: Native resilience for external SSDs with hardware UUID tracking and graceful disconnect safety barriers.
7. **Secure and Auditable Deployment**: Default-deny secret exclusion, independent deployment ledgers, and mandatory human approval gates before touching remote servers.
8. **Native Desktop Performance**: Powered by Tauri 2 and Rust for minimal memory footprint (<50MB RAM idle) and near-instant startup.
9. **Cross-Platform Extensibility**: Designed with strict platform abstractions to support future operating systems (Windows) cleanly.
10. **User Ownership of Projects and Data**: Your code, your databases, and your configurations remain 100% on your hardware.

---

## 4. Primary Workflows

### Workflow 1: Instant Project Setup on External SSD
1. Developer plugs in external SSD (`/Volumes/DeveloperSSD`).
2. Opens LocalForge Desktop or runs `localforge import /Volumes/DeveloperSSD/Projects/client-store`.
3. LocalForge detects document root (`public/`), suggests PHP 8.2, binds local domain `client-store.test`.
4. Developer works in their preferred editor at native APFS speeds.

### Workflow 2: Safe External Disconnect & Reconnect
1. Developer unplugs external drive without stopping the UI.
2. LocalForge detects volume removal, marks project as `DISCONNECTED`, and prevents any automated tool or process from recreating an empty folder on the internal disk.
3. Developer plugs drive back in; LocalForge matches the volume UUID and automatically restores project availability.

### Workflow 3: AI-Assisted Diagnostics via MCP
1. Developer asks Claude Code: "Why is my client-store project throwing a 502 Bad Gateway?"
2. Claude Code calls `localforge_runtime_diagnostics` via MCP.
3. LocalForge analyzes the PHP-FPM socket, inspects local Nginx error logs, and returns actionable insights without root privileges or secret exposure.

### Workflow 4: Audited Smart Deployment
1. Developer completes work and runs "Prepare Deployment".
2. LocalForge compares the current working tree against the last deployed SHA-256 checksum ledger.
3. Identifies 3 modified files and 1 newly built asset; automatically filters out `.env` and `.git/`.
4. Presents an interactive Deployment Preview. Developer approves with one click.
5. Secure SFTP upload executes, verifies remote SHA-256 hashes, and records the deployment transaction.
