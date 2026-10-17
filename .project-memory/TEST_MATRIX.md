# LocalForge — Test Matrix & Verification Evidence

This document records all test scenarios, executed checks, environments, and verification statuses.

> [!NOTE]
> All results follow the strict honest verification policy: **PASS**, **FAIL**, **BLOCKED**, **NOT RUN**, or **MANUAL VERIFICATION REQUIRED**. Code inspection is never reported as an executed test.

---

## Phase 0 Verification Matrix

| Test ID | Test Category | Description / Command | Environment | Status | Verification Evidence / Notes |
|---|---|---|---|---|---|
| **TEST-001** | Environment Audit | macOS & Darwin kernel version check (`sw_vers`, `uname -a`) | macOS 27.0 (Apple Silicon M1) | **PASS** | macOS 27.0, Kernel Version 27.0.0, arm64 Darwin Kernel. |
| **TEST-002** | Toolchain Audit | Rust and Cargo availability check (`rustc --version`, `cargo --version`) | Host Toolchain | **PASS** | `rustc 1.94.0`, `cargo 1.94.0`, `rustup 1.29.0` verified. |
| **TEST-003** | Toolchain Audit | Node.js and Bun runtime check (`node -v`, `npm -v`, `bun -v`) | Host Toolchain | **PASS** | Node `v20.19.5`, npm `11.7.0`, Bun `1.3.12` verified. |
| **TEST-004** | Toolchain Audit | Xcode CLI Tools and Apple Clang check (`xcode-select -p`, `clang -v`) | Host Developer | **PASS** | Apple clang `21.0.0` at `/Applications/Xcode.app/Contents/Developer`. |
| **TEST-005** | Storage Audit | Active working directory filesystem and mount inspection (`df -h`, `mount`) | External APFS SSD | **PASS** | Volume `/Volumes/Mac Harici Disk` confirmed APFS, journaled, 135 GiB free. |
| **TEST-006** | Git Setup | Git initialization and branch configuration (`git status`, `git branch`) | Git CLI 2.56.0 | **PASS** | Repo initialized, `main` set, `phase-0/foundation` branch active. |
| **TEST-007** | Document Integrity | Markdown link and cross-reference check script execution | Repository root | **PASS** | All relative file links verified existing on disk. |
| **TEST-008** | Cargo Workspace Build | `cargo check --workspace` | Rust Cargo | **NOT RUN** | Expected: No source code permitted in Phase 0. Scheduled for Phase 1. |
| **TEST-009** | Desktop App Launch | `npm run tauri dev` | Tauri Desktop | **NOT RUN** | Scheduled for Phase 1 (Desktop Foundation). |
| **TEST-010** | External Disconnect Test | Physical drive unmount simulation | Systems Layer | **NOT RUN** | Scheduled for Phase 1 (TASK-103). |

---

## Verification Status Definitions
- **PASS**: Executed with exit code 0 and actual output matched expectations.
- **FAIL**: Executed and failed to meet expectations.
- **BLOCKED**: Could not be executed due to missing pre-requisite.
- **NOT RUN**: Intentionally scheduled for future phases or skipped by design.
- **MANUAL VERIFICATION REQUIRED**: Interactive human or physical intervention required.
