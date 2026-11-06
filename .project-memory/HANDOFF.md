# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Final Acceptance Corrections Completed — Real Host Activation Remains Strict No-Go / Awaiting PO Acceptance**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `b05c89ddd3459c3a33c9ecda3d4f6390ca518f3e`
- **Previous Milestone Status**:
  - **Gate U5.F Final Authorization & Recovery Safety Closeout**: Implemented asymmetric Ed25519 authorization, flock replay ledger, fail-closed staged rollback; conditionally accepted by PO.
- **Completed in Gate U5.F Final Acceptance Corrections**:
  1. `P0-01 — Enrolled Product Owner Trust Anchor Architecture`:
     - Implemented `crates/storage/src/trust_anchor.rs`.
     - Disallowed reading `LOCALFORGE_PO_PUBLIC_KEY` from the execution environment (fails closed if set).
     - Trust anchor enrolled via `localforge storage enroll-trust-anchor` requiring explicit human authorization phrases (`"I AUTHORIZE ENROLLMENT OF PRODUCT OWNER TRUST ANCHOR"` / `"I AUTHORIZE REPLACEMENT OF EXISTING PRODUCT OWNER TRUST ANCHOR"` with `--replace`).
     - Automated agents cannot self-enroll or replace trust anchor without human intervention.
     - Enforced `0600` permissions, UID ownership, non-symlink checks, and SHA-256 fingerprint verification.
     - Added `localforge storage show-trust-anchor` for status inspection.
     - Zero production keys or approval tokens minted during testing or implementation.
  2. `P0-02 — Never Swallow Rollback Errors & Post-Migration Verification`:
     - Eliminated `let _ =` ignored rollback in `tools/cli/src/main.rs` Step 8.
     - Post-migration validation explicitly verifies independent loopback port unique index, project record counts, and Projetest PHP 8.4 compliance.
     - On any verification failure, triggers atomic quarantine and restore, and explicitly propagates results.
     - Verifies restored database is clean Schema V1, contains 2 projects, and Projetest is PHP 8.4.
     - Preserves quarantined failed live state for diagnosis; halts dependent activation steps.
     - Preserves both original failure and rollback failure diagnostics if rollback fails.
     - Added regression tests in `crates/storage/src/rollback.rs` for failed post-migration index validation and failed rollback error propagation.
  3. **Verification**:
     - 27/27 `localforge-storage` tests pass (including trust anchor lifecycle, replacement phrase guard, attacker key substitution rejection, rollback error preservation, and simulated index validation failure).
     - 15/15 `localforge-runtime` integration tests pass.
     - 88/88 library tests pass across all workspace crates.
     - 33/33 desktop frontend tests pass (121 tests total).
     - `cargo clippy --workspace -- -D warnings`: 0 warnings.
     - `cargo fmt --check`: 0 diffs.
     - `localforge storage migrate-v2` dry-run preflight verified on live host database with zero writes.
- **Next Permitted Action**: Await explicit Product Owner acceptance and subsequent activation authorization specifically for Gate U5.F execution before making ANY persistent modifications to `localforge.db`, writing vhosts, or signaling Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
