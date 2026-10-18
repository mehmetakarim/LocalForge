# LocalForge — Spatial Studio Visual Vision & Product Philosophy

**Document Version**: 1.0.0  
**Status**: APPROVED DESIGN BASELINE (Planning Phase R1)  
**Authors**: Principal macOS Product Designer, Design Systems Architect, Tauri 2 UX Engineer  

---

## 1. Executive Summary & Design Thesis

LocalForge was initially scaffolded with a functional, Apple-inspired flat dark interface. While clean, modern developer tools often suffer from either sterile SaaS minimalism (flat tables, cold gray borders) or over-stylized "cyberpunk" neon that impedes serious engineering.

**Spatial Studio** introduces an atmospheric, tactile, and spatially grounded design language tailored specifically for macOS Apple Silicon. Inspired by the depth, quiet sophistication, and restrained illumination of next-generation spatial computing interfaces, Spatial Studio treats local and remote development environments as **tangible spatial objects**.

```
+-----------------------------------------------------------------------------------------+
|                                    macOS Traffic Lights                                 |
|                                                                                         |
|   [ Floating Vertical Dock ]      [ Spatial Overview Stage ]                            |
|   +------------------------+      +-------------------------------------------------+   |
|   | (•) Studio             |      |  [Atmospheric Indigo Glow Halo]                 |   |
|   | (•) Projects           |      |                                                 |   |
|   | (•) Services           |      |  +--------------------+  +--------------------+ |   |
|   | (•) Storage            |      |  | Spatial Card 1     |  | Spatial Card 2     | |   |
|   | (•) Deployments        |      |  | (Active PHP 8.4)   |  | (MariaDB 11.4)     | |   |
|   | (•) AI / MCP           |      |  +--------------------+  +--------------------+ |   |
|   |                        |      |                                                 |   |
|   | (•) Settings           |      +-------------------------------------------------+   |
|   +------------------------+                                                            |
|                                                                                         |
|                         [ Floating AI Command Capsule (Cmd+K) ]                         |
+-----------------------------------------------------------------------------------------+
```

### Core Design Values:
1. **Atmospheric Restraint**: Deep navy-black foundations with quiet indigo lighting gradients. Glow is used sparingly to signify vitality and focal depth, never as decorative neon noise.
2. **Tactile Spatial Depth**: Layered Z-axis planes, multi-stop diffuse shadows, 1px specular light rims, and authentic glass refraction create tangible volume.
3. **Dual-Mode Harmony**: Seamless transition between high-level macro awareness (**Spatial Studio**) and micro technical precision (**Native Workspace**).
4. **Honest System Truth**: Every light, pulse, and counter represents real, verified operating system and process telemetry. If a daemon is starting or a volume is disconnected, the spatial object reflects that physical state immediately.
5. **Pure macOS Pedigree**: Flawless window vibrancy, typography tuned to the San Francisco dynamic scale, standard keybindings (`Cmd+K`, `Cmd+1..7`), and zero lag on ProMotion (120Hz) displays.

---

## 2. Visual Character Breakdown

### 2.1 The Atmospheric Dark Canvas
- **Foundation Layer**: `#06080D` (Void Dark) to `#0B0F19` (Deep Navy Graphite).
- **Ambient Lighting**: Subtle radial and elliptical gradients placed behind active workspace clusters. Colors are restricted to deep ultramarine (`rgba(45, 65, 125, 0.25)`) and twilight indigo (`rgba(79, 90, 160, 0.18)`), feathering smoothly over a 300px radius.
- **Surface Texture**: Zero artificial film grain or noise overlays that could muddy text readability. Pure, pristine subpixel anti-aliasing.

### 2.2 Floating Vertical Navigation Dock
- **Architecture**: A detached, pill-shaped floating sidebar suspended 16px from the left window border and centered vertically or hugging the top-left below macOS traffic lights.
- **Material**: Ultra-thick liquid glass (`backdrop-filter: blur(32px) saturate(190%)`), `#0E1320/65%` background, framed by a 1px dual-boundary border (`rgba(255,255,255,0.12)` top/left specular rim; `rgba(0,0,0,0.4)` bottom shadow rim).
- **Navigation Items**: Minimalist line-and-glyph icons with active-state glow pills and silent spring-physics indicator slides.

### 2.3 Three-Dimensional Spatial Project & Server Cards
- **Perspective**: Subtle CSS 3D perspective (`perspective: 1200px`) with interactive micro-tilt on mouse hover (`transform: perspective(1200px) rotateX(1deg) rotateY(-1.5deg) translateZ(8px)`).
- **Specular Rim Highlights**: 1px inner border using `linear-gradient(135deg, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.03) 50%, rgba(255,255,255,0.08) 100%)`.
- **Card Anatomy**:
  - Top header: Status pip, project name, environment tag, external storage indicator.
  - Body: Runtime spec (PHP version, port, socket path, memory gauge).
  - Bottom deck: Live URL trigger, terminal shortcut, deployment preview trigger.

### 2.4 Floating AI Command Capsule
- **Placement**: Anchored floating horizontally centered at the bottom of the stage (or summoned to center screen on `Cmd+K`).
- **Aesthetic**: Slender 44px height capsule with frosted smoked acrylic finish, shimmering indigo border glow, and inline hotkey badges (`⌘`, `K`).
- **Behavior**: Dual-purpose input acting as a global fuzzy project switcher, diagnostic interrogator ("Why is port 8080 blocked?"), and MCP workflow preview generator.

---

## 3. The Two Complementary Interface Modes

LocalForge rejects the false dichotomy between "pretty high-level dashboard" and "cluttered developer IDE". It provides two cohesive presentation modes sharing the exact same underlying state:

```
+----------------------------------------------------------------------------------+
|                               LocalForge Core State                              |
|          (Projects, Storage Volumes, Process Supervisors, Health Probes)         |
+----------------------------------------------------------------------------------+
                            /                                    \
                           v                                      v
          +-------------------------------+      +-------------------------------+
          |         SPATIAL STUDIO        |      |        NATIVE WORKSPACE       |
          |       (Macro Topology)        |      |     (Micro Technical Detail)  |
          +-------------------------------+      +-------------------------------+
          | - 3D Project & Server Objects |      | - High-density split panes    |
          | - Global ambient health halos |      | - Raw stream log viewer       |
          | - Visual storage topology     |      | - FPM pool & Nginx config diff|
          | - One-click launch & overview |      | - Database query console      |
          | - Atmospheric spatial depth   |      | - Checksum deployment ledger  |
          +-------------------------------+      +-------------------------------+
```

### Mode Switcher
Users toggle smoothly between modes via:
- Window header segmented toggle: `[ Spatial Studio | Workspace ]`
- Global keyboard shortcut: `Cmd+Shift+S` (Studio) / `Cmd+Shift+W` (Workspace)
- Contextual drill-down: Clicking any spatial card's "Inspect Deep State" zooms fluidly into the Native Workspace tab for that project.

---

## 4. Native macOS Ergonomics & Licensing Safety

- **Window Treatment**: Full macOS unified titlebar with custom traffic light inset padding (`x: 20px, y: 18px`).
- **Asset Provenance**: 100% custom SVG icons or Lucide open-source MIT assets. Zero proprietary Apple SF Symbols exported to forbidden non-Apple platforms.
- **Window Vibrancy**: Leverages Tauri 2 `macOSPrivateApi: true` with `NSVisualEffectViewMaterialUnderWindowBackground` for authentic window translucency that reacts to macOS wallpaper changes.
