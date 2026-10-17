# AGENTS.md — LocalForge Engineering Instructions & Collaboration Protocol

Welcome to the **LocalForge** engineering repository. This document defines the non-negotiable architectural boundaries, development discipline, multi-agent collaboration protocols, security baseline, and session lifecycle rules for all AI coding agents (Claude Code, OpenAI Codex, Antigravity, and other compatible systems) and human engineers.

---

## 1. Project Overview & Boundaries

- **Repository**: `mehmetakarim/LocalForge`
- **Product**: LocalForge — Open-source, AI-ready local development and deployment environment.
- **Current Authorized Phase**: **Phase 0 — Foundation & Project Memory**
- **Strict Phase Discipline**: Work MUST strictly remain within the currently authorized phase. NEVER start implementation of Phase 1 or subsequent phases without explicit human approval.

---

## 2. Teamwork & Multi-Agent Collaboration Protocol

To maintain architectural integrity and prevent race conditions or merge conflicts across multi-agent environments:

### 2.1 Role Distribution
1. **Lead Architect / Lead Agent**: Responsible for task assignment, architectural oversight, cross-cutting consistency, final validation, and centralized documentation maintenance.
2. **Worker Agents (Specialist Agents)**: Work strictly on assigned code or specific research tasks within their authorized boundary.
3. **Auditor Agent / Verification Agent**: Audits implementations against ADRs, runs verification suites, and produces honest assessment reports.

### 2.2 Write Permission Restrictions (Conflict Prevention)
- **RESTRICTED FILES**:
  - `brain.md`
  - `docs/tasks.md`
  - `.project-memory/PROJECT_STATE.md`
  - `.project-memory/TASKS.md`
  - `.project-memory/HANDOFF.md`
  - `.project-memory/DECISIONS.md`
- **RULE**: Worker Agents **MUST NEVER** directly modify these restricted memory and tracking files.
- **CENTRALIZED COMMITS**: Only the **Lead Agent** or **Auditor Agent** is authorized to update memory files, task statuses, and roadmap states after tasks have been verified.
- **WORKER REPORTING**: Worker agents must report their findings, diffs, test logs, and root-cause analyses in their designated work output or temporary scratch reports.

---

## 3. Mandatory Session Protocols

### 3.1 Startup Protocol (Run Before Modifying Files)
Every development session must begin with:
1. Identify the repository root (`pwd`).
2. Inspect Git working-tree status (`git status`, `git branch`, uncommitted changes).
3. Read `AGENTS.md` (this file).
4. Read `CLAUDE.md` if running via Claude Code.
5. Read `.project-memory/HANDOFF.md` for current handoff status.
6. Read `.project-memory/PROJECT_STATE.md` to identify the active phase and blockers.
7. Read `.project-memory/ROADMAP.md` and `.project-memory/TASKS.md`.
8. Review relevant Architecture Decision Records in `docs/architecture/adr/`.
9. Confirm currently authorized phase.
10. Report planned scope to the user before editing code.

### 3.2 Closure Protocol (Run Before Concluding Sessions)
Every meaningful development session must conclude with:
1. Review all modified files against authorized scope.
2. Run relevant automated checks and linters.
3. Document tests honestly (PASS, FAIL, BLOCKED, NOT RUN, MANUAL VERIFICATION REQUIRED).
4. Update task backlog (`.project-memory/TASKS.md` and `docs/tasks.md`) via Lead Agent.
5. Update `.project-memory/PROJECT_STATE.md` and `.project-memory/SESSION_LOG.md`.
6. Update `.project-memory/KNOWN_ISSUES.md` and `.project-memory/TEST_MATRIX.md`.
7. Refresh `.project-memory/HANDOFF.md` with concise next steps.
8. Report Git working-tree status and uncommitted changes.
9. Deliver an engineering summary in Turkish (as required by user preferences).

---

## 4. Architectural Non-Negotiables

### 4.1 Shared Core Architecture
- The domain and service logic resides in a shared Rust core crate workspace (`crates/core`, `crates/runtime`, `crates/storage`, `crates/project-manager`, `crates/deployment`, `crates/platform`).
- The Desktop UI (Tauri 2), CLI (`tools/cli`), and MCP Server (`tools/mcp-server`) MUST consume the same core crates.
- UI or CLI layers must never become the source of truth for project state or implement disjoint business rules.

### 4.2 Platform Isolation & macOS First
- LocalForge targets macOS Apple Silicon (`arm64`) as tier-1.
- Operating system specifics (process management, Keychain, mount points, Launchd) must be abstracted behind platform traits (`crates/platform`).
- Do not hardcode Linux or Windows specifics directly into core domain logic.

### 4.3 External Workspace Support
- External storage (e.g., `/Volumes/...`) is a first-class citizen.
- Never assume external disks have identical mount paths across reboots. Use persistent volume identification (UUID / Volume serial).
- Detect disconnection gracefully. If a volume is disconnected, fail operations safely without recreating directories on the internal disk.
- Never automatically store massive runtime binaries or databases on the internal disk without user visibility.

### 4.4 Least Privilege & Safety
- **No Root Execution**: Never require or execute background services as `root`. Development servers (Nginx, PHP-FPM, MariaDB) must run under the current user with unprivileged ports (>1024) or local loopback proxying.
- **Path Traversal Protection**: All filesystem operations must be bounded by project roots and canonicalized. Untrusted symlinks pointing outside the project root must be rejected.
- **Never Auto-Delete User Projects**: Destructive actions require explicit user confirmation.
- **Atomic Operations**: Configuration files and deployment state files must be updated atomically via temporary files and renames.

### 4.5 Security & Deployment
- Git diff is NOT a deployment ledger. Track deployment artifacts with independent checksums and manifests.
- Default-deny sensitive files: `.env*`, `.git/`, SSH keys, private certificates, build caches.
- Credentials must reside in the OS Keychain, never in plain-text manifests or Git.
- Model Context Protocol (MCP) tools must be read-safe and enforce explicit human approvals for mutating or deployment-triggering actions.

---

## 5. Honest Verification Policy

Do not fabricate or assume test results:
- **PASS**: Executed with exit code 0 and validated expected output.
- **FAIL**: Executed and failed with diagnostic output recorded.
- **BLOCKED**: Cannot run due to missing dependencies or prerequisite tasks.
- **NOT RUN**: Skipped intentionally or pending later execution.
- **MANUAL VERIFICATION REQUIRED**: Requires interactive human check (e.g., UI inspection, system prompt).

Clearly distinguish static code inspection from active runtime verification.
