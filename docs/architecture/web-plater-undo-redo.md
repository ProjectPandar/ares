# Web plater undo and redo

## Source boundary

This slice ports OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c` GUI behaviour. Paths are
relative to `src/slic3r/`.

- `Plater::priv::take_snapshot` (`GUI/Plater.cpp:15260-15330`) saves the `Model`, the selection
  and the `PartPlateList` before an edit, through `UndoRedo::StackImpl::take_snapshot`
  (`Utils/UndoRedo.cpp:937-975`): the snapshots ahead of the active one are released, and the
  current state becomes an unsaved topmost state.
- `Plater::priv::undo` / `redo` (`GUI/Plater.cpp:15350-15381`) move one modifying snapshot back or
  forward and are refused while a job such as an arrange runs (`undo_redo_blocked_by_job`).
  `StackImpl::undo` (`Utils/UndoRedo.cpp:1091-1130`) first captures the topmost state so redo can
  return to it; `load_snapshot` (`:1009-1089`) puts the model, selection and plate list back.
- A loaded project starts an empty stack (`SnapshotType::ProjectSeparator`,
  `GUI/Plater.cpp:15309-15310, 16216`).
- The plater edits the web ports take a snapshot first: canvas and gizmo drags
  (`GLCanvas3D::do_move` / `do_rotate`, `GUI/GLCanvas3D.cpp:5643-5649, 5781-5787`), the object
  manipulation fields (`GizmoObjectManipulation.cpp:343, 372, 400`), mirror and scale to fit, add
  and remove instance, set number of instances, delete of an instance, an object, a plate's objects
  and all objects, arrange, and add, delete and lock plate (`GUI/Plater.cpp:10767, 10795, 19293,
  19337, 19399, 19412, 23417, 23549, 23583`; `GUI/Selection.cpp:516, 1487`). Selecting a plate is
  a selection snapshot (`"select partplate!"`, `:22629`) that undo skips; the web takes none.
- Keys: Undo `Ctrl+Z` and Redo `Ctrl+Y` (`GUI/Shortcuts.cpp:74-75`), `Cmd` on macOS; `Ctrl+Shift+Z`
  redoes as well, as in the design panel (`GUI/CAD/DesignPanel.cpp:4276-4280`). They act on the
  Prepare tab (`PLATER | OBJECT_LIST`) and leave a focused text field its own undo.

## Rust and web destinations

- `crates/ares-core/src/plater/snapshot.rs`: `PlaterSnapshot`, `PlaterSession::snapshot` and
  `load_snapshot` over the session's model, plate list, arrange orders, tower positions and current
  plate. `crates/ares-core/src/project/domain/model_snapshot.rs` holds the model part: the objects
  and the per-plate custom G-codes, the project data plater edits change.
- `crates/ares-wasm/src/plater/snapshot.rs`: `takeSnapshot` / `loadSnapshot` bindings. The worker
  keeps each session's snapshots by id and frees them with the session; `PlaterSession`
  (`web/packages/ares-wasm-js`) exposes `takeSnapshot`, `loadSnapshot` and `dropSnapshots`.
- `web/app/src/state/undoRedo.ts`: the stack (`UndoRedo::Stack`), with the selection and current
  plate of each snapshot. A load drops every slice result like an edit.

## Deferred

- The stack's memory limit (`release_least_recently_used`); snapshots share the meshes.
- Gizmo enter/leave snapshots and their noisy-snapshot reduction, the undo/redo toolbar history
  and the dirty-project state.

## Verification

`web/e2e/undoRedo.spec.ts` makes five edits (canvas drag, add instance, Position X, arrange,
delete), then steps back and forward with `Ctrl+Z`, `Ctrl+Y` and `Ctrl+Shift+Z`. Each step must
show the recorded instances, plates, object list and canvas pixels. The spec also checks that a
new edit drops the redo states, that the keys do nothing in a focused field or on the Preview tab,
that an undo invalidates the slice, and that a loaded project has no history. Plate 1 must slice
to the same G-code bytes after undoing every edit as it did at load.
