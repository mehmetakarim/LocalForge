# ADR-0005: Runtime Distribution and Isolated PHP-FPM Architecture

- **Status**: PROPOSED (Requires Product Owner Review)
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Senior Rust Engineer, Systems Engineer
- **Approval Requirement**: Product Owner Approval Required

---

## 1. Context

LocalForge needs to provide and manage:
- Multiple PHP versions (e.g. PHP 8.1, 8.2, 8.3, 8.4) running concurrently.
- Web server routing (Nginx).
- Relational database (MariaDB).

Two primary distribution approaches exist:
1. **Host Package Discovery (e.g., Homebrew)**: Relying on existing packages installed on the user's Mac.
2. **LocalForge-Managed Runtime Packages (Bottles/Tarballs)**: Downloading and unpacking self-contained, pre-compiled runtime binaries into `~/.localforge/runtimes/`.

---

## 2. Decision

We propose a **Phased Hybrid Runtime Strategy**:

### Phase 2 Bootstrap: Guided Homebrew Discovery & Management
- LocalForge detects existing Homebrew installations (`/opt/homebrew` on Apple Silicon).
- If present, LocalForge discovers installed PHP versions, Nginx, and MariaDB binaries.
- If absent or missing specific versions, LocalForge generates a transparent, human-readable command recommendation (e.g., `brew install php@8.2`) rather than running package managers silently in the background.

### Future Phase Target: Self-Contained Managed Runtime Packages
- For true zero-config operation (especially for non-Homebrew users), LocalForge will distribute pre-compiled, notarized macOS arm64 runtime packages extracted into isolated LocalForge cache directories.

### Runtime Isolation Model
- **Independent PHP-FPM Pools**: Each project runs its own dedicated Unix domain socket (e.g. `~/.localforge/run/project-xyz-php82.sock`). No project shares an FPM master pool.
- **No Global Version Switching**: Projects declare their PHP version in `.localforge/project.yaml`. Switching one project's runtime does not impact other active projects.
- **Unprivileged Loopback**: All services bind exclusively to loopback addresses (`127.0.0.1` / Unix domain sockets) on unprivileged ports (>1024).

---

## 3. Alternatives Considered

### Alternative A: Docker Containers for Everything
- *Rejected*: Extreme resource consumption, slow filesystem sync on external SSDs, high battery drain, and requires Docker Desktop (licensing implications).

### Alternative B: Direct Global Brew Mutations
- *Rejected*: Silently running `brew install` or `brew link` disrupts developer machines and violates Product Principle #3 & #10.

---

## 4. Consequences & Risks

### Positive
- Projects can run completely different PHP versions simultaneously without conflict.
- Zero administrative permissions required; developer's global environment remains untouched.

### Negative / Risks
- Relies on Homebrew during initial Phase 2, which requires users to have Homebrew installed until pre-compiled packages are hosted.

---

## 5. Verification Status

- Homebrew 7.0.7 verified present on host machine during Phase 0 audit.
- Strategy pending PO review before Phase 2 implementation.
