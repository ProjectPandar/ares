# A single-file import keeps the previous workspace inactive

Upstream baseline: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Upstream behavior

`Plater::load_files` delegates to `Plater::priv::load_files`
(`src/slic3r/GUI/Plater.cpp:18416–18424,8962`). Before processing the inputs, the
private function opens an application-modal `Loading…` progress dialog and creates
a `wxBusyCursor` (`:9007–9009`). `ProgressDialog` disables the other windows and
re-enables them when it finishes (`src/slic3r/GUI/Widgets/ProgressDialog.cpp:314,869–890`).
The STL path installs the loaded model objects during that operation
(`Plater.cpp:10119–10120`), and `load_model_objects` adds them to the object list
(`:10457–10461`). The previous workspace cannot accept selection while the load
is in progress.

## Web destination

This aligns the Web plater's interaction timing with the GUI; it does not port
the widgets or change the slicing engine. In `web/app/src/state/actions.ts`,
`importFile` currently reads the file before entering the loading state and its
error handling. A second import can therefore leave `Ready to slice` visible
with the old scene still interactive while the new file is being read.

- For an accepted single STL or 3MF, set `loadingFile` before requesting the bytes.
  Read the bytes and install the scene inside the existing error-handling scope.
  Clear loading only after publication and its existing setup finish or after
  reporting a failure. Rejected byte reads use the existing import error path
  and leave the previous scene available again.
- `web/app/src/shell/Shell.tsx` makes the workspace, including the object list and
  viewport, noninteractive while loading. The existing status bar remains available
  and shows `Loading model…`. On success the new workspace is interactive, with
  selection reset as it is today.

File admission, scene replacement, and multi-file confirmation and assembly keep
their existing behavior. A new modal dialog, percentages, cancellation, and
changes to slice or preview operations are out of scope.

## Verification

Add a Playwright app test under `web/e2e/`. Import a small STL through the picker
and wait for the first object's row and `Ready to slice`. Start a second import
and hold only its `File.arrayBuffer()` result until the test releases it. Wait
for the read to be entered, then require `Loading model…` and use a real pointer
click at the previous object's on-screen location. The click must not select
that object. Release the original read and verify that the old row is replaced,
the new scene shows `Ready to slice`, and the new object's row can be selected.
Observe public UI state only; use the held read and its release to order the test,
not timed sleeps or access to the app store.
