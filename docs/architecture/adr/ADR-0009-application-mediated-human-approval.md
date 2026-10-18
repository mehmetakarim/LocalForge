# ADR-0009: Application-Mediated Human Approval & AI Agent Authorization Boundary

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Rust Security Auditor, QA Lead
- **Phase**: Recovery Phase R0

---

## 1. Context & Vulnerability Analysis

During the independent technical audit A1, a critical authorization bypass vulnerability was identified in `tools/mcp-server/src/main.rs` and `crates/deployment/src/preview.rs`:
1. `localforge_deploy_preview` returned `"approval_token": preview.preview_id` directly in the MCP JSON-RPC tool response to the AI model.
2. `localforge_deploy_execute` accepted that same `approval_token` as an argument without checking whether a human or an automated agent provided it.
3. Because the AI model was both the requester and the recipient of the token, it could self-authorize production file transfers and remote mutations without any genuine human involvement.

This violated the core architectural invariant defined in ADR-0006: **MCP tool invocations cannot bypass human approval**.

---

## 2. Decision

We establish a strict, tamper-proof **Application-Mediated Human Authorization Architecture** governing all mutating or deployment-triggering operations:

### 2.1 Complete Disconnection of MCP from Production Execution
- The Model Context Protocol (MCP) server is an AI assistant interface, not an administrative backdoor.
- **Preview Only**: MCP tools may request read-only diffs and previews (`localforge_deploy_preview`).
- **No Token Exposure**: MCP responses MUST NEVER include usable approval tokens, authorization nonces, or signature keys.
- **Fail-Closed Execution**: Production execution (`localforge_deploy_execute` with `dry_run: false`) is rejected fail-closed at the MCP barrier. Only dry-run simulations are permitted via MCP.

### 2.2 Application-Mediated Human Approval Protocol (Future Architecture)
When real deployment execution is introduced in subsequent phases:
1. **Cryptographic Binding**:
   An approval ticket must cryptographically bind:
   - `project_id` & `canonical_project_root`
   - `target_id` (e.g. `production`, `staging`)
   - `manifest_sha256` (exact hash of all files, target paths, and checksums to be transferred/deleted)
   - `timestamp` & `expires_at` (short TTL, e.g. 300 seconds)
   - `nonce` (ephemeral random 256-bit entropy)
2. **Out-of-Band Human Interaction**:
   - The approval ticket is submitted directly to the LocalForge native desktop application via internal IPC / socket.
   - The desktop GUI renders an interactive modal showing the diff, destination host, and destructive actions.
   - The human operator must explicitly click "Approve and Deploy" or provide biometric / OS-level authorization.
3. **One-Time Nonce Consumption**:
   - An approval ticket is valid for exactly one execution.
   - Once consumed, the nonce is immediately invalidated in memory.
   - If the project files change after preview generation, the hash mismatch immediately invalidates the ticket.

### 2.3 Interactive CLI Boundary
For command-line deployments (`localforge deploy apply`):
- Authorization requires an interactive TTY (`/dev/tty`) confirmation prompt directly answered by a human operator, preventing piped / scripted execution without deliberate flags.

---

## 3. Security Invariants

- **AI Cannot Self-Authorize**: No AI agent or automated LLM process can generate, retrieve, or sign an approval ticket.
- **State Immutability on Rejection**: A rejected or unverified deployment MUST NEVER update the deployment ledger or modify persistent disk state.
- **Fail-Closed Default**: Any missing component (missing human approval, expired TTL, altered file manifest) results in immediate safe termination.

---

## 4. Consequences

- **Positive**: Eliminates risk of AI hallucinating or autonomously triggering production deletions or file overwrites.
- **Positive**: Hardens compliance with enterprise security requirements and least-privilege principles.
- **Trade-off**: AI agents cannot autonomously complete end-to-end production deployments without prompting the user to approve in the desktop application. This friction is intentional and essential for security.
