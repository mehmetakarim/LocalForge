# ADR-0004: External Workspace Storage Model and Disconnect Resilience

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal Software Architect, Systems Engineer, Security Engineer
- **Approval Requirement**: Technical Team Consensus (Achieved)

---

## 1. Context

Supporting projects hosted on external SSDs (e.g. `/Volumes/DeveloperSSD/Projects/stepperskip`) is a foundational value proposition of LocalForge. However, external storage on macOS introduces critical engineering challenges:
1. **Mount Path Volatility**: An external drive unmounted uncleanly or remounted with a duplicate name may mount at `/Volumes/DeveloperSSD 1`.
2. **Accidental Internal Recreation**: If a drive is disconnected and an automated process or IDE attempts to access or write to the expected path, macOS will create an empty folder inside `/Volumes/DeveloperSSD` on the internal system disk.
3. **Drive Swap Hazard**: A different USB drive with the same label could be plugged in, leading to data corruption or accidental file overwrites.
4. **Data Category Conflicts**: Storing volatile runtime packages, large caches, or MariaDB transaction logs on an external drive can cause severe database corruption upon sudden physical disconnection.

---

## 2. Decision

We mandate a **Persistent Volume Identity Model** and strict **Storage Category Partitioning**:

### 2.1 Persistent Volume Identity
- Every external project registered with LocalForge records:
  - `volume_uuid`: Filesystem / Hardware UUID (obtained via DiskArbitration / `statfs`).
  - `mount_path`: Last known mount point.
  - `relative_path`: Path relative to the volume mount root.
- **Pre-Flight Volume Check**: Before any project operation (start, edit, deploy), LocalForge verifies that the volume mounted at `mount_path` matches `volume_uuid`. If mismatched or absent, the operation immediately halts with `Error::VolumeDisconnected`.
- **Directory Creation Barrier**: LocalForge will **NEVER** automatically invoke `create_dir_all` on a path under `/Volumes/` without first verifying the underlying volume is mounted and healthy.

### 2.2 Storage Category Partitioning
1. **User Project Source & Manifest**: Resides on the user-selected volume (Internal or External).
2. **LocalForge SQLite Metadata**: Stored exclusively on internal disk (`~/Library/Application Support/LocalForge/localforge.db`).
3. **MariaDB Database Data**: Kept on internal SSD by default for corruption prevention. External database storage is supported only via explicit, validated opt-in.
4. **Runtime Caches & Logs**: Placed on internal disk to avoid external disk wear and corruption during surprise disconnection.

---

## 3. Alternatives Considered

### Alternative A: Relying Solely on Absolute File Paths
- *Rejected*: Inevitably leads to the accidental internal directory recreation bug and fails when mount paths change dynamically.

### Alternative B: Forcing Projects to Internal Disk Only
- *Rejected*: Direct violation of Product Principle #6 ("External storage first-class support").

---

## 4. Consequences & Risks

### Positive
- Robust protection against data loss, silent path redirects, and disk confusion.
- Safe degraded state: when an external disk is unplugged, the UI reflects "Drive Disconnected" and blocks all mutations cleanly.

### Negative / Risks
- Requires native macOS filesystem and disk arbitration inspection.

---

## 5. Verification Status

- Verified active test environment is running on an external APFS volume (`/Volumes/Mac Harici Disk`).
- Implementation scheduled for Phase 1 (TASK-103).
