# LocalForge Design System Specification

**Design System Code**: `Spatial Studio v1.0`  
**Target Platform**: macOS Apple Silicon (`arm64`), WebKit / Tauri 2 Webview  
**Accessibility Target**: WCAG 2.2 Level AA Compliant  

---

## 1. Color Palette & Token Architecture

All design tokens are defined using CSS Custom Properties with full dark and light mode adaptivity. Dark mode is the primary, signature theme.

### 1.1 Foundation Canvas Tokens

| Token Name | Dark Mode Value | Light Mode Value | Semantic Role |
|---|---|---|---|
| `--lf-bg-void` | `#06080D` | `#F2F4F7` | Deepest background beneath transparent windows |
| `--lf-bg-canvas` | `#0A0D15` | `#F8F9FA` | Main workspace stage canvas |
| `--lf-bg-surface-subtle` | `rgba(18, 24, 38, 0.50)` | `rgba(255, 255, 255, 0.65)` | Passive backdrop containers, muted wells |
| `--lf-bg-surface` | `rgba(22, 29, 46, 0.65)` | `rgba(255, 255, 255, 0.82)` | Primary card and panel surface |
| `--lf-bg-surface-elevated`| `rgba(30, 40, 64, 0.78)` | `rgba(255, 255, 255, 0.94)` | Hovered cards, floating dock, modal backings |
| `--lf-bg-surface-floating`| `rgba(38, 50, 80, 0.88)` | `#FFFFFF` | AI Command Capsule, context popovers |

---

### 1.2 Atmospheric Lighting & Accent Tokens

| Token Name | Value | Role |
|---|---|---|
| `--lf-glow-indigo` | `rgba(79, 110, 247, 0.15)` | Primary ambient workspace halo behind spatial clusters |
| `--lf-glow-ultramarine` | `rgba(45, 65, 125, 0.22)` | Deep stage background vignetting |
| `--lf-accent-primary` | `#4F6EF7` | Primary action buttons, active tab indicators, focus rings |
| `--lf-accent-hover` | `#637EF9` | Interactive hover state for primary elements |
| `--lf-accent-active` | `#3D59D6` | Pressed / clicked state for primary elements |
| `--lf-accent-subtle` | `rgba(79, 110, 247, 0.12)` | Active tab pills, badge backgrounds |

---

### 1.3 Border & Specular Highlight Tokens

To achieve authentic glass refraction without noisy raster graphics, LocalForge uses dual-stop specular gradients:

| Token Name | Value (Dark Theme) | Semantic Role |
|---|---|---|
| `--lf-border-glass` | `rgba(255, 255, 255, 0.08)` | Baseline glass containment boundary |
| `--lf-border-subtle` | `rgba(255, 255, 255, 0.12)` | Passive card and container borders |
| `--lf-border-highlight`| `rgba(255, 255, 255, 0.22)` | Top-edge specular rim highlight for 3D realism |
| `--lf-border-active` | `rgba(79, 110, 247, 0.45)` | Selected cards, active input boundaries |
| `--lf-border-glow` | `0 0 0 1px rgba(79,110,247,0.5), 0 0 16px rgba(79,110,247,0.25)` | AI command bar active focus glow |

---

### 1.4 Typography & Text Semantics

| Token Name | Dark Value | Light Value | Contrast Ratio |
|---|---|---|---|
| `--lf-text-primary` | `rgba(255, 255, 255, 0.95)` | `rgba(10, 15, 25, 0.95)` | > 14:1 (AAA) |
| `--lf-text-secondary` | `rgba(255, 255, 255, 0.68)` | `rgba(10, 15, 25, 0.68)` | > 7:1 (AAA) |
| `--lf-text-tertiary` | `rgba(255, 255, 255, 0.45)` | `rgba(10, 15, 25, 0.45)` | > 4.6:1 (AA) |
| `--lf-text-disabled` | `rgba(255, 255, 255, 0.25)` | `rgba(10, 15, 25, 0.25)` | Non-text / disabled |
| `--lf-text-accent` | `#708BFF` | `#2D4ED8` | > 6:1 (AA) |

---

### 1.5 System Status Color Palette (Triple-Channel Signaling)

> [!IMPORTANT]
> **Accessibility Mandate**: Status MUST NEVER be communicated via color alone. Every status state must combine:
> 1. Semantic Color Token
> 2. Specific Icon Glyph (e.g. Check, AlertTriangle, XCircle)
> 3. Text Label or Tooltip

| Status State | Color Token | Glow Token | Icon Glyph | Text Label |
|---|---|---|---|---|
| **RUNNING** | `#10B981` (Emerald) | `rgba(16, 185, 129, 0.25)` | `Play` / `Activity` | "Running" |
| **STOPPED** | `#6B7280` (Cool Gray)| None | `Square` | "Stopped" |
| **STARTING** | `#3B82F6` (Sky Blue) | `rgba(59, 130, 246, 0.30)` | `Loader` (Spinning)| "Starting..." |
| **STOPPING** | `#9CA3AF` (Muted) | None | `Loader` (Spinning)| "Stopping..." |
| **DEGRADED** | `#F59E0B` (Amber) | `rgba(245, 158, 11, 0.25)` | `AlertTriangle` | "Degraded" |
| **FAILED** | `#EF4444` (Crimson) | `rgba(239, 68, 68, 0.30)` | `AlertCircle` | "Failed" |
| **DISCONNECTED**| `#EC4899` (Magenta) | `rgba(236, 72, 153, 0.25)`| `Unplug` | "Volume Disconnected"|
| **UNKNOWN** | `#8B5CF6` (Violet) | None | `HelpCircle` | "Unverified State" |
| **UNAVAILABLE**| `#4B5563` (Dark Slate)| None | `Slash` | "Unavailable" |

---

## 2. Typography System

LocalForge adheres strictly to macOS San Francisco typography metrics with fixed letter spacing and optical sizing.

```css
/* Typography Scale Tokens */
--lf-font-display: "SF Pro Display", -apple-system, BlinkMacSystemFont, sans-serif;
--lf-font-text: "SF Pro Text", -apple-system, BlinkMacSystemFont, sans-serif;
--lf-font-mono: "SF Mono", Menlo, Monaco, "Courier New", monospace;
```

| Type Style | Font Family | Size | Weight | Line Height | Tracking | Use Case |
|---|---|---|---|---|---|---|
| **Display Hero** | Display | 28px | 600 SemiBold | 34px | -0.022em | Spatial Studio stage titles |
| **Title 1** | Display | 20px | 600 SemiBold | 26px | -0.017em | Section headings, primary modal titles |
| **Title 2** | Text | 16px | 600 SemiBold | 22px | -0.011em | Spatial Card titles, panel headers |
| **Headline** | Text | 14px | 600 SemiBold | 18px | -0.006em | Sub-headers, field labels |
| **Body Regular** | Text | 13px | 400 Regular | 18px | 0 | Standard interface text, descriptions |
| **Body Medium** | Text | 13px | 500 Medium | 18px | 0 | Interactive button labels, active pills |
| **Caption** | Text | 11px | 400 Regular | 14px | +0.005em | Metadata tags, timestamps, disk UUIDs |
| **Mono Code** | Mono | 12px | 400 Regular | 16px | 0 (Tabular)| Ports, socket paths, commit SHAs |
| **Mono Micro** | Mono | 10px | 500 Medium | 12px | +0.02em | Badges, SHA-256 prefixes |

---

## 3. Spatial Depth & Elevation Hierarchy

```
[ Layer 4 ]  Modals & Full-Screen AI Command Center (Z: 40, Elevation 4)
             Drop Shadow: 0 24px 64px -8px rgba(0,0,0,0.75), 0 0 0 1px rgba(255,255,255,0.15)
     ^
[ Layer 3 ]  Floating Vertical Navigation Dock & Floating AI Capsule (Z: 30, Elevation 3)
             Drop Shadow: 0 12px 36px -4px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.12)
     ^
[ Layer 2 ]  Interactive Spatial Project & Server Cards (Z: 20, Elevation 2)
             Drop Shadow: 0 8px 24px -2px rgba(0,0,0,0.40), Specular Top Rim: 1px rgba(255,255,255,0.16)
     ^
[ Layer 1 ]  Subtle Panels, Stage Containers & Status Bars (Z: 10, Elevation 1)
             Drop Shadow: 0 2px 8px rgba(0,0,0,0.25)
     ^
[ Layer 0 ]  Atmospheric Void Canvas & Background Vibrancy Halos (Z: 0, Elevation 0)
```

---

## 4. Materials & Glass Effects

### 4.1 Liquid Glass (Floating Dock & AI Capsule)
```css
.lf-material-dock {
  background: rgba(14, 19, 32, 0.68);
  backdrop-filter: blur(36px) saturate(200%);
  -webkit-backdrop-filter: blur(36px) saturate(200%);
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 
    0 16px 40px -6px rgba(0, 0, 0, 0.55),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.18),
    inset 0 0 16px rgba(79, 110, 247, 0.05);
}
```

### 4.2 Spatial Card Material
```css
.lf-material-card {
  background: linear-gradient(
    145deg,
    rgba(24, 32, 52, 0.65) 0%,
    rgba(14, 18, 28, 0.55) 100%
  );
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.09);
  box-shadow: 
    0 8px 28px -4px rgba(0, 0, 0, 0.42),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.16);
  border-radius: 16px;
  transition: transform 0.28s cubic-bezier(0.16, 1, 0.3, 1),
              box-shadow 0.28s cubic-bezier(0.16, 1, 0.3, 1),
              border-color 0.28s cubic-bezier(0.16, 1, 0.3, 1);
}

.lf-material-card:hover {
  transform: translateY(-2px) scale(1.008);
  border-color: rgba(255, 255, 255, 0.20);
  box-shadow: 
    0 16px 44px -6px rgba(0, 0, 0, 0.60),
    0 0 24px -2px rgba(79, 110, 247, 0.18),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.26);
}
```

---

## 5. Interaction States & Transitions

### 5.1 Focus States
- Visible, high-contrast focus rings are mandatory for keyboard accessibility (`Tab`, `Shift+Tab`).
- Ring token: `outline: 2px solid #4F6EF7; outline-offset: 2px;`.
- Never disable `outline` without providing an equivalent accessible ring.

### 5.2 Hover States
- Duration: `180ms` with smooth deceleration curve (`cubic-bezier(0.16, 1, 0.3, 1)`).
- Subpixel scale limited to maximum `1.01x` to avoid blurriness or layout reflows.

### 5.3 Active / Pressed States
- Instant feedback (`60ms` response): `transform: scale(0.985)`.
- Slight darkening of fill and reduction of shadow blur.

### 5.4 Disabled States
- Opacity reduced to `0.40`.
- Cursor set to `not-allowed`.
- Pointer events disabled for mutating buttons; tooltips remain accessible to explain *why* the action is disabled.

### 5.5 Reduced Motion Mode (`prefers-reduced-motion: reduce`)
When user enables reduced motion in macOS System Settings:
- All 3D tilt, hover translations, and continuous pulse animations are completely disabled (`transform: none !important; transition-duration: 0.01ms !important;`).
- State changes use instantaneous, crisp cross-fades.

---

## 6. Performance, GPU & Hardware Constraints

Spatial Studio delivers high visual fidelity without compromising host system efficiency:

### 6.1 Resource Consumption Budgets
- **Idle CPU**: < 1.0% CPU utilization across all cores when the desktop application is idle.
- **Idle Memory**: < 80MB total resident memory footprint (Rust core + WebKit webview combined).
- **Process Awakening**: Background timers and status polling sleep when the window is minimized or hidden (`document.visibilityState === 'hidden'`).

### 6.2 Rendering & GPU Frame Rate Budget
- **Display Adaptivity**: 60fps baseline for standard displays; 120fps opportunistic support on ProMotion displays. 120Hz is an enhancement, NEVER a baseline hardware requirement.
- **Glass Layer Budget**: Maximum 2 concurrent `backdrop-filter: blur(...)` layers on screen. Stacking 3 or more blur layers is strictly prohibited to prevent GPU fill-rate exhaustion.
- **Continuous Animation Guard**: Ambient halos and breathing status pulses MUST automatically pause when:
  1. System state is static (no active transition or error condition).
  2. The window is out of focus (`window.onblur`).
  3. Reduced motion is requested.

### 6.3 Scalability & Virtualization
- **Large Project Catalogs**: When managing > 12 projects, Spatial Studio cards automatically paginate or utilize DOM virtualization (`react-window` / virtual grid) to prevent DOM node bloat and compositor slowdowns.
- **Window Resizing**: Layout utilizes CSS Grid with fractional units. Resize recalculations are throttled (`16ms` / `requestAnimationFrame`) to prevent stutter during live window drag-resize.
