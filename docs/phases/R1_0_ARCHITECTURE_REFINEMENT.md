# LocalForge — Phase R1.0: Architecture Refinement Specification

**Phase**: Recovery Phase R1.0  
**Status**: IN EXECUTION / ACTIVE  
**Parent Milestone**: Recovery Phase R1  
**Deciders**: Principal Rust Engineer, Product Owner, Security Auditor  

---

## 1. Explicit Scope

Phase R1.0 is strictly a design specification, architecture refinement, and planning subphase. No runtime implementation or UI code modification is permitted.

### Included in R1.0 Scope:
1. **Local Domain Resolution Clarity**:
   - Establish loopback HTTP routing via `127.0.0.1:8080` (or `localhost:8080`) as the guaranteed unprivileged baseline.
   - Decouple `.test` browser DNS resolution as an explicitly configured resolver capability requiring separate acceptance tests.
2. **MariaDB Storage Policy Revision**:
   - Mandate internal application storage (`~/Library/Application Support/LocalForge/mariadb/`) as the default database location.
   - Decouple project source files on external APFS drives from internal database storage.
   - Defer external database storage until crash recovery and disconnect safety are fully proven.
3. **Verified Process Identity & Lifecycle Safety**:
   - Specify executable path and start-timestamp verification to prevent PID reuse accidents.
   - Establish dedicated process groups (PGID) with bounded 5s SIGTERM escalation to SIGKILL.
   - Formulate shared Nginx loopback gateway vs independent per-project PHP-FPM pools.
4. **Spatial Studio Performance & Hardware Budgets**:
   - Define measurable CPU (<1%), memory (<80MB), frame rate (60fps baseline, 120fps opportunistic), and blur layer budgets.
   - Establish continuous animation pause guards and DOM virtualization for >12 projects.
5. **R1 Subphase Governance Split**:
   - Partition R1 into four sequential subphases: R1.0, R1.1, R1.2, and R1.3 with strict gateway approvals.

---

## 2. Test Matrix for R1.0

| Test ID | Area | Verification Description | Execution Method | Criteria |
|---|---|---|---|---|
| **TEST-R1.0-01** | Documentation | ADR-0010, ADR-0011, ADR-0012 consistency with PO decisions. | Static inspection | PASS |
| **TEST-R1.0-02** | Documentation | Phase subphase documents (R1.0, R1.1, R1.2, R1.3) present with explicit scopes. | File validation | PASS |
| **TEST-R1.0-03** | Git & Safety | Workspace code remains unpolluted; R0 fail-closed deployment controls intact. | `git status`, `cargo test` | PASS |
| **TEST-R1.0-04** | Performance | Design system documents include measurable CPU, memory, and blur constraints. | Document review | PASS |

---

## 3. Acceptance Criteria

- [x] Local domain resolution accurately distinguishes Host-header routing from browser DNS resolution.
- [x] MariaDB default storage is firmly assigned to internal application data directory.
- [x] ADR-0011 refined and ADR-0012 established with verified process identity.
- [x] Spatial Studio design system updated with measurable performance and animation pause budgets.
- [x] Subphase specifications R1.0 through R1.3 authored with strict acceptance gateways.
- [x] Zero runtime implementation code modified during R1.0.

---

## 4. Evidence of Actual Execution & Artifacts

- **Git Branch**: `phase-r1/r1-0-architecture-refinement`
- **Committed Artifacts**:
  - `docs/architecture/adr/ADR-0010-spatial-studio-design-system.md`
  - `docs/architecture/adr/ADR-0011-real-runtime-process-supervision.md`
  - `docs/architecture/adr/ADR-0012-process-identity-verification-and-storage-isolation.md`
  - `docs/design/SPATIAL_STUDIO_VISION.md`
  - `docs/design/DESIGN_SYSTEM.md`
  - `docs/design/INFORMATION_ARCHITECTURE.md`
  - `docs/design/COMPONENT_SPECIFICATIONS.md`
  - `docs/design/MOTION_AND_MATERIALS.md`
  - `docs/product/FUTURE_MONITORING_VISION.md`
  - `docs/phases/R1_0_ARCHITECTURE_REFINEMENT.md`
  - `docs/phases/R1_1_RUNTIME_IMPLEMENTATION.md`
  - `docs/phases/R1_2_DATABASE_AND_RESILIENCE.md`
  - `docs/phases/R1_3_SPATIAL_STUDIO_IMPLEMENTATION.md`

---

## 5. Gateway Approval Requirement

Transition to **Phase R1.1** is strictly blocked until the Product Owner formally reviews and approves the R1.0 engineering report and architecture deliverables.
