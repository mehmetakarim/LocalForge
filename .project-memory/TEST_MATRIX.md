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
| **TEST-008** | Cargo Workspace Build & Test | `cargo test --workspace` | Rust Cargo 1.94.0 | **PASS** | All unit tests passed across core, platform, storage, project-manager, and desktop lib. |
| **TEST-009** | Cargo Workspace Linter | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Rust Toolchain | **PASS** | Zero clippy errors or warnings across entire workspace. |
| **TEST-010** | Frontend Build & Type Check | `npm run build` (`tsc && vite build`) | Node 20.19.5 | **PASS** | TypeScript strict mode passed; Vite bundle generated successfully in `dist/`. |
| **TEST-011** | Full Tauri Desktop App Check | `cargo check --workspace` (with `localforge-desktop`) | Tauri 2 & Cargo | **PASS** | Tauri 2 application and macOS backend compiled cleanly without errors. |
| **TEST-012** | Local Runtime Engine Tests | `cargo test -p localforge-runtime` | Rust Cargo 1.94.0 | **PASS** | Health probe port find, MariaDB paths, PHP-FPM pool config, and Nginx config tests passed. |
| **TEST-013** | Deployment & Checksum Ledger | `cargo test -p localforge-deployment` | Rust Toolchain | **PASS** | Exclusions (.env, .localforge), SHA-256 computation, and approval token gate tested & passed. |
| **TEST-014** | Headless CLI Operations | `./target/debug/localforge status / volumes / --help` | macOS Apple Silicon | **PASS** | Evaluated loopback ports, detected external APFS disk with UUID, showed clean diagnostics. |
| **TEST-015** | MCP Server JSON-RPC Stdio | Echo pipe `tools/list`, `tools/call` into `localforge-mcp` | JSON-RPC 2.0 Stdio | **PASS** | Successfully listed 5 tools, ran diagnostics, diff preview, and approval gate token execution. |
| **TEST-016** | Security Traversal Hardening | Malicious relative vectors (`../`, `/private/etc`, nested tricks) | `crates/storage` suite | **PASS** | All 5 security tests passed; path normalization strictly rejects escapes. |
| **TEST-017** | Phase 4 Workspace Audit | `cargo test --workspace` & `cargo clippy` | Rust Toolchain 1.94.0 | **PASS** | 13/13 tests passed across 7 crates with 0 clippy warnings. |

---

## Phase R1.3B Final Acceptance Remediation Matrix

| Test ID | Test Category | Description / Command | Environment | Status | Verification Evidence / Notes |
|---|---|---|---|---|---|
| **TEST-R1.3B-REM-01** | Visual Verification | Playwright Chrome capture of 9 UI surfaces (0, 2, 6 projects, inspector, workspaces, responsive 1024x768) | Vite v6.4.4 & Chrome Headless | **PASS** | 9 verified PNG screenshots captured in `docs/design/r1_3b/implementation_verified/` and packaged in ZIP. 2-proj & inspector render valid projects and drawer. |
| **TEST-R1.3B-REM-02** | Browser Routing Isolation | `check_domain_resolution` in Tauri & Nginx `default_server` | Rust Cargo & ToSocketAddrs | **PASS** | Raw loopback IP returns 404; unresolvable `.test` domain opens `RoutingAlertModal` with cURL fallback. Verified via `test_r1_3b_stage_b_nginx_default_server_blocks_cross_contamination_and_requires_host_header`. |
| **TEST-R1.3B-REM-03** | Storage Capacity Integrity | `StorageVolumes` capacity calculation & zero-fallback removal | Node.js Test Runner | **PASS** | `r1_3b_remediation.test.mjs` verifies `usagePercent` is null (not 45%) for zero/negative/disconnected/overflow bytes. |
| **TEST-R1.3B-REM-04** | Services Telemetry State | UNKNOWN preservation for unprobed Nginx, PHP-FPM, MariaDB | Node.js Test Runner | **PASS** | `r1_3b_remediation.test.mjs` verifies `runtime == null` or `undefined` properties render `BİLİNMİYOR` badges without collapsing into `false`. |
| **TEST-R1.3B-REM-05** | Real Import Workflow | `ImportProjectModal` folder path import | React 19 & Tauri | **PASS** | "Mevcut Projeyi Aç" opens dedicated modal invoking `import_project` rather than project creation. |
| **TEST-R1.3B-REM-06** | Workspace Rust Test Suite | `cargo test --workspace` | Apple Silicon Mac M1 | **PASS** | 58/58 tests passed (%100 PASS) across all 7 workspace crates. |
| **TEST-R1.3B-REM-07** | Desktop Frontend Test Suite | `npm test --prefix apps/desktop` | Node.js Test Runner | **PASS** | 19/19 tests passed (%100 PASS) including fixture safety, health classification, and R1.3B remediation suites. |
| **TEST-R1.3B-REM-08** | Linter & Clippy Audit | `cargo clippy --workspace --all-targets -- -D warnings` | Rust Toolchain 1.94.0 | **PASS** | Zero warnings/errors across entire workspace. |
| **TEST-R1.3B-REM-09** | Formatter Audit | `cargo fmt --check` | Rust Toolchain 1.94.0 | **PASS** | Zero diffs across entire codebase. |
| **TEST-R1.3B-REM-10** | Production Bundle & Typecheck | `npm --prefix apps/desktop run build` | TypeScript 5.7 & Vite | **PASS** | Clean compilation in 8.77s; zero type errors. |

---

## Phase R1.3B Final Acceptance Closeout Matrix

| Test ID | Test Category | Description / Command | Environment | Status | Verification Evidence / Notes |
|---|---|---|---|---|---|
| **TEST-R1.3B-CLOSE-01** | Import Error Workflow | Failed import preserves entered path, displays error, and never closes modal | Node.js Test Runner | **PASS** | `r1_3b_remediation.test.mjs` verifies state preservation and error propagation. |
| **TEST-R1.3B-CLOSE-02** | Production Utility Tests | Direct imports of `storageCapacity.ts`, `servicesTelemetry.ts`, `browserRouting.ts` | Node.js / Jiti Test Runner | **PASS** | 20/20 desktop tests pass directly testing production logic. |
| **TEST-R1.3B-CLOSE-03** | Domain Routing Isolation | Live multi-domain isolation & default_server 404 block (`test_r1_3b_live_domain_routing_acceptance_and_isolation`) | macOS Darwin / Nginx 1.31.7 | **PASS** | Unknown domain -> 404; Alpha -> Alpha only; Beta -> Beta only; zero cross-project leakage. |
| **TEST-R1.3B-CLOSE-04** | Genuine Native Tauri Capture | 7 high-resolution screenshots captured from live Cocoa app (`target/debug/localforge-desktop`) backed by SQLite (`localforge.db`) | macOS Apple Silicon / Tauri 2 | **PASS** | 7 verified screenshots (1600×1000) stored in `docs/design/r1_3b/native_tauri_verified/` and packaged into `localforge_r1_3b_native_tauri_screenshots.zip`. OCR verified live data. |
| **TEST-R1.3B-CLOSE-05** | Workspace Rust Test Suite | `cargo test --workspace` | Apple Silicon Mac M1 | **PASS** | 58/58 tests passed (%100 PASS) across all 7 workspace crates. |
| **TEST-R1.3B-CLOSE-06** | Desktop Frontend Test Suite | `npm test --prefix apps/desktop` | Node.js Test Runner | **PASS** | 20/20 tests passed (%100 PASS) in 481ms. |
| **TEST-R1.3B-CLOSE-07** | Linter & Clippy Audit | `cargo clippy --workspace --all-targets -- -D warnings` | Rust Toolchain 1.94.0 | **PASS** | Zero warnings/errors across entire workspace. |
| **TEST-R1.3B-CLOSE-08** | Formatter Audit | `cargo fmt --all -- --check` | Rust Toolchain 1.94.0 | **PASS** | Zero diffs across entire codebase. |
| **TEST-R1.3B-CLOSE-09** | Production Bundle & Typecheck | `npm --prefix apps/desktop run build` | TypeScript 5.7 & Vite 6.4.4 | **PASS** | Clean build in 6.67s; zero type errors. |

---

## Verification Status Definitions
- **PASS**: Executed with exit code 0 and actual output matched expectations.
- **FAIL**: Executed and failed to meet expectations.
- **BLOCKED**: Could not be executed due to missing pre-requisite.
- **NOT RUN**: Intentionally scheduled for future phases or skipped by design.
- **MANUAL VERIFICATION REQUIRED**: Interactive human or physical intervention required.
