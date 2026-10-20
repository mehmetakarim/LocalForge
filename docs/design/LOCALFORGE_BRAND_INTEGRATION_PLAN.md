# LOCALFORGE BRAND INTEGRATION PLAN & DESIGN SPECIFICATION (STAGE A — REVISION 1)

> **Document Status**: Authoritative Visual Design Revision — Stage A Final Candidate  
> **Target Release**: LocalForge R1.3B  
> **Brand Kit Source**: `LocalForge-Logo-Kiti.zip` (Version 1.0, October 2026)  
> **Review Package**: `docs/design/r1_3b/brand_aligned/localforge_brand_assets.zip`  
> **Execution Boundary**: Pure Visual Design Gate. Zero modifications to production React, Rust, or Tauri application code.

---

## 1. Revision Overview & Response to Visual Findings

This revision addresses all four Product Owner review findings from the initial Stage A submission:

### 1.1 Resolution of Finding 01 — Resolved Brand Asset Rendering
- **Root Cause**: The previous HTML prototypes referenced brand raster images via an incorrect relative depth (`../../../` instead of `../../../../`), resulting in unresolved image elements in Chrome headless renders.
- **Correction**:
  1. A dedicated local asset package has been established at `docs/design/r1_3b/brand_aligned/assets/` holding the exact canonical assets:
     - `localforge-emblem.png` (1.6 MB original raster emblem)
     - `localforge-logo.png` (4.1 MB original brand lockup)
     - `localforge-app-icon.png` (1.2 MB 1024×1024 master icon)
  2. All HTML prototypes now resolve direct sibling assets (`assets/localforge-emblem.png` and `assets/localforge-app-icon.png`), ensuring 100% reproducible offline loading.
  3. A self-contained review archive has been built: [`localforge_brand_assets.zip`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/localforge_brand_assets.zip).
  4. Every image has been verified with `naturalWidth > 0` before and after screenshot rendering.

### 1.2 Resolution of Finding 02 — Restoration of Direction A+ Spatial Studio Composition
- **Root Cause**: The previous iteration inadvertently drifted toward dense horizontal technical cards, losing the spatial signature of the approved baseline.
- **Correction**:
  - Restored **tall vertical translucent project monoliths** (`width: 260px`, `min-height: 330px`) on a centered stage.
  - Restored **perspective floor and subtle radial reflections** (`perspective-floor` and `stage-reflection`).
  - Restored **strong negative space** and centered hero typography (`Tüm servisler etkin.`).
  - Restored **floating left navigation dock** and **right count switcher** (0, 2, 6 project modes).
  - Restored **bottom AI command capsule** (`bottom-capsule`) with pulsing orb and connected workspace pill (`/Volumes/Mac Harici Disk > LocalForge`).
  - Applied official brand palette (`--lf-night`, `--lf-plum`, `--lf-dusk`, `--lf-violet`, `--lf-lilac`, `--lf-mist`, `--lf-white`) exclusively as material tints and accents without altering spatial geometry.

### 1.3 Resolution of Finding 03 — Native Workspace Polish
- **Projects Workspace**: Enhanced readability, clean project cards with clear distinction between primary actions and secondary details, preserved search and filter pills.
- **Services & Runtime**: Elevated telemetry typography to 12px–14px for effortless legibility; exact commandline, PID, and configuration evidence preserved.
- **Storage & Volumes**: Grounded APFS container contracts, persistent volume UUIDs (`E4B30F82-9C1D-4E7B-8F09-2D88C523B0A1`), and safe disconnect protection indicators.
- **Project Inspector**: Meaningful Direction A+ monolith stage preserved behind the slide-over drawer; section spacing expanded to 22px–26px; database deletion strictly disabled with red cautionary guard; SFTP deployment strictly enforced as "Dry-Run Only".

### 1.4 Resolution of Finding 04 — Honest Prototype Data Declaration
- All six prototypes explicitly declare static illustrative fixtures via a dedicated pill: `İllüstratif Statik Prototip`.
- No claims of active live backend telemetry are made for static mockups.
- Destructive operations remain locked and inoperable.

---

## 2. Canonical Brand Tokens & Semantic Isolation

```css
:root {
  /* Canonical Brand Palette (LocalForge Logo Kiti v1.0) */
  --lf-night:  #191337;  /* Gece: Base canvas, deepest shadows */
  --lf-plum:   #2B2450;  /* Erik: Spatial glass surfaces, cards, drawers */
  --lf-dusk:   #3B376A;  /* Alacakaranlık: Stage mid-tone, backplanes */
  --lf-violet: #695FA4;  /* Menekşe: Primary buttons, active dock, highlights */
  --lf-lilac:  #B7ADDF;  /* Lila: Elephant silhouette, focus rings, specular rims */
  --lf-mist:   #D4CCF1;  /* Sis: Secondary labels, micro-telemetry, highlights */
  --lf-white:  #FFFBFF;  /* Kar Beyazı: High-contrast headings and active text */

  /* Semantic Health Tokens (Strictly Isolated from Brand Colors) */
  --health-healthy: #10B981;  /* Emerald: Verified running process */
  --health-warning: #F59E0B;  /* Amber: Indeterminate or warning status */
  --health-error:   #EF4444;  /* Ruby: Port conflict, crash, destructive lock */
  --health-unknown: #6B7280;  /* Slate: Stopped project, unprobed service */

  /* Typography */
  --font-display: 'Plus Jakarta Sans', -apple-system, sans-serif;
  --font-body:    'Figtree', -apple-system, sans-serif;
  --font-mono:    'JetBrains Mono', Menlo, monospace;
}
```

---

## 3. Visual Deliverables Audit (The Six Corrected Screens)

All six prototypes are delivered in `docs/design/r1_3b/brand_aligned/` with verified 1600×1000 PNG renderings:

| Screen | HTML Prototype | Rendered 1600×1000 PNG | Rendered Assets | Verified Composition |
| :--- | :--- | :--- | :--- | :--- |
| **01. Spatial Studio Overview** | [`spatial_studio_overview.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/spatial_studio_overview.html) | [`spatial_studio_overview.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/spatial_studio_overview.png) (385 KB) | Elephant emblem badge (100% loaded) | Direction A+ tall vertical monoliths, perspective floor, reflections, left dock, command capsule |
| **02. Projects Workspace** | [`projects_workspace.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/projects_workspace.html) | [`projects_workspace.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/projects_workspace.png) (152 KB) | Elephant emblem badge (100% loaded) | Filter pills, search bar, structured project cards, primary action buttons |
| **03. Services & Runtime** | [`services_runtime.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/services_runtime.html) | [`services_runtime.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/services_runtime.png) (281 KB) | Elephant emblem badge (100% loaded) | Nginx port 8080 ownership, PHP-FPM socket pool, MariaDB engine, ownership verification panel |
| **04. Storage & Volumes** | [`storage_volumes.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/storage_volumes.html) | [`storage_volumes.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/storage_volumes.png) (223 KB) | Elephant emblem badge (100% loaded) | Tier-1 external APFS volume gauge, persistent UUIDs, safe disconnect policy card |
| **05. Project Inspector** | [`project_inspector.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/project_inspector.html) | [`project_inspector.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/project_inspector.png) (207 KB) | Elephant emblem badge in header & drawer | Direction A+ stage visible behind drawer, disabled DB delete lock, Dry-Run Only SFTP deployment |
| **06. Welcome / Splash** | [`welcome_splash.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/welcome_splash.html) | [`welcome_splash.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/welcome_splash.png) (173 KB) | Official 1024×1024 app icon (100% loaded) | Apple squircle app icon, Plus Jakarta Sans headings, 3 onboarding action cards |

---

## 4. Visual Comparison with Direction A+ Baseline

| Compositional Dimension | Approved Direction A+ Baseline | Stage A Visual Revision | Compliance Status |
| :--- | :--- | :--- | :--- |
| **Project Presentation** | Tall vertical translucent monoliths (`260×330px`) | Restored tall vertical translucent monoliths with specular rim (`--lf-lilac`) | **IDENTICAL & PRESERVED** |
| **Stage Geometry** | Centered stage, perspective floor (`rotateX(65deg)`), radial floor reflection | Restored perspective floor, floor reflection, and deep atmospheric backdrop (`#191337`) | **IDENTICAL & PRESERVED** |
| **Negative Space** | Generous breathing room, centered hero headline | Restored centered headline ("Tüm servisler etkin.") with Plus Jakarta Sans typography | **ENHANCED & PRESERVED** |
| **Navigation Elements** | Floating left dock, right count switcher (0, 2, 6) | Restored floating left dock in glass (`--lf-plum`), right zoom switcher | **IDENTICAL & PRESERVED** |
| **Command Center** | Bottom floating capsule with AI status orb and Mac workspace breadcrumb | Restored bottom floating capsule with lilac pulsing orb and `/Volumes/Mac Harici Disk` pill | **IDENTICAL & PRESERVED** |
| **Brand Identity** | Plain text / generic placeholder | Official LocalForge elephant emblem + Plus Jakarta Sans brand lockup | **OFFICIAL BRAND ALIGNED** |
| **Health Signals** | Emerald / Green | Strictly Emerald Green (`#10B981`) — Never purple/violet | **STRICTLY ENFORCED** |

---

## 5. Self-Contained Review Package

The complete asset package is bundled as a single offline archive:
- **Location**: `docs/design/r1_3b/brand_aligned/localforge_brand_assets.zip`
- **File Size**: ~6.67 MB
- **Contents**:
  - `assets/localforge-emblem.png` (Canonical elephant emblem)
  - `assets/localforge-logo.png` (Canonical brand lockup)
  - `assets/localforge-app-icon.png` (Canonical 1024×1024 master app icon)
  - `tokens/tokens.json` (W3C standard token specification)
  - `tokens/tokens.css` (CSS custom properties)

---

**LOCALFORGE BRAND INTEGRATION STAGE A — AWAITING FINAL VISUAL ACCEPTANCE.**
