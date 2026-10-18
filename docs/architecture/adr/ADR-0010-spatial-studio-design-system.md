# ADR-0010: Spatial Studio Design System Architecture & Dual-Mode UI

- **Status**: ACCEPTED
- **Date**: 2026-10-18
- **Deciders**: Principal macOS Product Designer, Design Systems Architect, Tauri 2 UX Engineer
- **Phase**: Recovery Phase R0 / Planning Phase R1

---

## 1. Context

Early prototypes of LocalForge implemented a flat Apple-style dark mode dashboard. While functional, developer feedback and modern desktop ergonomics indicate the need for:
1. Greater spatial hierarchy and physical intuition for local vs remote environments.
2. A unified aesthetic that avoids both generic flat SaaS tables and over-stylized neon gimmicks.
3. Two distinct usage modes: an ambient, macro overview for quick management (**Spatial Studio**) and an information-dense, micro diagnostic environment (**Native Workspace**).

---

## 2. Decision

We establish the **Spatial Studio Design System** as the official visual and interaction specification for LocalForge:

### 2.1 Visual Foundation
- **Atmospheric Canvas**: Void dark foundations (`#06080D` to `#0B0F19`) accented with restrained, diffuse indigo ambient halos (`rgba(79, 110, 247, 0.15)`).
- **Spatial 2.5D Depth**: Layered Z-axis elevations, multi-stop soft shadows, and 1px top-left specular glass highlights (`linear-gradient(135deg, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.02) 100%)`).
- **Floating Controls**: Floating vertical navigation dock and floating AI command capsule suspended over the stage with `backdrop-filter: blur(32px)`.

### 2.2 Dual-Mode Architectural Separation
The application provides two complementary, synchronized presentation views sharing the exact same underlying state:
- **Spatial Studio Mode**: Macro topology, 3D interactive project cards, ambient health halos, quick-launch triggers.
- **Native Workspace Mode**: Micro technical detail, split-pane file/manifest editors, live streaming log viewers, FPM pool tuning, and database consoles.

### 2.3 Triple-Channel Status Signaling (Accessibility)
To comply with WCAG 2.2 Level AA, status MUST NEVER rely on color alone. Every status representation requires:
1. Semantic Color (Emerald, Amber, Crimson, etc.)
2. Unique Glyph/Icon (Activity, AlertTriangle, AlertCircle, Unplug, etc.)
3. Explicit Text Label (e.g. "Running", "Degraded: Port Conflict", "Disconnected").

---

## 3. Consequences

- **Positive**: Establishes a unique, premium product identity that feels native to macOS Apple Silicon.
- **Positive**: Eliminates cognitive overload by cleanly separating high-level status monitoring from low-level debugging.
- **Trade-off**: Requires strict discipline in component authoring to maintain shared design tokens across both modes without code duplication.
