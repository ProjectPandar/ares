# Scale and size fields on the plater session (v2.5.0-beta)

Every `file:line` is at OrcaSlicer `345efec64`. This slice ports the "Scale ratios" and "Size" fields of the object operations panel, for one selected instance in world coordinates. That is how the existing move and rotate fields work (`web-plater-move-rotate-arrange.md`, "Move" and "Rotate"). `web-plater-move-rotate-arrange.md` lists scale as deferred; this ARD takes it out of that list.

## Upstream boundary
- **The fields**, in `slic3r/GUI/Gizmos/GizmoObjectManipulation.cpp`:
  - The values shown (`update_settings_value`, `:115-150`). For a single full instance in world coordinates, the size is the selection's box in the current reference system (`:132`), the unscaled size is the unscaled instance box (`:133`), and the scale is `size / unscaled * 100` (`:134`).
  - `change_scale_value` (`:408-430`). An edit of one axis takes the cached scale with that axis replaced and divides it by `100 * Ones()` for a single full instance (`:420-421`).
  - `change_size_value` (`:433-460`). The size edit divides by the full unscaled instance box size in world coordinates (`:447-449`).
  - `do_scale` (`:463-481`). A uniform scale applies the edited axis's factor to all three axes (`:475`), then `limit_scaling_ratio`, `Selection::setup_cache`, `Selection::scale` and `GLCanvas3D::do_scale`.
  - `limit_scaling_ratio` (`:484-490`): no axis grows past `MAX_NUM` 9999.99 mm (`:45`).
  - The uniform toggle, `m_uniform_scale {true}` (`GizmoObjectManipulation.hpp:99`, `set_uniform_scaling` `:618-626`), and `reset_scale_value` (`:609-616`), which sets all three axes back to 100.
- **The selection math**, in `slic3r/GUI/Selection.cpp`:
  - `scale` (`:1475-1478`) and `scale_and_translate` (`:1627-1719`). In instance mode with an absolute world scale, the factor becomes relative through the full unscaled instance box (`:1631-1645`). `transform_instance_relative` (`:3364-…`) then applies it about the selection's dragging centre, which `setup_cache` sets (`:1238`).
  - `synchronize_unselected_instances(GENERAL)` (`:3045-…`): the object's other instances take the scale. `DISABLE_INSTANCES_SYNCH` is 0 (`libslic3r/Technologies.hpp:18`).
  - The boxes: `get_full_unscaled_instance_bounding_box` (`:1050-…`) and `get_bounding_box_in_current_reference_system` (`:1107-…`).
- **The model update**, `GLCanvas3D::do_scale` (`slic3r/GUI/GLCanvas3D.cpp:5889-5985`). The instance transforms go to the model. An `auto_drop` instance that was not sinking before the scale (`min_zs`, `SINKING_Z_THRESHOLD`), or that ends above the bed, drops onto it (`:5949-5962`). This is the same rule as `do_rotate`, which `plater::selection::drop_after_rotate` already ports.

## Ares destination
- **Core**, `crates/ares-core/src/plater/`:
  - `PlaterSession::set_scale(object, instance, axis, percent, uniform)` and `set_size(object, instance, axis, mm, uniform)`: the two field edits, through a `scale_selection` that ports `Selection::scale` in world instance mode with its instance sync.
  - `scale_and_size(object, instance)`: the values the fields show.
  - The drop reuses `selection::drop_by` with `do_scale`'s rule.
- **WASM and web**: `PlaterSession.setScale` / `setSize` / `scaleAndSize`, and the Scale (%) and Size (mm) rows with the uniform toggle in `sidebar/ObjectManipulation.tsx`, beside Position and Rotate.

## Included and deferred
- **Included:** uniform and per-axis scale and size edits, the scaling limit, the reset, the instance sync of a multi-instance object, and the drop to the bed.
- **Deferred:**
  - The scale gizmo's drag handles (`GLGizmoScale3D`).
  - `scale_to_fit_print_volume`.
  - Volume (part) scaling.
  - The local and instance coordinate systems; the web shows world coordinates, as for move and rotate.
  - Undo snapshots, as for every plater edit.

## Oracle and RED
- **Oracle.** Native slicing the project the GUI exports for the session (`--slice <plate>`), as the move and rotate cases do (`crates/ares-cli/tests/orca_parity/plater_session.rs`). The session's instance transform is checked against the matrix the upstream math gives, and its G-code against Native's.
- **RED** (a `plater_session` child module):
  1. Uniform 120 % on `orca_cli_ender3.3mf`'s 10 mm cube at (110, 110, 5). It scales about its centre to 12 mm, then drops back to the bed: diag(1.2, 1.2, 1.2), offset (110, 110, 6).
  2. Per-axis size X 12 mm with the uniform toggle off: diag(1.2, 1, 1), offset (110, 110, 5).
  3. A multi-instance object (`tests/parity/plater-scale/two-instances.3mf`, the same cube as two instances, made by `generate.mjs` beside it): scaling one instance by 120 % scales the other as well, as `synchronize_unselected_instances` does.
- The values stay where slicing the scaled cube is at Native parity. A 15 mm cube or a 20 x 10 x 10 mm box hits a crosshatch sparse-infill gap that does not depend on scaling: the same gap shows with the scale baked into the mesh.
- **Web E2E** (with the UI commit): the Scale and Size fields in `plateScale.spec.ts`, after `plateRotate.spec.ts`, against Native slicing the GUI export.
