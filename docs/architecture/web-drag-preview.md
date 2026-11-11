# Web drag preview

## Source boundary

This slice ports OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c` GUI behaviour. Paths are
relative to `src/slic3r/GUI/`.

A drag in the Prepare canvas has two phases.

- While the pointer moves, only the selection's GL volumes change: `GLCanvas3D::on_mouse`
  calls `m_selection.translate(cur_pos - m_mouse.drag.start_position_3D)` for a canvas drag
  (`GLCanvas3D.cpp:5174-5211`), and the Move, Rotate and Scale gizmos call
  `Selection::translate`, `Selection::rotate` and `Selection::scale_and_translate`
  (`Selection.cpp:1279, 1343, 1475`). Each call applies the drag's total since the press to the
  transforms cached when the drag started (`m_cache`, `Selection::setup_cache` `:1238`;
  `dragging_center` and `rotation_pivot` `:2547-2548`). The Model is not changed.
- On release the edit is committed once: `do_move(L("Move object"))` for a canvas drag
  (`GLCanvas3D.cpp:5371-5378`), `do_move(L("Gizmo-Move"))` (`Gizmos/GLGizmoMove.cpp:135`),
  `do_rotate(L("Gizmo-Rotate"))` (`Gizmos/GLGizmoRotate.cpp:665`) and
  `do_scale(L("Gizmo-Scale"))` (`Gizmos/GLGizmoScale.cpp:232`). These write the volumes'
  transforms back to the Model, drop instances onto the bed and notify the plates
  (`GLCanvas3D::do_move` `:5643`, `do_rotate` `:5781`, `do_scale` `:5889`). Each first takes the
  undo snapshot (`take_snapshot`, `:5649, 5787, 5895`).

## Current Ares shell

`web/app/src/state/plates.ts` sends every pointer step through the worker as one
`PlaterSession.translate`, `rotate` or `scale` call. Each call runs `Selection::X` and then
`do_X` (`PlaterSession::translate_instance`, `rotate_instance`, `scale_instance`), and its
result rebuilds the scene. A drag of N steps therefore commits N times: N worker round trips,
N drops onto the bed and N scene updates. With several objects on the plate this makes dragging
visibly slow.

## Behaviour

- The press, the hit test and the per-step pointer geometry are unchanged
  (`web/app/src/viewport/instanceDrag.ts`).
- A press on a Rotate gizmo ring asks the core once for the instance's rotation pivot,
  `PlaterSession.rotationPivot`. This is `Selection::setup_cache` storing
  `m_cache.rotation_pivot = get_bounding_sphere().first` (`Selection.cpp:2548`), the point
  `Selection::rotate` turns about (`:1391`). The web does not compute the sphere. The answer
  is the only worker message before the release. The preview starts turning when it arrives.
- While dragging, the viewport sets the dragged instance's three.js matrix to its start matrix
  composed with the drag's total so far. No other worker message is sent and no geometry is
  rebuilt.
  - Canvas drag and Move gizmo: translation by the total world displacement.
  - Rotate gizmo: rotation by the total angle about the ring's axis through the core's rotation
    pivot. The angle is the one the pointer sweeps about the ring's centre.
  - Scale gizmo: the grabber's total ratio about the instance box centre, the core's
    `dragging_center` (`:2547`).
  - The open gizmo is placed on the previewed instance, so its handles follow the drag.
  - The previewed matrix is published as the canvas's `data-drag-matrix` while the drag lasts.
- On `pointerup` (and `pointercancel`) a drag that moved commits once. The single
  `PlaterSession.translate`, `rotate` or `scale` call gets the totals since the press, which is
  Orca's `Selection::X(total from m_cache)` followed by one `do_X`. The undo snapshot is taken
  just before that call, as `do_X` takes it, instead of when the drag starts. When the call
  returns, the core scene replaces the preview.
- Other instances of the dragged object (sibling instances) do not move during the drag. They
  take their synchronized transforms on release, from the core's commit.
- A press without movement commits nothing.

### Equivalence with the per-step shell

- **Move:** the per-step drop is a no-op for a drag that keeps Z (canvas drags lie on the
  horizontal plane through the press point; Move gizmo X/Y). The committed offset is the start
  offset plus the summed displacement, equal to the step-wise result up to floating-point
  summation order.
- **Rotate and scale:** the step-wise shell dropped onto the bed after every step, so a turn
  about X or Y and every scale differ from Orca. The new commit drops once on release, which is
  Orca's result. Rotations about one axis add up, and scale ratios multiply, so one call with the
  total equals Orca's `m_cache`-relative transform.

## Tests

`web/e2e/dragPreview.spec.ts`, desktop project. Worker messages are counted by wrapping
`Worker.prototype.postMessage` in an init script.

1. Asymmetric rotate. `tests/parity/island-visit-order/four-disjoint-prisms.3mf` holds one
   50 x 50 mm object whose bounding-sphere centre is about 1.4 mm from its box centre in XY. A
   quarter turn on the Z ring, pressed where no other ring crosses it, would land about 2 mm
   away from a preview turned about the box centre.
2. Four models. `tests/parity/byobject-perobject-multi-copy-brim/combined-brim.3mf` has two cube
   objects, the first with two instances; Add instance gives the first a third. The second cube
   is dragged, and it has no sibling instance. No committed single-plate fixture has four
   objects, and the web has no object copy and paste yet. The test does a canvas drag, a Move
   gizmo X drag, a Rotate gizmo 45 degree Z drag and a Scale gizmo X drag, each in steps.

For every drag:

- `data-gizmo` follows the pointer, so the preview moves.
- After release, the committed gizmo handles are within 1 px of the last preview, and the
  committed `data-instances` matrix of the dragged instance is within 1e-4 of the last
  `data-drag-matrix`.
- No other instance changes.
- While the button is down the worker receives nothing, except the one rotation pivot query of a
  ring press.
- After release, exactly one undo snapshot and one transform edit are sent.

After the four-model drags, one Ctrl+Z per drag restores each earlier `data-instances`.

The test records per pointer step the time from the step to the viewport's next draw, and writes
it with the worker messages to `target/web-e2e/drag-preview/result.json`. The same spec run on
the per-step shell records the step-wise numbers for comparison (it fails there). Both are
copied under the run's log directory.

`firstFrame.spec.ts` drags the model at the canvas centre. Its idle check now starts after the
release commit has changed `data-instances` and the viewport has drawn it. The existing
`plateMove`, `plateRotate`, `plateScale`, `plateMirror`, `platerToolbar`, `undoRedo` and
`viewportObjectMenu` specs keep passing, including their OrcaSlicer oracle legs.

## Deferred

- `Selection::synchronize_unselected_instances` during a drag (`Selection.cpp:1435, 1469,
  1697`): sibling instances follow on release, not while dragging.
- `Selection::ensure_not_below_bed` during a drag (`:1335, 1994, 3159`): the preview may dip
  below the bed until the release drop.
- The Rotate gizmo's placement. Orca centres the rings on the bounding sphere and sizes them by
  its radius (`GLGizmoRotate::init_data_from_selection`, `Gizmos/GLGizmoRotate.cpp:236-247`),
  and measures the pointer's angle about that centre. The web gizmo is centred on the box centre
  and sized from the box, as before this slice. The turn itself is about the core's pivot.
- Dragging the wipe tower (`Selection::translate` wipe tower branch) and keyboard nudges
  (`GLCanvas3D.cpp:1057-1058`, `finish_selection_edit` `:4336-4348`) stay as they are.
