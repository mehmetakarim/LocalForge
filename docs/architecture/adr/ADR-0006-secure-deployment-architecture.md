# ADR-0006: Secure Deployment Architecture & Git-Aware Ledger

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Security Engineer, Senior Rust Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

Developers frequently need to deploy changed files to remote staging and production servers (typically via SFTP). Existing tools often rely simply on raw `git diff` outputs or upload entire project trees.
Crucial vulnerabilities and failure modes include:
1. **Accidental Secret Disclosure**: Uploading `.env`, `.git/`, SSH keys, or private database dumps.
2. **Untracked File Discrepancies**: A newly generated production build asset or uploaded media file may not be tracked in Git, yet needs deployment. Conversely, local scratch files should not be uploaded.
3. **Unreliable Deployment History**: A `git diff` shows commit differences, not what was actually transferred and verified on the remote server. Network drops during transfers leave partial deployments in an unknown state.
4. **AI Autonomy Danger**: Allowing AI agents (via MCP or automated tools) to upload or delete remote production files autonomously without human oversight is a critical operational hazard.

---

## 2. Decision

We establish an **Audited Deployment Ledger** and strict **Security Transfer Policies**:

### 2.1 Independent Deployment Ledger
- Deployment state is stored in an independent local ledger (`.localforge/deployments/<target-id>.json`), tracking:
  - Last successfully deployed commit SHA.
  - Per-file SHA-256 checksums at the moment of deployment.
  - Remote file path mappings and timestamps.
- Change detection evaluates the current working tree against the deployment ledger, categorizing files as: `Created`, `Modified`, `Deleted`, `Unchanged`, or `Ignored`.

### 2.2 Default-Deny File Exclusion Engine
- Built-in, non-overridable exclusion rules:
  - Environment files: `.env`, `.env.*` (except `.env.example`).
  - Source control: `.git/**`, `.gitignore`.
  - Credentials & Secrets: `*.pem`, `*.key`, `*.pfx`, `id_*`, `known_hosts`.
  - LocalForge Internal Metadata: `.localforge/secrets/**`, `.localforge/run/**`.
  - OS & Editor metadata: `.DS_Store`, `.idea/**`, `.vscode/**`.

### 2.3 Deployment Preview & Mandatory Human Approval
- Prior to initiating any network transfer, LocalForge constructs a **Deployment Preview Manifest**.
- The preview details every file to be uploaded, byte counts, and target paths.
- **Human Gate**: Execution requires explicit human confirmation via the Desktop UI or interactive CLI confirmation.
- **MCP Invariant**: An MCP tool invocation (`localforge_deploy_execute`) **CANNOT** bypass this human approval requirement.

### 2.4 Transport Protocol
- **SFTP (SSH File Transfer Protocol)** is the primary, required deployment protocol for Phase 3.
- Key and password credentials must be retrieved from the macOS Keychain.

---

## 3. Alternatives Considered

### Alternative A: Raw `git archive` Upload
- *Rejected*: Does not handle untracked build artifacts or granular remote synchronization.

### Alternative B: Plain FTP
- *Rejected*: Unencrypted protocol exposing credentials over public networks. Only considered as an explicitly warned, opt-in legacy compatibility flag.

---

## 4. Consequences & Risks

### Positive
- Complete auditable ledger of remote server deployments.
- Leakage of secrets is eliminated by design.
- Safe recovery from interrupted transfers via SHA-256 verification.

### Negative / Risks
- Remote server file modifications made outside LocalForge (drift) cannot be fully detected without a remote agent.

---

## 5. Verification Status

- Security boundaries reviewed in `docs/security/THREAT_MODEL.md`.
- Implementation scheduled for Phase 3 (TASK-301 & TASK-302).
