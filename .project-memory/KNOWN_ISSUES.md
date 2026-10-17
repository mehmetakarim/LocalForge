# LocalForge — Known Issues & Limitations

This document tracks known issues, technical defects, environmental constraints, and architectural limitations discovered during development.

---

## Active Issues & Environmental Constraints

### ISSUE-001: External Volume Mount Point Volatility on macOS
- **Classification**: Environmental Constraint / System Behavior
- **Discovered In**: Phase 0 (Environment Audit)
- **Component**: `crates/storage` & `crates/platform`
- **Description**: On macOS, external drives unmounted abruptly or remounted with name collisions may receive modified mount paths (e.g., `/Volumes/Mac Harici Disk 1` instead of `/Volumes/Mac Harici Disk`).
- **Impact**: Relying on absolute string paths will cause false-negative workspace disconnect errors or broken file paths.
- **Resolution Plan / Mitigation**: Mitigated via ADR-0004. LocalForge will record filesystem/hardware UUIDs (via DiskArbitration / IOKit) in `.localforge/project.yaml` and SQLite metadata. The path will be dynamically resolved against active volumes matching the UUID.
- **Status**: ARCHITECTED (Implementation planned in Phase 1)

### ISSUE-002: Non-Root Port Binding Limitation (<1024)
- **Classification**: System Constraint
- **Discovered In**: Phase 0 (Architecture Research)
- **Component**: `crates/runtime`
- **Description**: Binding to standard HTTP port 80 and HTTPS port 443 on macOS typically requires `root` privileges. Asking users for `sudo` password violates the least-privilege architectural principle.
- **Impact**: Direct `http://myproject.test` without port suffix would require elevated privilege if using standard port 80 directly.
- **Resolution Plan / Mitigation**: In Phase 2, LocalForge will bind Nginx to high unprivileged loopback ports (e.g., `127.0.0.1:8080`) and offer an unprivileged local PAC (Proxy Auto-Configuration) or isolated loopback aliasing approach (`pfctl` rules only with explicit one-time user authorization, or default port-based local URLs).
- **Status**: MITIGATION DESIGNED (Evaluation in Phase 2)

### ISSUE-003: Homebrew Dependency Variance
- **Classification**: Toolchain Dependency
- **Discovered In**: Phase 0 (Task 0.2 Research)
- **Component**: `crates/runtime`
- **Description**: End-user Homebrew installations vary widely across developers (different PHP versions, customized compile flags, varying Homebrew prefix `/opt/homebrew` vs `/usr/local`).
- **Impact**: Inconsistent runtime behavior if relying solely on user's pre-existing Homebrew packages.
- **Resolution Plan / Mitigation**: Documented in ADR-0005. Phase 2 will support Homebrew discovery with explicit path and version validation, paving the way for self-contained, pre-compiled LocalForge runtime bottles.
- **Status**: MITIGATION DESIGNED (Phase 2)
