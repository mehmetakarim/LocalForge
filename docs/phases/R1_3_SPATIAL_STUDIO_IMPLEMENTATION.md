# LocalForge — Phase R1.3: Spatial Studio UI Implementation Specification

**Phase**: Recovery Phase R1.3  
**Status**: SPECIFIED / AWAITING R1.2 COMPLETION  
**Prerequisite**: Formal Product Owner Approval of Phase R1.2  
**Deciders**: Principal macOS Product Designer, Frontend Lead, Tauri 2 Architect  

---

## 1. Explicit Scope

Phase R1.3 transitions the desktop application (`apps/desktop`) from the initial prototype to the approved **Spatial Studio** design system, integrating the live runtime engine verified in R1.1 and R1.2.

### In Scope for R1.3:
1. **Design System Tokens & CSS Architecture**:
   - Establish CSS variables for atmospheric void canvas, indigo lighting halos, and specular highlights in `apps/desktop/src/index.css`.
   - Implement dual-mode switcher: Spatial Studio (macro topology) vs Native Workspace (micro technical detail).
2. **Component Implementation**:
   - `SpatialCard`: 2.5D interactive card with micro-tilt physics, status halo, and action deck.
   - `FloatingDock`: Left-anchored liquid glass navigation dock with spring pill active indicator.
   - `AICommandCapsule`: Bottom floating command bar (`Cmd+K`) expanding into fuzzy search and MCP prompt center.
   - `StorageTopologyCard`: Visual APFS disk representation showing internal SSD vs `/Volumes/...` external drive with safety barrier tags.
   - `ServiceHealthPill`: Triple-channel operational status chip (color + icon + text).
3. **Performance & Hardware Safeguards**:
   - Enforce idle CPU < 1.0% and memory < 80MB.
   - Enforce 60fps baseline with opportunistic 120Hz ProMotion support.
   - Max 2 concurrent backdrop-filter blur layers.
   - Automatically pause breathing pulse animations when window is out of focus or state is static.
   - Virtualize or paginate project grid when count exceeds 12 projects.
   - Complete `prefers-reduced-motion` compliance.

---

## 2. Test Matrix for R1.3

| Test ID | Level | Target | Test Procedure | Acceptance Threshold |
|---|---|---|---|---|
| **TEST-R1.3-01** | Visual | `apps/desktop::SpatialCard` | Render card; verify 1px specular top highlight and 3D micro-tilt on mousemove. | Smooth transform, 0 layout shifts |
| **TEST-R1.3-02** | Accessibility | `apps/desktop::A11y` | Inspect color contrast for all text tokens; test keyboard tab navigation and focus rings. | WCAG 2.2 AA compliant (>4.5:1 contrast) |
| **TEST-R1.3-03** | Motion | `apps/desktop::Motion` | Enable `prefers-reduced-motion`; verify all 3D tilt, spring animations, and glow pulses cease. | Zero transform motion, instant crossfades |
| **TEST-R1.3-04** | Performance | `apps/desktop::Perf` | Measure idle CPU/memory over 60 seconds with 5 project cards active. | CPU < 1.0%, RAM < 80MB |
| **TEST-R1.3-05** | Scalability | `apps/desktop::Grid` | Load mock catalog of 30 projects; verify pagination or virtualized rendering. | Compositor maintains 60fps on scroll |
| **TEST-R1.3-06** | Dual-Mode | `apps/desktop::Navigation` | Switch between Spatial Studio and Native Workspace; verify state synchronization. | Instant switch (`<50ms`), identical state |
| **TEST-R1.3-07** | Build & CI | `apps/desktop` | Run `npm run build` (`tsc && vite build`) and Tauri build check. | 0 TypeScript errors, clean bundle |

---

## 3. Acceptance Scenario & Exit Criteria

1. **Aesthetic Parity**: The desktop window faithfully reproduces the approved Spatial Studio atmosphere (deep void canvas, soft indigo lighting, floating dock).
2. **Accurate Telemetry**: PIDs, socket paths, and memory statistics reflect real backend state from `crates/runtime`.
3. **Evolving Status States**: Project state dynamically transitions between `STOPPED`, `STARTING`, `RUNNING`, `DEGRADED`, and `DISCONNECTED`.
4. **Accessible Navigation**: Complete keyboard navigation via `Cmd+1..7` and `Cmd+K`.
5. **Production Build**: Clean frontend build and passing workspace tests.

---

## 4. Evidence & Handoff Requirements

- Git commit on branch `phase-r1/r1-3-spatial-studio-implementation`.
- Screen recordings / screenshots validating Spatial Studio and Native Workspace modes.
- Product Owner final acceptance before merging to `main`.
