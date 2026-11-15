# Web viewport: plate and model rebuilds keep their shader programs

## Problem

`Viewport.setBed` and `Viewport.setScene` (`web/app/src/viewport/Viewport.ts`) dispose the
old plates and instances with their materials and build new ones with new materials. three.js
releases a shader program when the last material that uses it is disposed, so the next
render compiles and links the same programs again, on the main thread. Every new scene does
this: a plater edit that changes the mesh data, a preset change that re-places an STL, a
plate added or deleted. Under the e2e Chromium (SwiftShader) one rebuild blocks the main
thread for up to about 170 ms in `getProgramParameter` / `getShaderParameter`.

`web/e2e/worker.mobile.spec.ts` on `pixel` taps Slice right after choosing the presets. The
slice waits for the STL plater session to open (`state/stlSession.ts`, `stlSessionOpened`),
and the sessions of the preset changes land as the slice starts and replace the scene; the
rebuild that follows stalls the 200 ms heartbeat in about 3 of 10 runs.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`):

- `src/slic3r/GUI/GLShadersManager.cpp:22`, `GLShadersManager::init`, called once from
  `src/slic3r/GUI/OpenGLManager.cpp:318`: every shader program is compiled when OpenGL
  starts. `GLShadersManager::get_shader` (`:129-133`) looks a compiled program up by name.
- The bed and plates draw with the shader they look up at render time:
  `Bed3D::render_default` (`src/slic3r/GUI/3DBed.cpp:814`, `get_shader("flat")` at `:820`),
  `PartPlate::render` (`src/slic3r/GUI/PartPlate.cpp:5330`, `get_shader("flat")` at
  `:5334`) and its grid (`:2453`). The colour is a uniform set per draw
  (`GLModel::render`, `src/slic3r/GUI/GLModel.cpp:674`).
- The objects draw with the shader `GLCanvas3D::_render_objects` looks up
  (`src/slic3r/GUI/GLCanvas3D.cpp:9604`, `get_shader` at `:9677-9679`); `GLVolume::render`
  (`src/slic3r/GUI/3DScene.cpp:539-552`) uses the current program.
- A new bed shape or plate resets geometry only: `Bed3D::set_shape` resets its triangles
  (`3DBed.cpp:270`, `:341`), `PartPlate::calc_gridlines` its grid
  (`PartPlate.cpp:1632-1634`), and `GLModel::reset` (`GLModel.cpp:547-575`) deletes vertex
  and index buffers, never a program.

## Destination

- `web/app/src/viewport/bed.ts`: `plateMaterials` builds the materials every plate draws
  with (surface, minor and major grid, outline, current outline, axes) once per viewport.
  `buildPlates` builds plate geometry with them; `recolorPlates` sets the theme colours on
  them and redraws the index labels.
- `web/app/src/viewport/models.ts` (new, moved out of `Viewport.ts` first, unchanged):
  instance building. `ModelMaterials` holds one material per volume type and selection
  state, built on first use and kept for the viewport's life; a new filament colour is set
  on the model-part materials. Selection swaps an instance's materials instead of tinting
  its own.
- `web/app/src/viewport/dispose.ts`: `disposeGeometries` frees the geometry of a tree and
  leaves its materials.
- `web/app/src/viewport/Viewport.ts`: `setBed` and `setScene` dispose geometry only;
  `dispose` frees the kept materials.

## Included

- The plates (surface, grids, outlines, axes) and the object instances (model parts and the
  non-part volume styles, selected or not).

## Deferred

- Toolpaths, option markers and gizmos keep building and disposing their own materials;
  the stall this slice removes does not involve them.
- Plate index labels keep one sprite material and canvas per plate, as now.

## Failure modes and how each is ruled out

1. **A rebuild compiles shaders.** `web/e2e/viewportRebuild.spec.ts` counts `linkProgram`
   calls while counting `bufferData` uploads as the witness that a rebuild happened:
   - deleting an object of a two-plate project (new scene: plates and instances rebuilt);
   - adding a plate (plates rebuilt);
   - switching the process preset of an STL with presets (a new plater session replaces
     the scene, the path that lands after the Slice tap).
   Each uploads buffers and links no program.
2. **Shared materials draw differently.** The selection tint and the volume styles stay
   as they are: `web/e2e/visual.spec.ts`, `web/e2e/viewportTheme.spec.ts` (switched
   equals built) and the plater specs that select objects.
3. **The UI stalls while the worker slices.** `web/e2e/worker.mobile.spec.ts` on `pixel`
   passes in every one of repeated runs.

## Commits and acceptance

1. `docs(web)`: this document.
2. `test(web)`: `web/e2e/viewportRebuild.spec.ts`; on the current web each rebuild links
   programs.
3. `refactor(web)`: instance building moves from `Viewport.ts` to `models.ts` unchanged.
4. `fix(web)`: kept plate and model materials, geometry-only disposal.

Checks: typecheck and lint; viewportRebuild, viewportTheme, worker.mobile (pixel repeated,
iphone, ipad), mobile, visual, persistence, platerToolbar, multiPlate, plateDeleteObject,
viewportObjectMenu and undo/redo of plater edits.
