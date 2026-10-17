# LocalForge — Non-Goals & Scope Boundaries

To ensure engineering focus, architectural integrity, and rapid delivery of core value, this document explicitly outlines what LocalForge is **NOT** designed to do during initial releases.

---

## 1. Explicit Non-Goals for Early Phases (Phase 0 – Phase 3)

### 1.1 Not a General-Purpose Container Orchestrator
- **Out of Scope**: LocalForge will **NOT** attempt to replicate Kubernetes, Docker Compose, or Podman.
- **Rationale**: LocalForge is designed for lightweight, high-performance, native local PHP development directly on macOS host hardware. Full container virtualization introduces heavy RAM overhead and slow disk I/O on external SSDs.

### 1.2 Not a Cloud Hosting / CI/CD Platform
- **Out of Scope**: LocalForge will **NOT** host servers, run public cloud infrastructure, manage Kubernetes clusters, or become a SaaS deployment pipeline.
- **Rationale**: LocalForge empowers the developer's local machine to safely push verified artifacts to their own remote infrastructure (via SFTP).

### 1.3 Not an Unrestricted Autonomous AI Agent
- **Out of Scope**: LocalForge will **NOT** allow AI agents (via MCP or automated tools) to execute production server deployments or perform destructive filesystem deletions without interactive human approval.
- **Rationale**: Safety and auditability are non-negotiable. Human-in-the-loop governance protects production systems from hallucinated commands or prompt injection attacks.

### 1.4 No Unencrypted Legacy Protocols by Default
- **Out of Scope**: Plain FTP and unencrypted Telnet are excluded from default deployment flows.
- **Rationale**: Transmitting credentials over unencrypted channels violates modern security standards. Only secure protocols (SFTP) are supported initially.

### 1.5 No Global System Configuration Hijacking
- **Out of Scope**: LocalForge will **NOT** silently replace system-level `/etc/hosts`, overwrite system DNS daemons, alter system-wide Homebrew formulae, or demand root administrative privileges.
- **Rationale**: Preserving host system stability and security integrity is paramount.

### 1.6 No Multi-Language Monolith in Initial Releases
- **Out of Scope**: Supporting Python Django, Ruby on Rails, Node.js SSR, and Java Spring runtimes is excluded from early releases.
- **Rationale**: Focusing strictly on PHP (WordPress, Laravel, Symfony, custom stacks) ensures a polished, reliable developer experience before expanding into other runtimes.
