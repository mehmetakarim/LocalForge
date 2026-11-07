# LocalForge — Session Handoff Document

> **Authoritative handoff protocol for AI coding assistants (Claude Code, Codex, Antigravity) and human engineers.**

---

## 1. Current Status

- **Current State**: **Gate U5.F Trust Anchor Enrollment Closeout Complete — Real Host Activation Remains Strict No-Go / Awaiting Product Owner Manual Action**
- **Active Git Branch**: `phase-r1/r1-3b-uat-recovery`
- **Target Remote**: `origin/phase-r1/r1-3b-uat-recovery`
- **Baseline Commit**: `4a165c2180bacd47088a25546a836bc54b905aa8`
- **Previous Milestone Status**:
  - **Gate U5.F Final Acceptance Corrections**: Implemented enrolled trust anchor architecture and rollback error preservation (Commit `4a165c2`).
- **Completed in Gate U5.F Trust Anchor Enrollment Closeout**:
  1. `Clarification of Static Authorization Phrases & Threat Boundary`:
     - Explicitly documented that confirmation phrases are intent confirmation guards to prevent accidental CLI execution, not cryptographic human authentication.
     - Documented macOS / POSIX DAC reality: Any process running as UID 501 (`mehmetakar`) has access to files in `~/Library/Application Support/LocalForge/security/`. We honestly state this OS boundary and residual risk.
     - The real security boundary relies on: (a) offline Ed25519 private key custody, (b) out-of-band manual enrollment by PO, (c) independent fingerprint audit, and (d) fail-closed trust anchor verification.
  2. `Independent Product Owner Key Generation & Enrollment Workflow`:
     - Documented step-by-step keypair generation in `~/.localforge_secrets/po_ed25519_private.pem` (mode 0600) via OpenSSL / Python outside workspace.
     - Documented raw 32-byte public key hex extraction and independent SHA-256 fingerprint computation.
     - Strict rule: private key never enters workspace, repository, logs, or agent conversation.
  3. `Two-Gate Authorization Architecture`:
     - **Gate U5.F-E**: Explicit human authorization for manual trust anchor enrollment and fingerprint verification.
     - **Gate U5.F-M**: Explicit human authorization for live database migration using an independently signed approval token.
  4. `Proposal Runbook & Fail-Closed Matrix Update`:
     - Updated Section 8 of `docs/phases/R1_3B_U5F_CONTROLLED_ACTIVATION_PROPOSAL.md` with full workflows and fail-closed matrix (10 distinct failure cases).
  5. `Verification Suite`:
     - 27/27 `localforge-storage` tests pass.
     - 88/88 workspace Rust tests pass.
     - 33/33 desktop frontend tests pass (121 tests total).
     - `cargo fmt --check`: 0 diffs.
     - `cargo check --workspace`: Clean.
- **Next Permitted Action**: Await explicit Product Owner manual action for Gate U5.F-E (Trust Anchor Enrollment) in native terminal. Real Mac activation remains STRICT NO-GO. Do NOT modify `localforge.db`, do NOT write vhosts, do NOT signal Nginx. Do NOT begin Gate U5.C, U5.D, or R1.3C. Do NOT merge into `main`.

---

## 2. Invariants & Guardrails

1. **Dedicated Branch**: Work strictly isolated on `phase-r1/r1-3b-uat-recovery`; never merge into `main` without PO approval.
2. **Projetest Invariant**: Project `Projetest` retains PHP 8.4 until explicit user/PO confirmation.
3. **MariaDB Protection**: MariaDB datadir on Application Support remains uninitialized pending explicit PO authorization.
4. **No Root / Sudo Execution**: All runtime processes (PHP-FPM, Nginx, MariaDB) run strictly unprivileged under user ownership.
5. **Verified Process Identity**: Nginx and PHP-FPM supervisors enforce exact canonical config path matching and Darwin kernel process ownership.
6. **No Fabricated Telemetry**: In production builds, synthetic metrics and fixtures are strictly rejected; zero fake charts.
