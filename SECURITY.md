# Security Policy & Vulnerability Disclosure

## Supported Versions

LocalForge takes local environment isolation, non-root execution, credential confidentiality, and deployment safety seriously. Security patches are applied to the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

---

## Non-Negotiable Security Principles

1. **Least Privilege**: LocalForge runtimes (PHP-FPM, Nginx, MariaDB) never run as `root` and default strictly to local loopback (`127.0.0.1`).
2. **Path Traversal Protection**: Project and file operations are strictly bounded to verified project roots. Canonicalized paths are verified before file reads, writes, or execution.
3. **External Storage Resilience**: When an external SSD or workspace is disconnected, LocalForge safe barriers prevent directory recreation on internal disks or data corruption.
4. **Default-Deny Deployment**: The smart deployment engine automatically excludes `.env*`, `.git/`, SSH keys, private certificates, and local runtime sockets.
5. **Human Approval Gates**: Mutating deployments strictly require a valid, ephemeral human approval token (`prev_<uuid>`). AI coding agents (Claude Code, Codex) interacting via the MCP server cannot trigger live remote deployments without explicit human confirmation.

---

## Reporting a Vulnerability

If you discover a security vulnerability within LocalForge:

1. **Do not disclose the vulnerability publicly** via GitHub Issues or public forums.
2. Email the maintainer directly at: `mehmetakarim@gmail.com` with the subject `[LocalForge Security] <Vulnerability Title>`.
3. Please include:
   - Detailed description and attack vector.
   - Proof-of-concept steps or reproducible script.
   - Affected components (`crates/storage`, `crates/deployment`, `tools/mcp-server`, etc.).
   - Estimated impact.

Maintainers will acknowledge receipt within 48 hours and coordinate a coordinated disclosure timeline.
