# LocalForge Component Specifications

**Document Version**: 1.0.0  
**Design Library**: `Spatial Studio Component Suite`  
**Framework**: React 19 + TypeScript + Tailwind CSS (v4) / Tauri 2  

---

## 1. Component 1: `SpatialCard`

The signature building block of the Spatial Studio view. Represents a self-contained local workspace, database instance, or runtime cluster as a tangible 3D object.

### 1.1 Visual Anatomy
```
+-------------------------------------------------------------+  <- 1px Specular Top Rim (rgba(255,255,255,0.22))
| [Status Pip]  [Project Name]             [Storage Tag: SSD] |  <- Header
|  ● RUNNING    StepperSkip Store           ⛁ Mac Harici Disk |
+-------------------------------------------------------------+
|                                                             |
|   Runtime: PHP 8.4 (FPM Socket :8080)                       |  <- Body Metrics
|   Local Host: http://stepperskip.test                       |
|   Active Memory: 24.5 MB | CPU: 0.2%                        |
|                                                             |
+-------------------------------------------------------------+
| [Open in Browser ↗]  [View Logs]             [Settings ⚙]   |  <- Action Footer
+-------------------------------------------------------------+
```

### 1.2 TypeScript Interface
```typescript
export interface SpatialCardProps {
  id: string;
  title: string;
  status: "RUNNING" | "STOPPED" | "STARTING" | "STOPPING" | "DEGRADED" | "FAILED" | "DISCONNECTED";
  statusMessage?: string;
  phpVersion: string;
  localDomain: string;
  volumeName: string;
  isExternalVolume: boolean;
  activePort?: number;
  cpuPercent?: number;
  memoryMb?: number;
  onStart?: () => void;
  onStop?: () => void;
  onOpenBrowser?: () => void;
  onOpenLogs?: () => void;
  onConfigure?: () => void;
}
```

### 1.3 Micro-Interactions & Styling
- **Perspective**: Wrapped in a container with `perspective: 1000px`.
- **Hover Tilt**: On `mousemove`, calculates offset from center:
  ```typescript
  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;
    setTransform(`perspective(1000px) rotateY(${x * 6}deg) rotateX(${-y * 6}deg) translateY(-3px)`);
  };
  ```
- **Drop Shadow**: Multi-stop elevation shadow:
  `box-shadow: 0 12px 32px -4px rgba(0, 0, 0, 0.45), 0 0 24px -2px rgba(79, 110, 247, 0.12), inset 0 1px 0 0 rgba(255, 255, 255, 0.20)`.

---

## 2. Component 2: `FloatingDock`

The detached, vertical navigation dock providing frictionless switching between the primary modules.

### 2.1 Anatomy
- **Orientation**: Vertical capsule suspended at `left: 20px`, vertically centered or anchored 60px below top window frame.
- **Surface**: Thick liquid glass (`background: rgba(14, 19, 32, 0.65)`, `backdrop-filter: blur(32px)`).
- **Navigation Items**:
  - `Studio` (`Compass` / `Sparkles`)
  - `Projects` (`Layers`)
  - `Services` (`Cpu` / `Server`)
  - `Storage` (`HardDrive`)
  - `Deployments` (`CloudUpload`)
  - `AI / MCP` (`Bot` / `Cpu`)
  - `Settings` (`Settings`)
- **Active Indicator**: Floating glowing pill (`background: rgba(79, 110, 247, 0.25)`, `border: 1px solid rgba(79, 110, 247, 0.5)`) that slides with spring physics to the active item.
- **Accessible Tooltips**: Expands an accessible tooltip to the right (`left: 100% + 12px`) displaying the module name and keybinding shortcut (e.g. `⌘1`).

---

## 3. Component 3: `AICommandCapsule` & Modal Command Center

The interactive entry point for system discovery, diagnostic interrogation, and MCP agent coordination.

### 3.1 Closed State (Floating Capsule)
- **Position**: Centered horizontally at the bottom edge (`bottom: 24px`), width: `440px`, height: `44px`.
- **Styling**: Smoked glass capsule with a soft glowing border:
  `border: 1px solid rgba(255, 255, 255, 0.14)`, `box-shadow: 0 12px 36px rgba(0,0,0,0.5), 0 0 16px rgba(79,110,247,0.15)`.
- **Content**: Subtle sparkling icon, placeholder `"Ask LocalForge or jump to project... (⌘K)"`, and quick keyboard shortcut badge.

### 3.2 Open State (Modal Command Center)
- Triggered by clicking the capsule or pressing `Cmd+K`.
- Fluidly expands into a centered modal dialog (`width: 640px`, `max-height: 480px`).
- **Input Field**: High-contrast, autofocus search bar.
- **Sectioned Results**:
  1. *Quick Actions*: "Create New PHP Project", "Inspect Port 8080", "Scan External SSD".
  2. *Projects*: Fuzzy match across all registered project names and local domains.
  3. *AI Diagnostic Prompts*: "Why did PHP-FPM fail to bind socket?", "Generate deployment preview for staging".
- **Truthful MCP Indicator**: Clearly denotes whether AI responses are locally computed from system state or executed via MCP tools.

---

## 4. Component 4: `StorageTopologyCard`

Visual representation of internal vs external APFS storage volumes with physical safety guards.

### 4.1 Visual Anatomy
- **Disk Type Glyph**: Custom SVG hardware icon depicting external USB-C/Thunderbolt SSD vs internal Apple Silicon NAND.
- **Volume Identity**: Human label (e.g., `Mac Harici Disk`), mount point (`/Volumes/...`), and persistent APFS UUID.
- **Capacity Indicator**: Two-tone progress bar with free space in high-contrast emerald and used space in dark indigo.
- **Safety Indicator**:
  - Connected: "● Volume Barrier Active (UUID Verified)".
  - Disconnected: "▲ Disconnected — Writes Safely Blocked".

---

## 5. Component 5: `ServiceHealthPill`

A compact, triple-channel operational status chip.

### 5.1 States
- **Running**: Emerald circle glyph + pulsing halo + `"8080 Active"`.
- **Degraded**: Amber warning triangle + static border + `"Port Conflict :8080"`.
- **Stopped**: Gray square glyph + muted border + `"Stopped"`.
- **Interactive**: Clicking opens a popover displaying the managed PID, socket file path (`.localforge/run/php-fpm.sock`), and direct restart button.

---

## 6. Component 6: `DeploymentPreviewCard`

Renders the audited cryptographic diff prior to any remote transfer, reinforcing the fail-closed security boundary.

### 6.1 Security Features
- **Diff Metrics**: Badges for Added (`+N`), Modified (`~M`), and Deleted (`-D`) files.
- **Checksum Proof**: Displays computed manifest SHA-256 fingerprint.
- **Default-Deny Badge**: "✓ Secrets Excluded (.env, .git/, keys verified absent)".
- **Phase R0 Security Shield**: Prominently informs the user:
  > **Remote Deployment Inactive in Recovery R0**  
  > Non-dry-run SFTP upload is disabled to prevent unverified mutations. Only dry-run cryptographic simulation is permitted.
- **Action Buttons**: `Run Simulation (Dry-Run)` (Active) vs `Deploy to Production` (Disabled with lock icon and tooltip explanation).

---

## 7. Component 7: `SpecularModal`

A window-level glass modal dialogue for destructive operations, project creation, and deep diagnostics.

### 7.1 Specifications
- **Backdrop**: `rgba(0, 4, 12, 0.65)` with `backdrop-filter: blur(20px)`.
- **Container**: `rgba(20, 27, 44, 0.88)`, `border: 1px solid rgba(255,255,255,0.18)`, `border-radius: 20px`.
- **Header**: Large title, close button (`Esc`), and subtle specular lighting sweep.
- **Actions**: High-contrast Primary Button (`#4F6EF7` gradient) paired with Ghost Cancel Button.
