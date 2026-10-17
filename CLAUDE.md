# CLAUDE.md — Claude Code Quick Reference

This file provides a concise entry point for Claude Code sessions. The canonical project rules, memory, and architecture are defined in `AGENTS.md` and `.project-memory/`.

---

## 1. Quick Orientation

- **Project**: LocalForge (Open-source AI-ready local development & deployment environment)
- **Primary Platform**: macOS Apple Silicon (`arm64`)
- **Authorized Phase**: **Phase 0 — Foundation & Project Memory** (Do NOT start Phase 1!)
- **Repository**: `mehmetakarim/LocalForge`

---

## 2. Essential Commands

```bash
# Inspect current Git branch & status
git status
git branch

# Verify environment audit
sw_vers
rustc --version
cargo --version
node --version

# Run Phase 0 markdown/link validation checks
# (Run scripts in scratch or tools once created)
```

---

## 3. Mandatory Memory Links

Before taking any action, review the following canonical files:
1. `AGENTS.md` — Core engineering principles, multi-agent protocol, and phase discipline.
2. `.project-memory/HANDOFF.md` — Current state and immediate next permitted tasks.
3. `.project-memory/PROJECT_STATE.md` — Active phase details and verified checkpoints.
4. `.project-memory/TASKS.md` — Stable task identifiers and backlog.
5. `docs/architecture/adr/` — Accepted architectural decision records.

---

## 4. Work Rules for Claude Code

- **Phase Discipline**: Only execute tasks defined under Phase 0. No application code or crate implementation in Phase 0.
- **Memory Integrity**: Only the Lead/Auditor Agent may update `.project-memory/` and `docs/tasks.md`.
- **Honest Verification**: Classify test outcomes strictly as PASS, FAIL, BLOCKED, NOT RUN, or MANUAL VERIFICATION REQUIRED.
- **Reporting**: Always communicate progress and final reports to the user in **Turkish**.
