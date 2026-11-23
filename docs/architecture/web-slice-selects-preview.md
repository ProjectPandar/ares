# Web: the Slice buttons select the Preview tab

## Problem

In OrcaSlicer, clicking "Slice plate" or "Slice all" switches the notebook from Prepare to
Preview as the slice starts. The web Slice buttons start the slice and leave the Prepare tab
selected; the user has to open Preview by hand. Ctrl+R already switches.

## Upstream reference

OrcaSlicer v2.5.0 (`40ecc66f7`), GUI only; no `libslic3r` behaviour is involved.

- `src/slic3r/GUI/MainFrame.cpp:2392-2417`, the `m_slice_btn` `wxEVT_BUTTON` handler:
  `exit_gizmo()`, `update(true, true)`, then `try_pop_up_before_slice(...)` (`:2400-2408`).
  When it returns true, the handler posts `EVT_GLTOOLBAR_SLICE_ALL` for `eSliceAll` or
  `EVT_GLTOOLBAR_SLICE_PLATE` otherwise (`:2411-2414`) and then calls
  `m_tabpanel->SelectPageByName(TAB_ID_PREVIEW)` (`:2415`). The event is posted, so the tab
  switches before the slice runs; Preview shows the progress and then the result. When it
  returns false, nothing is posted and the tab stays.
- `src/slic3r/GUI/FilamentMapDialog.cpp:127-215`, `try_pop_up_before_slice`: it returns true
  without a dialog unless the printer is a BBL-vendor printer with exactly two nozzles, and
  then only in manual filament-map mode or when forced (Linux forces it for BBL two-extruder
  printers, `MainFrame.cpp:2402-2405`). Only that dialog returns false: on cancel (`:214`), or on OK with a filament that
  neither extruder can print (`:209-210`).
- `src/slic3r/GUI/Plater.cpp:13747-13785`, `Plater::priv::on_action_slice_plate`: the posted
  event reslices and calls `select_view_3D("Preview")` (`:13783`), the canvas half of the same
  switch. Its IDEX/IQEX warning (`:13751-13770`) answers No with `select_view_3D("3D")` (`:13767`).
- `src/slic3r/GUI/MainFrame.cpp:892-897`, the `SlicePlate` shortcut (Ctrl+R,
  `Shortcuts.cpp:69`): posts `EVT_GLTOOLBAR_SLICE_PLATE` and selects Preview, without
  `try_pop_up_before_slice`. No menu item slices.

## Ares today

- `web/app/src/shell/TopBar.tsx:116-125`: "Slice plate" calls `slicePlate()` and "Slice all"
  calls `sliceAllPlates()` (`web/app/src/state/actions.ts:153-200`). Neither changes the route.
- `web/app/src/shell/Shell.tsx:45-47`: Ctrl+R calls `slicePlate()` and navigates to
  `/preview`, as `MainFrame.cpp:892-897` does.
- The tabs are the routes `/prepare` and `/preview`. `PreviewPage` shows "Slicing…" while the
  slice runs, then the result.
- There is no pre-slice dialog. The filament map (`web/app/src/sidebar/FilamentMapPanel.tsx`)
  is edited in the sidebar only, and nothing asks before a slice. So the web behaves as if
  `try_pop_up_before_slice` returned true.
- The buttons are disabled while there is no scene, while a slice runs, before the engine is
  ready and while `presetSliceBlocker` reports a blocker (`TopBar.tsx:116-124`). The early
  returns of `slicePlates` (`actions.ts:170-171`: no file or scene, a running slice, a blocker)
  cannot fire for a click on an enabled button, so such a click always starts a slice. A disabled button cannot be clicked, so the tab stays.

## Change

`TabBar` in `TopBar.tsx`: both Slice buttons call their action and then navigate to
`/preview`, in the click handler, as Ctrl+R does. The navigation does not wait for the slice,
as upstream selects the page right after posting the event.

### A canvas that mounts while its session opens

Each canvas (Prepare, Preview) asks for the STL session of the current inputs when it mounts
(`web/app/src/viewport/ViewportView.tsx:108-124`, `openStlSession` in
`web/app/src/state/stlSession.ts`). The plater keeps a session it has finished opening for the
same inputs, as switching tabs reloads nothing upstream. An opening that is still running is
not kept: a second canvas starts another opening of the same inputs, which replaces the first.
A slice waits for the opening it started under (`slicePlates`, `actions.ts:166-198`) and is
discarded when another opening replaced it, as a preset switch during a slice does
(Plater.cpp:11402-11466). With the tab switch, a click on Slice right after a preset change
mounts Preview while the session for the new preset opens, and the slice ends with the plate
unsliced.

`openStlSession` also hands a canvas the opening in progress when its inputs are exactly the
same (file, preset selection, vendor and imported preset sources). Any other input starts a new
opening that replaces the running one, as before.

### One GL context and its shader programs for both tabs

Each tab is a route that mounts its own `ViewportView`, and each `Viewport`
(`web/app/src/viewport/Viewport.ts:69`) creates a `THREE.WebGLRenderer`, so a new WebGL
context, and compiles its plate, model and gizmo programs on its first render. `dispose`
destroys both. The toolpath and option-marker `ShaderMaterial`s are built for each result
(`toolpathMesh.ts`, `optionMarkers.ts`); `setToolpaths` disposes the old ones first, and
three.js releases a program when the last material that uses it is disposed, so every result
in a newly mounted Preview compiles the segment program again. Before the change these costs
fell on the user's own switch to Preview. With the tab switch they fall inside the slice:

- When Preview mounts as the slice starts, the main thread is blocked for about 100 ms under
  the e2e Chromium (SwiftShader). About a third of the CPU samples are the new context
  (`getContext`) and about a quarter are program compiles and links on the first render. On
  mobile emulation under load this reaches 200-320 ms, over the 200 ms limit of
  `web/e2e/worker.mobile.spec.ts`.
- When the result reaches the open Preview, the first frame waits about 12 s for the segment
  program (`GetShaderiv`). That browser has no `KHR_parallel_shader_compile`.

Upstream creates the context and the programs once for the application:

- `OpenGLManager::init_glcontext` (`src/slic3r/GUI/OpenGLManager.cpp:352-378`) creates
  `m_context` on the first call and returns it to every later canvas; `View3D::init` and
  `Preview::init` (`src/slic3r/GUI/GUI_Preview.cpp:90`, `:289`) both take it through
  `GUI_App::init_glcontext` (`src/slic3r/GUI/GUI_App.cpp:2643-2646`).
- `GLShadersManager::init` (`src/slic3r/GUI/GLShadersManager.cpp:22`), called once from
  `OpenGLManager.cpp:318`, compiles every program when OpenGL starts.
- `ViewerImpl::init` (`src/libvgcode/src/ViewerImpl.cpp:790-867`) compiles the segment and
  option programs once, through `GCodeViewer::init` (`src/slic3r/GUI/GCodeViewer.cpp:1201`),
  from the Preview canvas's render and from `GLCanvas3D::load_gcode_preview`
  (`src/slic3r/GUI/GLCanvas3D.cpp:2558`, `:3888`). A new result loads data into the existing
  buffers and textures, never a program.
- Switching tabs shows one panel and hides the others (`Plater::priv::set_current_panel`,
  `src/slic3r/GUI/Plater.cpp:12564-12720`); no canvas is created or destroyed.

Change, in `web/app/src/viewport/`:

1. One `WebGLRenderer` for the application, created by the first `Viewport`. A `Viewport`
   attaches the renderer's canvas to its container; `dispose` detaches it and releases the
   viewport's own geometry, controls and listeners, never the renderer.
2. The plate and model materials (now one set per `Viewport`) live with that renderer, as
   `GLShadersManager` programs live with the context, so the Preview canvas reuses the
   programs Prepare linked.
3. One toolpath material and one option-marker material, also kept with the renderer, as
   `ViewerImpl` keeps its program ids. A result replaces only their textures, geometry and
   uniforms. The first Preview `Viewport` issues their compile and link with
   `renderer.compile` when it mounts, as `GCodeViewer::init` runs on the Preview canvas's
   first render; three.js checks a program's status on first use, so the mount does not wait.
   A browser with `KHR_parallel_shader_compile` links while the slice runs. Without it (the
   e2e Chromium) ANGLE links on first use, so the first result frame of the application still
   waits for the link, about 15 s under SwiftShader; every later result and Preview mount uses
   the linked programs.
4. Theme colours keep updating the shared materials (`setColors`). Only one `Viewport` exists at
   a time, because a route switch unmounts the other tab.

Spreading the work across frames does not apply: the blocking calls are one context creation
and one wait for a program, neither of which can be split. Moving rendering to a worker
(`OffscreenCanvas`) has no upstream counterpart and is deferred.

### One frame for a result

The web draws a new result before its moves view exists. `setToolpaths` adds the mesh with
every segment instanced (`toolpathMesh.ts`), and the enabled segments and view colours arrive
later from the engine (`MovesSlider` `onMoves`, then `setEnabled` / `setToolpathColors`). The
Legend and the sliders that appear with the result change the layout, and `#fitCanvas` calls
`setSize` and draws on every `ResizeObserver` callback, even when the size has not changed.
Under the e2e Chromium a full frame of the large `worker.mobile` result (562032 segments)
takes about 15 s of GPU work, and one result draws three: two of all segments (the resize and
the queued render) and one of the enabled segments. On WebKit each full frame blocks the main
thread for 2-3 s.

Upstream draws a loaded result once with its view applied:

- `ViewerImpl::load` (`src/libvgcode/src/ViewerImpl.cpp:1121-1298`) ends with
  `update_colors()` and `update_enabled_entities()` (`:1296-1297`), and `ViewerImpl::render`
  applies pending updates before `render_segments` (`apply_pending_updates`, `:1496-1515`),
  so the first frame of a result draws its enabled segments in its colours.
  `GCodeViewer::load_as_gcode` sends the data with `m_viewer.load` (`GCodeViewer.cpp:1625`).
- `GLCanvas3D::on_size` only marks the canvas dirty (`src/slic3r/GUI/GLCanvas3D.hpp:1278`),
  and `GLCanvas3D::_resize` returns when the size is unchanged (`GLCanvas3D.cpp:7895-7902`).
  The dirty canvas draws once on idle (`GLCanvas3D::on_idle`, `GLCanvas3D.cpp:4027-4056`).

Change, in `web/app/src/viewport/`:

1. A result's toolpath mesh is hidden until its first enabled view reaches the viewport
   (`setEnabled`); that view and its colours arrive in one commit and draw in one frame. The
   viewport sets a new result's toolpaths in the same commit, before that view, as a layout
   effect.
2. `#fitCanvas` with an unchanged container size neither calls `setSize` nor draws, as
   `_resize` returns. A changed size still resizes and draws at once: `setSize` clears the
   drawing buffer before the frame is painted.

`web/e2e/worker.mobile.spec.ts` and its 30 s screenshot timeout stay. Under the e2e Chromium
the first result frame of the application still waits for the segment program's link
(above), so its screenshot follows one link and one full frame instead of a link and three.

### Proposed, not implemented: tabs that stay mounted

Each tab is a route, so a switch unmounts one page (its sidebar, viewport, scene and key
handlers) and mounts the other. On WebKit a switch costs 80-120 ms of main-thread time with the
shared context (157-174 ms before it), almost none of it in GL; when the switch coincides with a
slice start under load it reaches 220 ms.

Upstream builds the panels once and only shows and hides them:

- `Plater::priv` creates one `Sidebar` (`src/slic3r/GUI/Plater.cpp:7876`) and the `View3D` and
  `Preview` panels once (`:7935-7937`, `panels` at `:7948-7950`).
- `Plater::priv::set_current_panel` (`:12564-12720`) shows the new panel first and then hides
  the others (`:12702-12720`); it creates and destroys nothing.

Proposed change, in `web/app/src/`:

1. `Shell` renders one `Sidebar` and both tab pages; the route only selects the shown page
   (`hidden` on the other). A page's key handlers act only while it is shown.
2. Each page keeps its `Viewport` and scene. The shown page's `Viewport` holds the shared canvas;
   a switch moves the canvas and draws once.

Open before any implementation: 28 specs select the viewport by
`getByTestId("viewport")` / `[data-testid=viewport]` (54 uses), and `slice-summary` and the view
toolbar would exist on both pages. Playwright locators match hidden elements, so each of these
selectors would need scoping to the shown page, or the hidden page would have to drop those test
ids. The open gizmo, the camera and the Preview sliders keep their state across switches, as
upstream does; specs that assume a reset would change.

## Verification

New Playwright spec `web/e2e/sliceSelectsPreview.spec.ts`, synthetic fixtures already in the
repository:

1. Import an STL on Prepare, click "Slice plate": the URL is `/preview` and the Preview tab is
   active while the slice is still running, and Preview then shows the result (layer count).
2. Open `tests/parity/custom-gcode-overview/two-plates.3mf` (two plates), click "Slice all":
   the URL is `/preview`, and the result is plate 2's.

Both fail before the change, because the route stays `/prepare`.

In `web/e2e/presetSwitchSlice.spec.ts`, with the existing worker relay of that spec: after a
first slice, switch the process preset twice; the relay holds the second switch's session
opening. Click "Slice plate" while it is held, then release it: the plate slices and the relay
saw no further opening. The hold is ordered by events, not timing. Before the session change
the Preview canvas starts a second opening and the slice ends unsliced.

Existing specs that click a Slice button and then drive Prepare-only controls follow the tab
switch. The whole web suite runs before and after the change. A spec that breaks only because
it is now on Preview gets a test-side navigation step back to Prepare, or a selector scoped to
the tab bar (Preview's "Plate not sliced" panel has its own "Slice plate" button). No
screenshot or G-code reference changes.

For the shared context: a new spec in `web/e2e/sliceSelectsPreview.spec.ts` marks the WebGL
context of the Prepare canvas (`canvas.getContext("webgl2")` returns the context the renderer
holds), clicks "Slice plate", and reads the mark from the Preview canvas's context. Before the
change Preview has a new context without the mark. `web/e2e/worker.mobile.spec.ts`, unchanged,
must pass on iphone, pixel and ipad: its 200 ms heartbeat now covers the Preview mount, and its
screenshot comes after the first frame of the result. `web/e2e/previewResultResponsive.spec.ts`
keeps passing. The whole web suite is paired before and after the change.

`presetSwitchSlice.spec.ts` ("a slice started while a preset switch opens the STL session")
waited for the Preview canvas as a canvas the Prepare tab did not have. Both tabs now show the
one canvas; the test waits for the canvas of the Preview viewport that holds the "Slice result"
card. The canvas attaches after that viewport's mount effects, which request its session.

## Deferred

- Rendering in a worker through `OffscreenCanvas`.
- The link of the segment program under SwiftShader without `KHR_parallel_shader_compile`
  (about 15 s, once per application, on the first result frame).

- The pre-slice filament-map dialog of BBL two-nozzle printers and its cancel path; the web
  has no such dialog yet.
- The IDEX/IQEX pre-slice warning and its return to Prepare.
- `get_enable_slice_status` (`MainFrame.cpp:2521-2570`) disables "Slice plate" while the
  current plate's result is valid, and on G-code or exported-3MF projects; the web enables it.
- Ctrl+R's condition: upstream checks `m_slice_enable`, the web checks the engine and the
  scene only.
