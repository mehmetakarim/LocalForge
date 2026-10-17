# LocalForge — Security Policy & Vulnerability Disclosure

LocalForge takes the security of local development environments, developer workstations, and deployment pipelines seriously. This document defines our security policies, vulnerability reporting procedure, and disclosure commitments.

---

## 1. Supported Versions

Only the latest active release and current development branch receive security fixes:

| Version / Branch | Supported |
|---|---|
| `main` (Development) | :white_check_mark: Yes |
| `phase-0/foundation` | :white_check_mark: Yes (Current active phase) |
| Older releases | :x: No (Update to latest) |

---

## 2. Reporting a Vulnerability

If you discover a security vulnerability in LocalForge:
1. **DO NOT file a public GitHub issue.**
2. Send an email to the security team / maintainer at:
   `mehmetakarim@gmail.com`
3. Include the following details:
   - Type of vulnerability (e.g. path traversal, secret exposure, MCP privilege escalation).
   - Component affected (`crates/storage`, `crates/deployment`, etc.).
   - Step-by-step reproduction instructions or a minimal proof of concept.
   - Potential impact on the host system or remote servers.

---

## 3. Vulnerability Response Timeline

- **Initial Acknowledgment**: Within 48 hours of report receipt.
- **Triage & Assessment**: Within 5 business days.
- **Fix Development & Patch Release**: Critical vulnerabilities patched within 14 business days.
- **Coordinated Public Disclosure**: After a security patch has been released and users have had adequate time to update.

---

## 4. Security Invariants for Contributors

All code contributed to LocalForge must uphold these invariants:
1. **No Root Requirement**: Never require the user to run `sudo` or grant root permissions.
2. **Path Sanitization**: Validate and canonicalize all filesystem paths against project boundaries before any file I/O.
3. **No Secrets in Logs or Manifests**: Never output passwords, tokens, or private keys to stdout, log files, or `.localforge/project.yaml`.
4. **Human Confirmation for External Mutations**: Never trigger remote SFTP uploads or permanent local deletions without explicit user approval.
