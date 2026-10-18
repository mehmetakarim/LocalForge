# LocalForge — Phase R1.1: Real PHP-FPM & Nginx Integration Specification

**Phase**: Recovery Phase R1.1  
**Status**: SPECIFIED / AWAITING R1.0 GATEWAY APPROVAL  
**Prerequisite**: Formal Product Owner Approval of Phase R1.0  
**Deciders**: Principal Rust Engineer, Systems Engineer, QA Lead  

---

## 1. Explicit Scope

Phase R1.1 focuses exclusively on live process supervision for PHP-FPM and the shared Nginx loopback gateway. Database integration (MariaDB) and UI redesign are strictly deferred to R1.2 and R1.3.

### In Scope for R1.1:
1. **Unprivileged Child Process Spawning**:
   - Discover installed PHP-FPM binaries on macOS (`/opt/homebrew`, `/usr/local`).
   - Spawn PHP-FPM as an unprivileged child process with dedicated pool configs.
   - Bind FastCGI communication exclusively to isolated Unix domain sockets (`.localforge/run/php-fpm.sock`) with `0660` permissions.
2. **Shared Nginx Gateway Management**:
   - Manage a single, unprivileged Nginx master process on loopback port `8080`.
   - Generate modular virtual host definitions in `.localforge/nginx/conf.d/*.conf`.
   - Implement graceful reload (`nginx -s reload`) on project start/stop.
3. **Verified Process Identity & PID Reuse Defense**:
   - Track PID, PGID, binary canonical path, and launch timestamp in an in-memory registry.
   - Verify identity via operating system checks before issuing signals.
   - Enforce bounded 5s graceful `SIGTERM` before `SIGKILL` escalation.
4. **Guaranteed Loopback HTTP Verification**:
   - Test and verify real HTTP GET requests to `http://127.0.0.1:8080` (with Host header dispatch) returning HTTP 200 OK and PHP execution payloads.
5. **Decoupled `.test` Domain Architecture**:
   - Document and test `.test` resolver integration as an independent capability requiring macOS `/etc/resolver/test` configuration.

---

## 2. Test Matrix for R1.1

| Test ID | Level | Target | Test Procedure | Acceptance Threshold |
|---|---|---|---|---|
| **TEST-R1.1-01** | Unit | `crates/runtime::php` | Generate isolated FPM pool configuration with user permissions and custom socket. | Exact config validation, 0 warnings |
| **TEST-R1.1-02** | Unit | `crates/runtime::nginx`| Build loopback virtual host routing `project.test` to unix socket on port 8080. | Syntax validation with `nginx -t -c ...` |
| **TEST-R1.1-03** | Integration | `crates/runtime::supervisor`| Spawn real PHP-FPM master process, verify socket creation and PID registration. | Process active, socket exists, verified binary |
| **TEST-R1.1-04** | Integration | `crates/runtime::supervisor`| Spawn Nginx master on port 8080; issue HTTP GET; assert HTTP 200 and PHP body. | Exit code 0, 200 OK received |
| **TEST-R1.1-05** | Security | `crates/runtime::supervisor`| Attempt PID reuse attack: artificially alter recorded start timestamp; verify signal abort. | `kill` rejected; unrelated process unharmed |
| **TEST-R1.1-06** | Lifecycle | `crates/runtime::supervisor`| Stop Project A; verify Project A's FPM pool stops while shared Nginx remains running. | Pool A stopped; Nginx active; 0 orphans |
| **TEST-R1.1-07** | Recovery | `crates/runtime::supervisor`| Simulate FPM crash (kill -9); verify supervisor detects exit and removes stale socket. | Status FAILED; stale socket cleared |

---

## 3. Acceptance Scenario & Exit Criteria

1. **APFS External Workspace**: Project initialized on `/Volumes/Mac Harici Disk/...`.
2. **Live Runtime Execution**: Nginx (port 8080) and PHP-FPM (socket) running with verified PIDs.
3. **HTTP 200 Verification**: `curl -s -H "Host: myproject.test" http://127.0.0.1:8080/` returns expected PHP output.
4. **Clean Termination**: Stop command terminates PHP-FPM cleanly; zero lingering processes verified via `pgrep`.
5. **Workspace Tests**: All tests in `cargo test --workspace` pass with 0 failures and 0 clippy warnings.

---

## 4. Evidence & Handoff Requirements

- Git commit on branch `phase-r1/r1-1-runtime-implementation`.
- Terminal logs proving live HTTP request output and clean `pgrep` checks.
- Product Owner approval required before proceeding to R1.2.
