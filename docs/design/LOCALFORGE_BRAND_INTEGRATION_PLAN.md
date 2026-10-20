# LOCALFORGE BRAND INTEGRATION PLAN & DESIGN SPECIFICATION (STAGE A)

> **Document Status**: Authoritative Design Proposal — Stage A Deliverable  
> **Target Release**: LocalForge R1.3B  
> **Brand Kit Source**: `LocalForge-Logo-Kiti.zip` (Version 1.0, October 2026)  
> **Scope**: Design Specification, Token Mapping, Visual Prototypes, macOS App Icon Guidelines, and Asset Architecture  
> **Execution Gate**: Implementation in production application code (`apps/desktop/src`) is **strictly deferred** to Stage B.

---

## 1. Executive Summary & Brand Alignment

The official LocalForge brand kit (`LocalForge-Logo-Kiti.zip`) establishes a distinctive, memorable, and developer-centric visual identity centered around the **lila elephant emblem** framed within an inverted rounded hexagon (shield). 

This integration plan maps the official brand tokens into the approved **Direction A+ Spatial Studio** design system while strictly maintaining:
1. **Direction A+ Spatial Composition**: 3D longitudinal perspective cards, floating left dock, centered macOS stage, and bottom command capsule.
2. **Strict Semantic Health Isolation**: System health indicators (**Emerald Green** for healthy, **Amber** for warnings, **Ruby Red** for errors/conflicts, **Neutral Slate** for stopped/unknown) are **never** colored purple, violet, or lilac.
3. **Data Capability & Safety Integrity**: Zero synthetic vanity metrics, disabled/guarded database destructive operations, and dry-run only SFTP deployment.
4. **macOS Native Craft**: First-class Apple Silicon arm64 typography, material blurs, and Apple squircle geometry.

---

## 2. Canonical Brand Asset Architecture

The repository asset tree has been standardized under `assets/brand/` to ensure full traceability and maintain a clean separation between upstream read-only source files, optimized raster assets, and design tokens:

```
assets/
└── brand/
    ├── original/                                  # Upstream read-only source of truth
    │   └── LocalForge-Logo-Kiti/
    │       ├── LocalForge-Logo-Kiti.html          # Canonical brand guideline & interactive guide
    │       └── logolar/
    │           ├── localforge-app-icon.png       # 2048×2048 master app icon
    │           ├── localforge-emblem.png         # 1300×1300 isolated elephant emblem
    │           └── localforge-logo.png           # 2048×2048 brand lockup (transparent)
    ├── raster/                                    # Normalized production-ready raster assets
    │   ├── localforge-app-icon-2048.png          # 2048×2048 master
    │   ├── localforge-app-icon-1024.png          # 1024×1024 macOS master icon
    │   ├── localforge-app-icon-512.png           # 512×512 icon asset
    │   ├── localforge-emblem.png                 # Isolated brand emblem for headers and badges
    │   └── localforge-logo.png                   # Horizontal wordmark lockup
    ├── tokens/                                    # Machine-readable design tokens
    │   ├── tokens.json                           # W3C Design Tokens Community Group specification
    │   └── tokens.css                            # CSS Custom Properties for web and desktop UI
    └── vector/                                    # Reserved for future vector exports (SVG / PDF)
        └── .gitkeep
```

---

## 3. Canonical Design Tokens & Color Mapping

### 3.1 Official Brand Color Palette

| Token Name | Color Name | Hex Code | RGB | Contrast on Dark | Usage in LocalForge Spatial Studio |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `--lf-night` | Gece | `#191337` | `25, 19, 55` | 1.15:1 (Base) | Canvas base, deepest atmospheric window shadows |
| `--lf-plum` | Erik | `#2B2450` | `43, 36, 80` | 2.05:1 | Spatial glass surfaces, cards, and inspector panels |
| `--lf-dusk` | Alacakaranlık | `#3B376A` | `59, 55, 106` | 3.20:1 | Mid-tone stage backdrop, card backplanes, borders |
| `--lf-violet` | Menekşe | `#695FA4` | `105, 95, 164` | 4.80:1 (AA) | Primary interactive actions, active navigation, focus |
| `--lf-lilac` | Lila | `#B7ADDF` | `183, 173, 223` | 9.45:1 (AAA) | Elephant emblem, hero brand color, specular borders |
| `--lf-mist` | Sis | `#D4CCF1` | `212, 204, 241` | 12.10:1 (AAA) | Secondary labels, micro-telemetry, subtle highlights |
| `--lf-white` | Kar Beyazı | `#FFFBFF` | `255, 251, 255` | 17.50:1 (AAA) | High-contrast primary headings and active text |

### 3.2 Semantic System Health Tokens (Strictly Isolated)

To prevent cognitive ambiguity between brand identity and operational runtime safety, health signals are strictly isolated from the purple palette:

| Health State | Token | Hex | Role | Rule |
| :--- | :--- | :--- | :--- | :--- |
| **Healthy / Running** | `--lf-health-healthy` | `#10B981` | Emerald Green | Process PID verified, port bound, probe responding |
| **Warning / Degraded** | `--lf-health-warning` | `#F59E0B` | Amber Orange | Indeterminate probes, unprobed runtimes, high load |
| **Error / Port Conflict** | `--lf-health-error` | `#EF4444` | Ruby Red | Port collision, crashed process, safety violation |
| **Stopped / Neutral** | `--lf-health-unknown` | `#6B7280` | Neutral Slate | Dormant project, unstarted runtime, pending setup |

> [!CAUTION]
> **Brand-Health Isolation Rule**: The brand lilac (`#B7ADDF`) or violet (`#695FA4`) MUST NEVER be used to indicate healthy runtime status. Green is exclusively reserved for verified live processes.

### 3.3 Typography Hierarchy

| Role | Font Family | Weight | Tracking | Usage |
| :--- | :--- | :--- | :--- | :--- |
| **Display / Headings** | `Plus Jakarta Sans` | 700 / 800 | `-0.02em` to `-0.03em` | Window titles, card names, hero headlines, modal titles |
| **Body / Controls** | `Figtree` | 400 / 500 / 600 | `-0.01em` | Explanations, button labels, descriptions, navigation |
| **Telemetry / Code** | `JetBrains Mono` | 400 / 500 / 600 | `+0.04em` | PIDs, ports, paths, hashes, UUIDs, terminal logs |

---

## 4. macOS App Icon Specification

The official master asset `localforge-app-icon.png` (2048×2048 px) has been inspected and downscaled to a native 1024×1024 master icon (`assets/brand/raster/localforge-app-icon-1024.png`).

### 4.1 Apple Human Interface Guidelines Compliance
- **Canvas Size**: 1024 × 1024 px.
- **Artwork Safe Area Grid**: Center 824 × 824 px zone. The geometric elephant emblem and hexagon rim sit completely within the safe area with generous optical margins (100px padding around bounding box).
- **Apple Squircle Mask**: Continuous curvature radius of ~22.37% (229px radius on 1024px canvas). The elephant trunk curl is safely contained inside the curved corner boundary.
- **Dock & Menu Bar Contrast**: The high-contrast lilac silhouette (`#B7ADDF`) against the deep dusk/plum gradient ensures high visibility across both light and dark macOS desktop backgrounds.

### 4.2 Brand Rule Adherence
- **No Watermarks / Low Opacity**: The brand guideline explicitly prohibits fading the emblem as a watermark. All app icons and headers use crisp, 100% opaque emblems.
- **No Color Alterations**: The elephant is preserved in its authentic lilac hue, avoiding common blue (PostgreSQL) or yellow/green developer tool tropes.

### 4.3 Stage B Tauri Generation Pipeline
When Stage B is authorized, the desktop icon bundle will be generated via Tauri tooling:
```bash
# Executed during Stage B implementation
npx @tauri-apps/cli icon assets/brand/raster/localforge-app-icon-1024.png --output apps/desktop/src-tauri/icons
```
Target generated files:
- `icon.icns` (macOS 16x16 up to 512x512@2x)
- `icon.ico` (Windows multi-resolution)
- `128x128.png`, `128x128@2x.png`, `Square310x310Logo.png`

---

## 5. Visual Integration Proposals (The 6 Screens)

All 6 screens have been designed and rendered to high-fidelity 1600×1000 PNG screenshots under `docs/design/r1_3b/brand_aligned/`.

### 5.1 Screen 01 — Spatial Studio Overview
- **HTML**: [`docs/design/r1_3b/brand_aligned/spatial_studio_overview.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/spatial_studio_overview.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/spatial_studio_overview.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/spatial_studio_overview.png)
- **Design Highlights**:
  - Unified titlebar featuring the official elephant emblem and `LocalForge` typography in `Plus Jakarta Sans`.
  - Direction A+ 3D longitudinal card perspective with subtle `7deg` X-axis tilt and ambient depth.
  - Specular top borders using `--lf-lilac` (`rgba(183, 173, 223, 0.4)`).
  - Floating left navigation dock with active button in `--lf-violet`.
  - Bottom command capsule with `⌘K` shortcut and persistent APFS volume indicator.
  - Active project status shown in verified Emerald Green (`#10B981`); stopped project shown in neutral slate.

### 5.2 Screen 02 — Projects Workspace
- **HTML**: [`docs/design/r1_3b/brand_aligned/projects_workspace.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/projects_workspace.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/projects_workspace.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/projects_workspace.png)
- **Design Highlights**:
  - Filter capsule bar with brand-colored active filter ("Tümü (2)") and search input (`⌘F`).
  - Project cards with clear information architecture: root path, local domain, PHP-FPM runtime version, MariaDB database name, and APFS persistent volume UUID.
  - Distinct primary action button in `--lf-violet` with lilac border highlight.
  - Completely devoid of synthetic vanity metrics.

### 5.3 Screen 03 — Services & Runtime
- **HTML**: [`docs/design/r1_3b/brand_aligned/services_runtime.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/services_runtime.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/services_runtime.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/services_runtime.png)
- **Design Highlights**:
  - 3 dedicated runtime cards: Nginx Web Gateway, PHP-FPM Pool, and MariaDB Database Engine.
  - Telemetry rendered in `JetBrains Mono`: port `8080`, verified Master PID `48921`, config path `~/.localforge/nginx/nginx.conf`.
  - Adversarial security panel highlighting **Process Ownership Verification**: arbitrary Homebrew Nginx instances are strictly flagged as unverified, preventing false-positive healthy reports.

### 5.4 Screen 04 — Storage & External Volumes
- **HTML**: [`docs/design/r1_3b/brand_aligned/storage_volumes.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/storage_volumes.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/storage_volumes.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/storage_volumes.png)
- **Design Highlights**:
  - Focuses on Tier-1 external APFS storage: `/Volumes/Mac Harici Disk`.
  - Capacity progress gauge styled with brand gradient (`--lf-violet` to `--lf-lilac`).
  - Persistent volume UUID display (`E4B30F82-9C1D-4E7B-8F09-2D88C523B0A1`).
  - Safe Disconnect Policy banner ensuring zero accidental file recreation on internal disk.

### 5.5 Screen 05 — Detailed Project Inspector
- **HTML**: [`docs/design/r1_3b/brand_aligned/project_inspector.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/project_inspector.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/project_inspector.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/project_inspector.html)
- **Design Highlights**:
  - Slide-over glass drawer (width 540px) pinned to right with deep blur (`--lf-plum` with lilac border).
  - Drawer header displays official elephant emblem badge and project identity.
  - **Data Safety Guard**: "Veritabanını Sıfırla / Sil" button is strictly **disabled and locked** with an explicit red cautionary banner.
  - **SFTP Deployment Guard**: Remote target strictly tagged **"Dry-Run Only"**, with live server write actions disabled.

### 5.6 Screen 06 — Application Welcome / Splash
- **HTML**: [`docs/design/r1_3b/brand_aligned/welcome_splash.html`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/welcome_splash.html)
- **Screenshot**: [`docs/design/r1_3b/brand_aligned/welcome_splash.png`](file:///Volumes/Mac%20Harici%20Disk/VibeProject/LocalForge/docs/design/r1_3b/brand_aligned/welcome_splash.png)
- **Design Highlights**:
  - First-run experience featuring the official 1024×1024 macOS app icon in Apple squircle mask with deep drop shadow.
  - Clear, welcoming typography in `Plus Jakarta Sans` and `Figtree`.
  - 3 primary onboarding actions: "Mevcut Projeyi Aç", "Yeni Proje Başlat" (primary brand button), and "Harici Disk Tanımla".
  - Clean footer badge: `v0.1.0-alpha • Apple Silicon (arm64) • macOS Tier-1 • Açık Kaynak`.

---

## 6. Before / After Visual Comparison

| Area | Previous Baseline (Early R1.3B) | Brand-Aligned Integration (Stage A) | Improvement Rationale |
| :--- | :--- | :--- | :--- |
| **Color Atmosphere** | Generic cold dark slate (`#0A0D14`, `#121826`) with generic blue tint | Rich atmospheric night purple (`#191337`, `#2B2450`, `#3B376A`) | Directly establishes LocalForge identity from official brand kit |
| **Window Header** | Plain text title with generic dots | Official LocalForge elephant emblem + wordmark in Plus Jakarta Sans | Instant product recognition and premium macOS native feel |
| **Active States** | Generic indigo/blue buttons | Signature violet (`#695FA4`) with lilac borders (`#B7ADDF`) | Cohesive brand touchpoints across controls, pills, and docks |
| **Health Distinction** | Health indicators occasionally blended with blue accents | Green strictly isolated to `#10B981` (Emerald); brand purple never used for health | Zero cognitive ambiguity between brand theme and critical alerts |
| **Typography** | Generic system font | Plus Jakarta Sans (Display) + Figtree (Body) + JetBrains Mono (Data) | High typographical personality aligned with developer ergonomics |
| **App Icon** | Tauri default placeholder / generic icon | Official geometric elephant squircle master (1024×1024) | Ready for native macOS bundle integration in Stage B |

---

## 7. Data Capability & Safety Guard Verification

Every design element across the 6 proposals strictly conforms to real backend capabilities:

1. **No Synthetic Telemetry**: No fictional CPU/memory charts or millisecond latency graphs are rendered. All displayed metrics (PID, ports, config paths, UUIDs) match real system outputs.
2. **Nginx Process Ownership**: Telemetry explicitly details the LocalForge config flag (`-c ~/.localforge/nginx/nginx.conf`) and verified listening port `8080`, rejecting foreign processes.
3. **Destructive Guardrails**: In `project_inspector.html`, the database delete button is statically locked with an explanatory note confirming that destructive mutations are forbidden.
4. **SFTP Deployment Simulation**: In `project_inspector.html`, deployment is restricted to `Dry-Run Only`, preventing unauthorized network mutations.

---

## 8. Stage B Implementation Checklist (Upon Authorization)

When Product Owner approval for Stage A is granted, Stage B will follow this exact execution sequence:

- [ ] **Step 01**: Import `assets/brand/tokens/tokens.css` into `apps/desktop/src/styles/` or Tailwind configuration.
- [ ] **Step 02**: Update `StudioOverview.tsx` titlebar with the official emblem lockup and Plus Jakarta Sans font.
- [ ] **Step 03**: Update navigation components with `--lf-violet` active states and `--lf-lilac` specular borders.
- [ ] **Step 04**: Replace desktop application icon bundle in `src-tauri/icons/` using `assets/brand/raster/localforge-app-icon-1024.png`.
- [ ] **Step 05**: Implement the 6 updated screens according to the approved HTML prototypes.
- [ ] **Step 06**: Execute automated verification test suite (`cargo test --workspace` and `npm test`).
- [ ] **Step 07**: Push implementation branch to origin and verify GitHub Actions CI.

---

**LOCALFORGE BRAND INTEGRATION — STAGE A AWAITING PRODUCT OWNER APPROVAL.**
