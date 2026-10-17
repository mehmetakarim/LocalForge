# ADR-0008: Project Memory System and Sequential Phase Governance

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Lead Agent, Project Manager
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

AI-assisted software development across multiple platforms (Claude Code, OpenAI Codex, Antigravity) and human contributors suffers from context amnesia between sessions. Common failure modes include:
- Prematurely starting future phases without acceptance of prior phases.
- Fabricating test results ("tests passed" when none were run).
- Parallel worker agents colliding and overwriting project state files.
- Fragmented or conflicting instructions across tool-specific configuration files.

---

## 2. Decision

We establish a **Strict, Git-Versioned Project Memory and Phase Governance Architecture**:

### 2.1 File-Based Canonical Memory System
The `.project-memory/` directory is the single source of truth for cross-session continuity:
- `PROJECT_STATE.md`: Active phase, verified checkpoints, blockers.
- `DECISIONS.md`: ADR index and approval states.
- `ROADMAP.md`: Multi-phase objectives and completion criteria.
- `TASKS.md`: Granular backlog with stable IDs and dependencies.
- `SESSION_LOG.md`: Chronological session activity logs.
- `KNOWN_ISSUES.md`: Identified bugs and system constraints.
- `TEST_MATRIX.md`: Honest verification outcomes.
- `HANDOFF.md`: Authoritative concise handoff for subsequent sessions.

### 2.2 Role Distribution & Write Isolation
- **Lead Agent / Auditor Agent**: Sole authority to modify memory documents (`brain.md`, `docs/tasks.md`, `.project-memory/*`).
- **Worker Agents**: Authorized only to edit source code files within their specific task scope; report results upward to the Lead Agent.

### 2.3 Sequential Phase Governance
- Work is strictly locked to the currently authorized phase.
- Moving from Phase $N$ to Phase $N+1$ strictly requires explicit Product Owner acceptance.
- Future phases must remain planned specifications only.

### 2.4 Honest Verification Policy
- Every test or audit outcome must be categorized strictly as:
  - `PASS` (executed, zero exit code, verified output)
  - `FAIL` (executed, error logged)
  - `BLOCKED` (prerequisite missing)
  - `NOT RUN` (scheduled for later phase or intentionally skipped)
  - `MANUAL VERIFICATION REQUIRED` (interactive or physical human check required)
- Code inspection must never be reported as an executed test.

---

## 3. Alternatives Considered

### Alternative A: Relying on AI Session Chat History
- *Rejected*: Context windows are ephemeral and truncate; chat transcripts are not version-controlled in the repository.

### Alternative B: External Issue Tracker Only (Jira, GitHub Issues)
- *Rejected*: AI coding tools require local, direct filesystem access to context without API keys or network friction.

---

## 4. Consequences & Risks

### Positive
- Perfect engineering continuity across different AI tools and human developers.
- Zero race conditions in multi-agent workflows.
- Absolute transparency into verified vs unverified functionality.

### Negative / Risks
- Requires discipline to update handoff and state files at session boundaries.

---

## 5. Verification Status

- Successfully initialized and enforced in Phase 0.
