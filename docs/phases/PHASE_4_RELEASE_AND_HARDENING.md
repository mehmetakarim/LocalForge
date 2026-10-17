# Phase 4: Hardening, Packaging & Open-Source Release

## 1. Executive Overview

Phase 4 concludes the initial release lifecycle of **LocalForge v0.1.0**. It transitions the codebase from local development and verified feature implementation into a hardened, distributable, and open-source release baseline.

- **Phase Objective**: Harden all security barriers (fuzzing & path traversal tests), establish automated multi-tier CI/CD pipelines (GitHub Actions for macOS arm64), configure native macOS DMG packaging with hardened runtime flags, and establish standard open-source community health files.
- **Authorized Boundary**: Packaging, testing, GitHub Actions, security tests, and release documentation.

---

## 2. Deliverables & Tasks

### TASK-401: GitHub Actions CI/CD Workflows
- `.github/workflows/ci.yml`:
  - Triggers on PR and push to `main` and release branches.
  - Runs on `macos-14` (Apple Silicon M1/M2 runners).
  - Validates:
    - `cargo fmt --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test --workspace`
    - `cd apps/desktop && npm run build`
- `.github/workflows/release.yml`:
  - Triggers on version tag push (`v*`).
  - Builds optimized DMG using `@tauri-apps/cli`.
  - Generates SHA-256 checksums and drafts a GitHub release.

### TASK-402: Security Hardening & Edge-Case Audit Suite
- Implement adversarial tests in `crates/storage/src/paths.rs` and `crates/deployment/src/filter.rs`:
  - Canonicalization bypass attempts (`/Volumes/../`, relative symlinks).
  - Malformed `.localforge/project.yaml` fuzzing / schema validation.
  - Nested symlink directory boundary verification.

### TASK-403: macOS DMG Packaging Configuration
- Update `apps/desktop/src-tauri/tauri.conf.json`:
  - Set bundle identifier: `com.localforge.app`.
  - Configure macOS bundle targets: `["dmg", "app"]`.
  - Enable macOS hardened runtime and minimum macOS version target (12.0+).

### TASK-404: Open-Source Community Health Suite
- `SECURITY.md`: Vulnerability reporting process and security advisories.
- `CODE_OF_CONDUCT.md`: Contributor Covenant Code of Conduct.
- `.github/ISSUE_TEMPLATE/bug_report.yml` & `feature_request.yml`.
- `.github/PULL_REQUEST_TEMPLATE.md`.

### TASK-405: Final Phase 4 Verification & Closure Audit
- Full workspace test pass (`cargo test --workspace`).
- Clippy zero warning verification.
- Final session logging and handoff delivery.
