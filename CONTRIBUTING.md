# LocalForge Contributing Guidelines & Engineering Standards

Welcome! We appreciate contributions to LocalForge. To maintain high engineering rigor, security, and multi-session continuity, all contributors and AI agents must follow these standards.

---

## 1. Core Principles

1. **Correctness & Safety Over Speed**: Never rush fragile code into the repository.
2. **Honest Verification**: Only mark tests as PASS if they were actually executed with exit code 0.
3. **Sequential Phase Control**: Never implement code beyond the currently authorized phase.
4. **Least Privilege**: Avoid executing commands as `root` or disabling platform security features (SIP, Gatekeeper).
5. **No Secret Leaks**: Never commit `.env`, API keys, private certificates, or user credentials.

---

## 2. Git & Branching Strategy

- **Default Branch**: `main`
- **Branch Naming**:
  - Phase work: `phase-<number>/<topic>` (e.g., `phase-0/foundation`)
  - Features: `feat/<feature-name>`
  - Bug fixes: `fix/<issue-name>`
  - Documentation: `docs/<topic>`
- **Commit Conventions**:
  All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) format:
  - `docs: update architecture overview and ADR-0004`
  - `chore: add gitignore rules for tauri build artifacts`
  - `feat(core): implement volume uuid resolver for macOS`
  - `fix(storage): prevent directory recreation on disconnected external disk`
  - `refactor(runtime): decouple nginx config generation`
  - `test(project-manager): add unit tests for project.yaml parser`
  - `ci: configure cargo audit and markdown lint check`
- **Pull Request Rules**:
  - Never push directly to `main`.
  - Feature branches must pass all local verification checks before opening a PR.

---

## 3. Code Standards & Toolchains

### 3.1 Rust Standards
- **Compiler**: Rust stable (Edition 2021).
- **Formatting**: `cargo fmt --check` must pass.
- **Linting**: `cargo clippy --all-targets --all-features -- -D warnings` must pass with zero warnings.
- **Error Handling**: Use `thiserror` for library crates and `anyhow` for binaries/tools where appropriate. Never use `unwrap()` or `expect()` in production code paths unless accompanied by an explicit invariant proof comment.
- **Process Spawning**: Always use structured `std::process::Command` without shell string interpolation (`sh -c`) to prevent command injection vulnerabilities.

### 3.2 Frontend & TypeScript Standards
- **Runtime & Manager**: Node.js v20+ / npm / bun.
- **Linting & Formatting**: ESLint + Prettier.
- **Type Safety**: TypeScript strict mode enabled (`noImplicitAny`, `strictNullChecks`). `tsc --noEmit` must pass with zero errors.
- **Components**: Functional React components with accessible HTML semantics and Tailwind CSS styling.

---

## 4. Definition of Done (DoD)

A task or pull request is technically complete only when:
1. **Scope Alignment**: Matches the task description in `.project-memory/TASKS.md` within the authorized phase.
2. **Automated Verification**:
   - Rust formatting and clippy pass without warnings (when code is present).
   - TypeScript checks pass without errors (when code is present).
   - Relevant unit and integration tests pass.
3. **Documentation Updated**:
   - Relevant ADR written or updated if architectural decisions were made.
   - Project memory files updated by the Lead/Auditor Agent.
   - Code documented with doc-comments (`///`) explaining rationale.
4. **Security Reviewed**:
   - Path bounds validated.
   - No hardcoded secrets or permissive file permissions.
5. **No Regressions**: Existing test suites continue to pass.

---

## 5. Multi-Agent Memory Protocol

When contributing using AI coding assistants (Claude Code, OpenAI Codex, Antigravity):
- Respect the **Write Authority Restriction**: Worker agents edit code only; Lead/Auditor Agent updates `brain.md`, `docs/tasks.md`, and `.project-memory/`.
- Review [`AGENTS.md`](AGENTS.md) at the beginning of each session.
