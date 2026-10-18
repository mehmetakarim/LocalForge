# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Phase R1.0 — Architecture Refinement Completed**
- **Active Git Branch**: `phase-r1/r1-0-architecture-refinement`
- **Target Remote**: `origin/phase-r1/r1-0-architecture-refinement`
- **Previous Milestone Status**:
  - **Recovery Phase R0**: CONDITIONALLY ACCEPTED (18/18 tests pass, CI Green).
  - **Spatial Studio Visual Direction**: APPROVED by Product Owner.
  - **R1 Subphase Split**: Formalized into R1.0, R1.1, R1.2, R1.3.
- **Completed Tasks in R1.0**:
  - `TASK-R1.0-01`: Local Domain Resolution & Loopback Architecture (Host-header vs `.test` DNS resolver decoupled).
  - `TASK-R1.0-02`: MariaDB Storage Policy (Internal application data storage mandated as default).
  - `TASK-R1.0-03`: Verified Process Identity & Lifecycle Safety (Executable path & start-ticks PID reuse defense).
  - `TASK-R1.0-04`: Spatial Studio Performance & Hardware Budgets (CPU <1%, RAM <80MB, max 2 blurs, pause guards).
  - `TASK-R1.0-05`: R1 Subphase Governance & Spec Documents (`R1_0` through `R1_3`).
  - `TASK-R1.0-06`: Architecture Decision Records (`ADR-0010`, `ADR-0011` amended, `ADR-0012`).
- **Next Permitted Action**: Present R1.0 engineering report and await Product Owner approval to begin Phase R1.1 implementation.

---

## 2. Invariants & Guardrails

1. **Strict Plan & Architecture Only**: Zero runtime or UI implementation code was modified in R1.0.
2. **Fail-Closed Deployment Intact**: Non-dry-run deployment execution remains rejected with `LocalForgeError::DeploymentNotImplemented`.
3. **No Unverified Claims**: Loopback routing is restricted to `127.0.0.1:8080`; browser `.test` resolver requires explicit configuration.
4. **Data Isolation**: MariaDB database files default to internal SSD storage (`~/Library/Application Support/LocalForge/mariadb/`).
5. **Phase Gate Enforcement**: Do NOT start R1.1 until Product Owner formally signs off on R1.0.
