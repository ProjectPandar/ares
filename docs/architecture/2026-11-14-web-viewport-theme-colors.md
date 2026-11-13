# Web viewport: a theme change recolours the plates

## Problem

Switching the theme calls `Viewport.setColors` (`web/app/src/viewport/ViewportView.tsx`,
the `theme` effect). `setColors` calls `setBed`, which disposes the plate group and builds
it again with new materials (`web/app/src/viewport/Viewport.ts`, `bed.ts` `buildPlates`).
Disposing the last material that uses a shader program releases the program, so the next
render compiles and links the bed shaders again, on the main thread. Under the e2e
Chromium (SwiftShader) one switch blocks the main thread for about 190 ms in
`getProgramParameter` / `getShaderParameter`; `web/e2e/worker.mobile.spec.ts` taps the
theme while the worker slices and its 200 ms heartbeat assertion fails on the `pixel`
project.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`):

- `src/slic3r/GUI/GLCanvas3D.cpp:1787`, `GLCanvas3D::on_change_color_mode`: passes the
  mode to the bed, the G-code viewer, the plates and the toolbars.
- `src/slic3r/GUI/3DBed.cpp:387-390`, `Bed3D::on_change_color_mode`, and
  `src/slic3r/GUI/PartPlate.hpp:1059`, `PartPlateList::on_change_color_mode`: store
  `m_is_dark`; the next render draws the same models in the other colours.
- `src/slic3r/GUI/GLShadersManager.cpp:22`, `GLShadersManager::init`, called once from
  `src/slic3r/GUI/OpenGLManager.cpp:318`: every shader is compiled when OpenGL starts. A
  colour-mode change compiles nothing.

## Destination

- `web/app/src/viewport/bed.ts`: `recolorPlates` sets the colours of the plates
  `buildPlates` built (surface, minor and major grid, outline, the current plate's outline
  and index label) on their existing materials, and redraws each index label on its own
  canvas.
- `web/app/src/viewport/Viewport.ts`: `setColors` sets the background and recolours the
  plates; it no longer rebuilds them. `setBed` still builds plates for a new bed shape,
  plate list or current plate, as now.

## Included

- The theme colours `ViewportColors` carries: background, bed, grid minor and major,
  outline, current plate.

## Deferred

- Colours that do not follow the theme (models, toolpaths, axes, gizmos) are unchanged.

## Failure modes and how each is ruled out

1. **A theme switch compiles shaders.** `web/e2e/viewportTheme.spec.ts` counts
   `linkProgram` calls across a switch to light and back to dark with a two-plate project
   open: none.
2. **The recoloured plates differ from plates built in that theme.** The same spec
   compares the canvas after the switch to light with the canvas of a page started in
   light with the same project open: equal pixels.
3. **The UI stalls while the worker slices.** `web/e2e/worker.mobile.spec.ts` on `pixel`
   keeps its 200 ms heartbeat assertion.

## Commits and acceptance

1. `docs(web)`: this document.
2. `test(web)`: `web/e2e/viewportTheme.spec.ts`; on the current web the switch links
   programs.
3. `fix(web)`: `recolorPlates` and `setColors`.

Checks: typecheck and lint; viewportTheme, worker.mobile (pixel, iphone, ipad), mobile,
visual, persistence and platerToolbar.
