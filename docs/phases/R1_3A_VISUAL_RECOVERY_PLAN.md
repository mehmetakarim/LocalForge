# LocalForge — Phase R1.3A-V: Spatial Studio Visual Recovery Plan
## Stage A Design Proposal & Side-by-Side Architectural Specification

**Document Version**: 1.0.0  
**Phase**: Recovery Phase R1.3A-V (Spatial Studio Visual Recovery)  
**Target Milestone**: Stage A — Visual Design Proposal & PO Approval Gate  
**Authors**: Lead Architect, Principal macOS Product Designer, Senior Tauri 2 / React Engineer  
**Status**: PROPOSED — AWAITING PRODUCT OWNER APPROVAL  
**Parent Commit**: `8a1c7a9`  
**Active Branch**: `phase-r1/r1-3a-spatial-studio-foundation`  

---

## 1. Detailed Failure Analysis of Phase R1.3A Baseline

Following runtime verification of the compiled desktop application on macOS Apple Silicon (`docs/screenshots/spatial_studio_final.png`), the Product Owner issued a formal visual failure verdict. While R1.3A successfully verified real process supervision and compiled without errors, the interface fundamentally missed the approved **Spatial Studio** visual language.

### Root Causes of Visual Failure:

1. **Conventional Left-Aligned "Admin Dashboard" Layout**:
   - The layout utilized a standard web dashboard grid (`flex-col` + left-aligned `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`), crowding all content against the left navigation margin (`ml-24`).
   - The stage lacked a defined center of visual gravity, focal depth, or intentional negative space, feeling like a responsive web app crammed inside a desktop frame.

2. **Noisy, Fragmented Macro Telemetry**:
   - The header crammed badges, titles, subtitles, refresh buttons, and action buttons across standard web flex rows.
   - The "Infrastructure Telemetry Ribbon" introduced a bulky rectangular container with contrasting pills (`Nginx Gateway`, `PHP-FPM Pools`, `MariaDB 3306`) that broke visual calm and drew excessive attention away from projects.
   - Filter pills (`All Workspaces`, `Running`, `External SSD`) and the project search input were rendered in harsh rectangular boxes with prominent borders (`border-white/10`), adding horizontal clutter.

3. **Flat, Generic Card Anatomy**:
   - The `SpatialCard` implementation looked like a typical SaaS dashboard card with dark background and rounded corners, rather than a tangible 2.5D physical spatial object.
   - Information was packed into dense key-value rows (`PHP Runtime`, `Local Gateway`, `Document Root`, `Port`) using generic table-like styling, instead of a sculpted physical device aesthetic with layered glass depth.

4. **Weak Atmospheric Lighting & Depth Illusion**:
   - The background lighting halos were placed off-screen or rendered too dimly (`opacity-40` with harsh circular bounds), failing to illuminate the stage beneath active workspaces.
   - Specular rim highlights were rendered as flat uniform borders rather than authentic directional light refraction (top-edge specular highlight fading into soft shadow).

5. **Critical Data Integrity Violation (Unacceptable Mock Fallback)**:
   - When no projects were registered, `App.tsx` automatically seeded synthetic projects (`StepperSkip Store` and `LocalForge Docs & API`) directly into production state.
   - When `get_runtime_overview` encountered an error, a synthetic mock object hardcoded `mariadb_alive: true` and `nginx_running: true`.
   - In `StudioOverview.tsx`, PHP-FPM status was hardcoded as `status="running"`.
   - These fallbacks violated the core engineering rule: **LocalForge must never fabricate telemetry or project state in production paths**.

---

## 2. Reference-to-Implementation Visual Gap Analysis

| Visual Dimension | Approved Spatial Studio Reference Vision | R1.3A Baseline Implementation | R1.3A-V Proposed Recovery Specification |
|---|---|---|---|
| **Spatial Composition** | Centered, tranquil stage. Expansive negative space framing cards as floating physical objects. Balanced visual symmetry. | Left-heavy, asymmetric web dashboard layout with cards crowded against the dock margin. | Centered stage canvas with golden-ratio margins, floating hero title, and balanced spatial card clusters. |
| **Visual Hierarchy** | Calm macro status heading with integrated telemetry. Clear focal hierarchy leading naturally to project objects. | Cluttered top header with multiple competing pill badges, a dense telemetry ribbon, and separate filter rows. | Unified Atmospheric Hero Header seamlessly integrating system telemetry into quiet ambient typography. |
| **Lighting & Depth** | Deep void (`#05070E`) with soft, diffuse twilight indigo/ultramarine lighting halos resting beneath active workspace objects. | Dark gray-blue canvas with flat borders and disconnected background glow blobs. | Sculpted lighting field with dual hardware-capped radial gradients, 1px top specular rims, and soft multi-stop diffuse shadows. |
| **Material Tactility** | Multi-layered liquid glass with authentic refraction, subtle specular rim highlights, and physical micro-elevation. | Simple translucent boxes with uniform 1px gray borders (`border-white/10`). | Dual-stop specular borders (`rgba(255,255,255,0.22)` top rim fading to `rgba(255,255,255,0.04)`), frosted smoked glass backing. |
| **Atmosphere & Emotion** | Calm mastery, silent power, tactile precision. Evokes next-generation Apple Pro audio/spatial hardware. | Busy, bright, noisy web admin panel reminiscent of generic Bootstrap or modern SaaS dashboards. | Quiet, distraction-free environment. Visual calm allowing developer focus, with subtle vitality pulses only on active daemons. |
| **Data Integrity** | Absolute honesty: unverified states shown as `UNKNOWN` or `UNAVAILABLE`; zero synthetic mocks in production. | Injected mock projects when empty; hardcoded PHP-FPM as `RUNNING`; fallback fabricated MariaDB latency. | Strict zero-fabrication boundary: genuine empty state, honest runtime status, clear error boundaries. |

---

## 3. New Centered Stage Composition

```
+---------------------------------------------------------------------------------------------------+
|  (•)(•)(•) LocalForge  [ Window Drag Region ]                                     v0.1.0 (Phase R1) |
|                                                                                                   |
|  [ Floating Dock ]               +---------------------------------------------+                  |
|  +--------------+                |         ATMOSPHERIC HERO HEADER             |                  |
|  | (•) Studio   |                |                                             |                  |
|  | (•) Projects |                |              Spatial Studio                 |                  |
|  | (•) Services |                |   Apple Silicon Loopback  •  3 Daemons Live  |                  |
|  | (•) Storage  |                +---------------------------------------------+                  |
|  | (•) Deploy   |                                                                                 |
|  | (•) AI / MCP |                     [ Ambient Indigo Glow Halo ]                                |
|  |              |                                                                                 |
|  | (•) Settings |         +-------------------------+     +-------------------------+             |
|  +--------------+         |   SPATIAL CARD OBJECT   |     |   SPATIAL CARD OBJECT   |             |
|                           |   ● RUNNING             |     |   ○ STOPPED             |             |
|                           |   StepperSkip Store     |     |   Client Portal API     |             |
|                           |   PHP 8.4  •  SSD Ext   |     |   PHP 8.3  •  SSD Int   |             |
|                           |   stepperskip.test:8080 |     |   portal.test:8080      |             |
|                           |   [ Open ↗ ] [ Stop ■ ] |     |   [ Start ▶ ] [ cURL ]  |             |
|                           +-------------------------+     +-------------------------+             |
|                                                                                                   |
|                                   [ Genuine Empty State if 0 Projects ]                           |
|                                                                                                   |
|                                [ Floating AI Command Capsule (⌘K) ]                               |
+---------------------------------------------------------------------------------------------------+
```

### 3.1 Structural Composition Rules:
1. **Centered Stage Alignment**:
   - The stage container will center dynamically within the viewport (`max-w-5xl mx-auto px-6`), balancing the visual weight of the left floating dock.
   - Ample breathing room: Minimum 48px top clearance below the macOS drag region, 40px stage padding, and 80px bottom clearance above the command capsule.

2. **Unified Atmospheric Hero Header**:
   - Replaces the cluttered multi-row headers and telemetry ribbon with a clean, cohesive focal anchor:
     - Title: `Spatial Studio` in 30px semi-bold SF Pro Display with subtle top-to-bottom specular text gradient.
     - Live Macro Telemetry Line: Subdued monospace text integrating verified platform, loopback IP (`127.0.0.1`), and service pips (Nginx, PHP-FPM, MariaDB) into a single horizontal phrase with micro status indicators.
     - Action Controls: Minimalist, glass-morphic `Refresh` and `+ New Project` buttons that blend harmoniously with the stage.

3. **Stage Grid & Dynamic Spacing**:
   - Projects display in a centered, well-proportioned 2-column or 3-column spatial object grid (`gap-6`).
   - Cards are constrained to optimal proportions (`min-w-[340px] max-w-[440px]`), ensuring dense technical data reads with high clarity.

4. **Truthful Empty State Stage**:
   - When zero projects are registered, the stage displays a spacious, elegant empty canvas featuring a glowing spatial blueprint wireframe, clear onboarding instructions, and a single prominent `Create First Project` action.

---

## 4. Component Redesign Strategy

### 4.1 `SpatialCard`: The Physical 2.5D Spatial Object
- **Material & Volume**:
  - Background: Deep frosted smoked acrylic (`rgba(13, 18, 30, 0.72)`) with `backdrop-filter: blur(24px) saturate(160%)`.
  - Specular Rim: 1px directional gradient border (`border-image: linear-gradient(135deg, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.03) 50%, rgba(79,110,247,0.15) 100%)`).
  - Elevation & Ambient Glow: Layered box shadow combining deep contact shadow (`0 20px 40px -12px rgba(0,0,0,0.6)`) with subtle indigo back-lighting (`0 0 32px -4px rgba(79,110,247,0.12)`).
- **Physical Header**:
  - Live Triple-Channel Status Badge: Glowing emerald pip + status text (`RUNNING`) + activity pulse.
  - Storage Badge: Physical disk icon with distinctive amber/gold boundary for external APFS drives (`Mac Harici Disk`).
- **Sculpted Metrics Body**:
  - High-contrast monospace typography for local domain (`project.test`) and port (`:8080`).
  - Tactile cURL copy button with inline visual feedback.
  - Runtime badge indicating verified PHP version (`v8.4 FPM`).
- **Action Deck**:
  - Primary Action: Glowing glass button for `Start` (indigo glow) or `Stop` (muted crimson rim).
  - Secondary Action: `Open Browser ↗` with direct navigation to `http://127.0.0.1:8080` (with Host header note) or local browser opener.

### 4.2 `FloatingDock`: The Liquid Glass Navigation Spine
- **Positioning**:
  - Detached floating vertical capsule suspended at `left: 24px`, vertically centered on the viewport.
  - Uncoupled from stage scrolling to remain a persistent anchor.
- **Glass Specular Finish**:
  - Surface: `rgba(10, 14, 24, 0.75)` with `backdrop-filter: blur(32px) saturate(180%)`.
  - Border: 1px specular highlight (`rgba(255, 255, 255, 0.16)` top edge, `rgba(0,0,0,0.4)` bottom edge).
- **Spring Active Indicator**:
  - Subtly glowing liquid pill behind the active route icon (`background: rgba(79, 110, 247, 0.22)`, `border: 1px solid rgba(79, 110, 247, 0.45)`).
  - Smooth spring transition (`transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1)`).
- **Tooltips**:
  - Frosted tooltips appearing on hover with micro hotkeys (`⌘1` .. `⌘7`).

### 4.3 `AICommandCapsule`: The Floating Command Center
- **Idle Placement**:
  - Centered horizontally at `bottom: 24px`, width `480px`, height `42px`.
  - Floating above stage content with subtle indigo aura (`box-shadow: 0 16px 48px rgba(0,0,0,0.6), 0 0 20px rgba(79,110,247,0.2)`).
- **Interactive State**:
  - Clicking or pressing `⌘K` fluidly expands into a centered modal dialog (`width: 640px`) with blur backdrop overlay.
  - Autofocus search input querying projects, local domains, services, and MCP tools.
  - Honest status query handling: queries actual backend state rather than mocking answers.

### 4.4 `ServiceHealthPill`: Uncluttered Triple-Channel Telemetry
- Replaces the bulky rectangular ribbon with an elegant, integrated macro status bar:
  - Each service (Nginx 8080, PHP-FPM, MariaDB 3306) is represented with:
    1. Semantic color pip (Emerald for live, Slate for stopped, Amber for degraded).
    2. Specific icon glyph (`Server`, `Cpu`, `Database`).
    3. Explicit text label with port/socket info.
  - Clicking opens an honest diagnostic popover displaying verified PID, socket path, and round-trip ping.

---

## 5. Design Token & CSS Architecture Changes

### 5.1 Updated CSS Custom Properties (`apps/desktop/src/index.css`)
```css
:root {
  /* Atmospheric Void & Stage Canvas */
  --lf-canvas-void: #05070E;
  --lf-canvas-stage: #080C16;
  --lf-canvas-panel: rgba(13, 18, 30, 0.72);
  --lf-canvas-dock: rgba(10, 14, 24, 0.75);
  --lf-canvas-capsule: rgba(15, 20, 35, 0.82);

  /* Directional Specular Rims & Borders */
  --lf-specular-top: rgba(255, 255, 255, 0.22);
  --lf-specular-mid: rgba(255, 255, 255, 0.08);
  --lf-specular-bottom: rgba(0, 0, 0, 0.40);
  --lf-border-glass: rgba(255, 255, 255, 0.09);
  --lf-border-active: rgba(79, 110, 247, 0.50);

  /* Atmospheric Lighting Gradients */
  --lf-halo-indigo: rgba(79, 110, 247, 0.16);
  --lf-halo-ultramarine: rgba(30, 27, 75, 0.28);
  --lf-halo-cyan: rgba(56, 189, 248, 0.08);

  /* Semantic Status Tokens (Triple-Channel AA Compliant) */
  --lf-status-running: #10B981;
  --lf-status-running-glow: rgba(16, 185, 129, 0.25);
  --lf-status-stopped: #6B7280;
  --lf-status-starting: #3B82F6;
  --lf-status-degraded: #F59E0B;
  --lf-status-failed: #EF4444;
  --lf-status-unknown: #8B5CF6;
  --lf-status-unavailable: #4B5563;

  /* Typography Scale */
  --lf-font-display: "SF Pro Display", -apple-system, BlinkMacSystemFont, sans-serif;
  --lf-font-text: "SF Pro Text", -apple-system, BlinkMacSystemFont, sans-serif;
  --lf-font-mono: "SF Mono", Menlo, Monaco, Consolas, monospace;
}
```

### 5.2 Controlled Hardware-Capped Blur Layers
To guarantee steady 60fps/120Hz ProMotion performance on Apple Silicon while preventing GPU memory thrashing:
- The entire application will use **strictly two** hardware-accelerated ambient blur layers:
  1. `Halo Primary`: Positioned center-top behind the active stage (`w-[720px] h-[520px] blur-[120px] opacity-35`).
  2. `Halo Secondary`: Positioned bottom-right (`w-[600px] h-[440px] blur-[100px] opacity-25`).
- No card or child component will introduce nested radial gradient blur filters. Card materials will use standard CSS `backdrop-filter: blur(20px)`.

---

## 6. Data Integrity Corrections (Zero-Fabrication Mandate)

### 6.1 Elimination of Mock Fallbacks in `App.tsx`
- **Current Defect**:
  Lines 52–86 of `apps/desktop/src/App.tsx` automatically injected synthetic `StepperSkip Store` and `LocalForge Docs & API` projects whenever `sync_projects` returned an empty array.
- **Remediation**:
  1. Remove all mock injection logic from the Tauri production execution path.
  2. When `sync_projects` returns `[]`, `projects` state must remain `[]`.
  3. The UI will render the genuine, accessible `EmptyState` component with an actionable `Create New Project` trigger.

### 6.2 Truthful Runtime Overview & Error Reporting
- **Current Defect**:
  Lines 39–50 of `App.tsx` caught errors from `get_runtime_overview` and fabricated `nginx_running: true`, `mariadb_alive: true`, and `mariadb_latency_ms: 0.4`.
- **Remediation**:
  1. If `get_runtime_overview` fails, set `runtime` state to `null` and record an honest diagnostic error message.
  2. `ServiceHealthPill` components must render `UNKNOWN` or `UNAVAILABLE` with a gray/violet pip and an alert glyph, clearly indicating that daemon status has not been verified.
  3. Never display synthetic latency numbers or fabricated socket paths.

### 6.3 Eliminating Hardcoded PHP-FPM Status in `StudioOverview.tsx`
- **Current Defect**:
  Line 120 of `StudioOverview.tsx` hardcoded `status="running"` for `PHP-FPM Pools`.
- **Remediation**:
  1. Evaluate PHP-FPM status based on actual installed versions discovered by `get_runtime_overview`:
     - If `runtime?.php_versions` is non-empty and supervisor reports active pools, display `RUNNING` or `READY`.
     - If no versions are installed or supervisor probe fails, display `STOPPED` or `UNAVAILABLE`.
     - If runtime overview failed, display `UNKNOWN`.

### 6.4 Regression Test Specifications
- Add an automated TypeScript test / assertion to ensure:
  - An empty project response produces an empty list, never fabricated records.
  - A runtime API error produces `UNKNOWN` or `UNAVAILABLE` status tokens across all service indicators.
  - Production builds (`npm run build`) pass strict TypeScript validation with zero loose `any` casts.

---

## 7. Performance & Accessibility Considerations

1. **Hardware-Capped GPU Budget**:
   - Total active blur layers capped at 2 in `AppShell`.
   - Card micro-tilt utilizes GPU-accelerated CSS `transform: perspective(1000px) rotateX(...) rotateY(...)` with `will-change: transform`.
   - Micro-tilt is completely bypassed when `prefers-reduced-motion: reduce` is detected.

2. **ProMotion 120Hz & 60fps Target**:
   - Zero heavy JavaScript physics loops. All animations use native CSS transitions with cubic-bezier easing (`cubic-bezier(0.16, 1, 0.3, 1)`).
   - Zero WebGL or Three.js runtime overhead.

3. **WCAG 2.2 AA Compliance**:
   - Contrast ratio for primary text (`--lf-text-primary: rgba(255,255,255,0.95)`) over stage canvas exceeds `14:1` (AAA).
   - Contrast ratio for secondary monospace telemetry exceeds `7:1` (AAA).
   - Triple-channel status signaling: Every status state combines color, distinctive iconography, and explicit text labels.

4. **Keyboard & Screen Reader Ergonomics**:
   - Full keyboard navigation: `⌘1`..`⌘7` for module navigation, `⌘K` for AI Command Capsule, `Esc` to dismiss modals.
   - All interactive controls have explicit `aria-label`, `role="tab"`, and visible focus rings (`focus-visible:ring-2 focus-visible:ring-indigo-500`).

---

## 8. Visual Acceptance Checklist & Side-by-Side Comparison

### 8.1 Side-by-Side Comparative Evaluation

| Feature / Area | Approved Spatial Studio Reference Vision | R1.3A Baseline Desktop Screenshot | R1.3A-V Stage A Proposed Solution |
|---|---|---|---|
| **Window Frame** | Seamless macOS titlebar with custom traffic light clearance (`h-11`). | Standard window header with plain text labels. | Unified glass header (`h-12`) with native `data-tauri-drag-region` and refined version badge. |
| **Stage Focal Center** | Centered, tranquil composition with generous negative space. | Left-aligned grid crowded next to the navigation bar. | Centered stage (`max-w-5xl mx-auto`) with balanced spatial symmetry and deep breathing room. |
| **Hero Status Header** | Elegant typography with integrated macro telemetry. | Cluttered text rows competing with a bulky telemetry ribbon. | Sculpted SF Pro Display title with inline monospace system telemetry and subtle glass action buttons. |
| **Service Telemetry** | Quiet, integrated triple-channel health indicators. | Heavy rectangular box with bright green pills and contrasting borders. | Refined, borderless micro-pills with glowing pips, clean typography, and interactive diagnostic popovers. |
| **Spatial Cards** | Floating 2.5D physical objects with top specular light rim. | Generic dark flat cards with uniform gray borders. | Multi-layered frosted glass with 1px top specular gradient rim, authentic contact shadow, and tactile footer. |
| **Dock Placement** | Detached vertical liquid glass capsule. | Attached-looking dark rectangle hugging the left edge. | Floating liquid dock with 32px backdrop blur, sliding spring indicator, and accessible tooltips. |
| **Command Capsule** | Floating bottom capsule with soft indigo aura. | Flat dark input box with basic border. | Frosted smoked acrylic capsule with shimmering border glow, inline hotkeys, and full `⌘K` modal. |
| **Empty State** | Spacious spatial wireframe with actionable prompt. | Fabricated mock projects (`StepperSkip Store`) masked empty state. | Honest, accessible empty canvas with zero synthetic projects. |
| **Data Integrity** | 100% verified OS process state; zero fabricated telemetry. | Synthetic fallbacks for projects, MariaDB, and PHP-FPM. | Strict zero-fabrication boundary: real backend telemetry or honest `UNKNOWN` / `UNAVAILABLE`. |

### 8.2 Product Owner Visual Acceptance Criteria:
- [ ] **Criteria 1 (Stage Geometry)**: The main view is centered with balanced negative space, eliminating the left-crowded admin dashboard feel.
- [ ] **Criteria 2 (Atmospheric Lighting)**: The canvas features deep void foundations (`#05070E`) with soft, diffuse indigo ambient halos and no distracting neon glare.
- [ ] **Criteria 3 (Spatial Tactility)**: Spatial cards exhibit authentic 2.5D depth, directional specular top rims (`rgba(255,255,255,0.22)`), and smooth micro-tilt.
- [ ] **Criteria 4 (Liquid Dock)**: The navigation dock is a genuinely detached floating capsule with liquid glass styling and smooth spring-loaded route selection.
- [ ] **Criteria 5 (Command Capsule)**: The AI Command Capsule floats gracefully at the bottom center with subtle aura and instantaneous `⌘K` modal expansion.
- [ ] **Criteria 6 (Zero Data Fabrication)**: Zero synthetic projects are injected when empty. Runtime errors produce honest `UNKNOWN` or `UNAVAILABLE` indicators. PHP-FPM is never hardcoded as `RUNNING`.
- [ ] **Criteria 7 (Performance & Accessibility)**: Strictly 2 blur layers, 60fps/120Hz smooth rendering, WCAG 2.2 AA compliant contrast, and full `prefers-reduced-motion` support.

---

## Conclusion & Stage A Authorization Request

This design proposal completely addresses the visual failure of R1.3A, establishing the precise architectural and styling blueprint to reproduce the approved Spatial Studio reference image while enforcing strict data integrity.

In strict adherence to the authorized workflow, **no application code has been modified in Stage A**. Implementation (Stage B) will begin only upon explicit Product Owner approval of this document.

---

**R1.3A-V VISUAL PROPOSAL — AWAITING PRODUCT OWNER APPROVAL.**
