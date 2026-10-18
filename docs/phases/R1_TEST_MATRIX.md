# LocalForge — Recovery Phase R1 Test Matrix & Verification Protocols

**Document Version**: 1.0.0  
**Phase**: Recovery Phase R1  
**Status**: SPECIFIED (Pending R1 Sprint Execution)  

---

## 1. Test Verification Standards

All tests in Phase R1 must strictly follow the Honest Verification Policy (AGENTS.md Section 5):
- **PASS**: Executed with exit code 0 and validated expected output.
- **FAIL**: Executed and failed with diagnostic output recorded.
- **BLOCKED**: Prerequisite dependency missing.
- **NOT RUN**: Planned for later execution in the sprint.
- **MANUAL VERIFICATION REQUIRED**: Interactive human or physical hardware test.

---

## 2. Test Execution Matrix

| Test ID | Test Category | Target Component | Description & Expected Result | Execution Mode | Target Status |
|---|---|---|---|---|---|
| **TEST-R1-01** | Unit | `crates/runtime::php` | Generate valid PHP-FPM pool config with custom socket path and unprivileged user. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-02** | Unit | `crates/runtime::nginx` | Generate valid non-root Nginx loopback vhost routing `project.test` to unix socket. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-03** | Unit | `crates/runtime::mariadb` | Build isolated datadir args and verify unprivileged port assignment. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-04** | Unit | `crates/runtime::health` | Detect free vs occupied ports (8080, 3306) accurately via socket bind probes. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-05** | Integration | `crates/runtime` | Spawn real PHP-FPM child process, verify `.sock` creation, ping via FastCGI. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-06** | Integration | `crates/runtime` | Spawn Nginx, issue HTTP GET to loopback port 8080, verify 200 OK with PHP info. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-07** | Integration | `crates/runtime` | Send SIGTERM to supervised process group; verify clean shutdown and zero orphan processes (`ps -ef`). | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-08** | Edge Case | `crates/runtime` | Port 8080 intentionally occupied; supervisor reports `DEGRADED (PortConflict)` without crashing. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-09** | Security | `crates/storage` | Attempt path traversal via document_root escaping project directory; verify rejection. | Automated (`cargo test`) | NOT RUN (Target: PASS) |
| **TEST-R1-10** | Hardware | `crates/platform` | Physically unmount external APFS drive while runtime is active; verify graceful shutdown. | Manual Hardware Test | MANUAL VERIFICATION REQUIRED |
| **TEST-R1-11** | UI | `apps/desktop` | Spatial Studio shows real PIDs and memory metrics; UI reflects STARTING -> RUNNING -> STOPPED. | Manual Interactive | MANUAL VERIFICATION REQUIRED |

---

## 3. Automated CI Regression Suite

All automated tests (TEST-R1-01 through TEST-R1-09) will be executed locally and in GitHub Actions CI (`macos-14` Apple Silicon runner) under:
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
