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
as upstream selects the page right after posting the event. Nothing else changes.

## Verification

New Playwright spec `web/e2e/sliceSelectsPreview.spec.ts`, synthetic fixtures already in the
repository:

1. Import an STL on Prepare, click "Slice plate": the URL is `/preview` and the Preview tab is
   active while the slice is still running, and Preview then shows the result (layer count).
2. Open `tests/parity/custom-gcode-overview/two-plates.3mf` (two plates), click "Slice all":
   the URL is `/preview`, and the result is plate 2's.

Both fail before the change, because the route stays `/prepare`.

Existing specs that click a Slice button and then drive Prepare-only controls follow the tab
switch. The whole web suite runs before and after the change. A spec that breaks only because
it is now on Preview gets a test-side navigation step back to Prepare, or a selector scoped to
the tab bar (Preview's "Plate not sliced" panel has its own "Slice plate" button). No
screenshot or G-code reference changes.

## Deferred

- The pre-slice filament-map dialog of BBL two-nozzle printers and its cancel path; the web
  has no such dialog yet.
- The IDEX/IQEX pre-slice warning and its return to Prepare.
- `get_enable_slice_status` (`MainFrame.cpp:2521-2570`) disables "Slice plate" while the
  current plate's result is valid, and on G-code or exported-3MF projects; the web enables it.
- Ctrl+R's condition: upstream checks `m_slice_enable`, the web checks the engine and the
  scene only.
