# LocalForge Motion & Materials Specification

**Document Version**: 1.0.0  
**Status**: APPROVED DESIGN BASELINE  
**Focus**: Physics, Glass Refraction, Lighting Models & Accessibility  

---

## 1. Core Motion Philosophy

Motion in LocalForge is not ornamental; it is an **informational instrument** that reinforces spatial hierarchy and physical intuition:
1. **Physical Credibility**: UI elements behave like physical objects with mass, velocity, and surface friction. Nothing starts or stops instantaneously without inertia unless reduced motion is active.
2. **Apple Silicon Optimization**: Animations target 120Hz ProMotion displays with zero dropped frames, utilizing hardware-accelerated CSS transforms (`translate3d`, `scale3d`) and WebGL/Canvas micro-shaders.
3. **Restraint & Silence**: Animation durations never exceed 320ms for common interactions. Feedback is immediate, calm, and predictable.

---

## 2. Spring Physics Tokens

LocalForge standardizes on spring physics curves rather than generic bezier curves:

```
+-----------------------------------------------------------------------------------+
| Spring Preset    | Mass | Stiffness | Damping | Duration Target | Use Case        |
+------------------+------+-----------+---------+-----------------+-----------------+
| Snappy Spring    | 0.8  | 420       | 28      | 140ms - 180ms   | Buttons, Pills  |
| Spatial Spring   | 1.0  | 280       | 24      | 220ms - 260ms   | Cards, Dock Nav |
| Modal Spring     | 1.2  | 200       | 26      | 280ms - 320ms   | Dialogs, Drawer |
| Fluid Tilt       | 0.6  | 180       | 18      | Micro-following | 3D Card Hover   |
+-----------------------------------------------------------------------------------+
```

### CSS Transition Approximation
For lightweight CSS transitions where JavaScript spring solvers are unnecessary:
```css
/* Snappy UI Feedback */
--lf-ease-snappy: cubic-bezier(0.16, 1, 0.3, 1);
--lf-duration-snappy: 160ms;

/* Spatial Floating Movement */
--lf-ease-spatial: cubic-bezier(0.25, 1, 0.5, 1);
--lf-duration-spatial: 240ms;

/* Entrance & Exit */
--lf-ease-entrance: cubic-bezier(0, 0, 0.2, 1);
--lf-ease-exit: cubic-bezier(0.4, 0, 1, 1);
```

---

## 3. Materials & Refraction Architecture

LocalForge leverages a layered material system inspired by macOS `NSVisualEffectView`:

```
+-----------------------------------------------------------------------------------+
| LAYER 3: Specular Rim (1px inset gradient, simulates beveled glass edge highlight)|
|          linear-gradient(135deg, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.02)|
+-----------------------------------------------------------------------------------+
| LAYER 2: Tinted Smoked Substrate                                                  |
|          rgba(18, 25, 42, 0.65) with 200% color saturation                       |
+-----------------------------------------------------------------------------------+
| LAYER 1: Dual-Pass Blur Filter                                                    |
|          backdrop-filter: blur(28px) saturate(190%)                               |
+-----------------------------------------------------------------------------------+
| LAYER 0: Background Canvas with Soft Ambient Halo Lighting                        |
+-----------------------------------------------------------------------------------+
```

### 3.1 Specular Light Angle
All specular highlights are modeled with a fixed, consistent light source located at the **top-left (135 degrees)**, echoing standard macOS window lighting conventions.

---

## 4. Reduced Motion Compliance (`prefers-reduced-motion`)

To ensure vestibular safety and adhere to WCAG 2.2 Criterion 2.3.3:

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
    transform: none !important;
  }

  .lf-material-card:hover {
    transform: none !important;
    border-color: var(--lf-border-active) !important;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4) !important;
  }
}
```
When reduced motion is active:
- Card 3D tilt is completely eliminated.
- Hover states transition instantly via crisp border color and opacity changes.
- Loading spinners are replaced with static progress bars or explicit textual status labels.
