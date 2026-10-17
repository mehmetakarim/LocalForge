# ADR-0001: Tauri 2 and Modular Rust Workspace Architecture

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Senior Rust Engineer, Desktop Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

LocalForge requires a high-performance, native desktop application on macOS (Apple Silicon arm64) that controls local server runtimes (PHP-FPM, Nginx, MariaDB), provides an interactive UI, and exposes headless interfaces (CLI and MCP server). The application must operate with minimal memory footprint, strict system security, and native platform integration.

Traditional solutions (e.g. Electron) carry substantial overhead (often exceeding 400MB idle RAM, large bundle sizes, and complex IPC security models). Furthermore, native Swift/AppKit development would limit long-term cross-platform extensibility (e.g. planned future Windows support).

---

## 2. Decision

We choose **Tauri 2** combined with a **Modular Rust Workspace** and a **React 19 / TypeScript / Vite / Tailwind CSS** frontend:
1. **Desktop Shell**: Tauri 2 leveraging macOS native WebKit (WKWebView).
2. **Backend Engine**: A modular Cargo workspace (`crates/*`, `tools/*`, `apps/*`).
3. **Frontend**: Modern React with TypeScript and Vite, styled with Tailwind CSS and accessible component patterns.

---

## 3. Alternatives Considered

### Alternative A: Electron
- *Pros*: Vast ecosystem, uniform Chromium behavior.
- *Cons*: Prohibitive resource consumption (>300–500MB idle RAM), bundle size >150MB, security concerns with Node integration. Disqualified due to product performance principles.

### Alternative B: Flutter Desktop
- *Pros*: Single codebase, fast rendering.
- *Cons*: Non-native text rendering on macOS, complex C FFI bindings to system storage APIs, less native look-and-feel.

### Alternative C: Pure Native (Swift / SwiftUI)
- *Pros*: Unmatched macOS integration and aesthetics.
- *Cons*: Cannot be shared with planned future Windows version; restricts the ability to run the exact same core engine in headless CLI and MCP environments.

---

## 4. Rationale

- **Performance & Footprint**: Tauri 2 binaries are compact (<20MB) and consume <40MB idle RAM.
- **Security**: Tauri 2 introduces a granular permissions model and strict IPC isolation.
- **Reusability**: Rust domain crates can be compiled directly into the Tauri app, a standalone CLI binary, or an MCP server without translation layers or IPC bridges.

---

## 5. Consequences & Risks

### Positive
- Exceptional responsiveness on Apple Silicon.
- Deep integration with macOS security frameworks (Keychain, IOKit).
- Zero-overhead core logic sharing across UI, CLI, and MCP.

### Negative / Risks
- WebKit engine differences between macOS versions (mitigated by target baseline of macOS 13+).
- Requires Rust compilation toolchain in CI and developer environments.

---

## 6. Verification Status

- Verified host environment has `rustc 1.94.0`, Apple Clang 21.0.0, and Node 20.19.5 installed.
- Scaffolding scheduled for Phase 1 (TASK-101).
