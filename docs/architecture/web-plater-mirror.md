# Mirror on the plater session (v2.5.0-beta)

Every `file:line` is at OrcaSlicer `345efec64`. This slice ports the object menu's "Mirror" items for one selected instance. They are the next deferred item of `web-plater-move-rotate-arrange.md` after scale (`web-plater-scale.md`), and they reuse that slice's `Selection::scale_and_translate` port.

## Upstream boundary
- **The menu**, `MenuFactory::append_menu_item_transform` (`slic3r/GUI/GUI_Factories.cpp:1351-1364`). The object menu (`:1765`) shows "Mirror:" with "Along X Axis", "Along Y Axis" and "Along Z Axis", each calling `Plater::mirror(axis)` (`Plater.cpp:22348`, `Plater::priv::mirror` `:10936-10939`).
- **`GLCanvas3D::mirror_selection`** (`GLCanvas3D.cpp:3152-3162`): a *relative* transformation in world coordinates (`set_relative`), then `Selection::setup_cache`, `Selection::mirror` and `do_mirror`.
- **`Selection::mirror`** (`Selection.cpp:1708-1712`): `scale_and_translate` with the factor -1 on the axis and 1 elsewhere.
  - In `scale_and_translate` (`:1627-1719`) a relative type skips the absolute-to-relative conversion (`:1631-1647`). The factor applies as `T(c) · S · T(-c) · M` about the dragging centre (`transform_instance_relative`, `:3364-3372`).
  - The object's other instances follow (`synchronize_unselected_instances(GENERAL)`, `:1693-1697, 3045-3095`). A mirror flips handedness (`mirrored`, `:3062-3063`); the GENERAL sync applies the linear part `(old_j · old_i⁻¹) · curr_i` either way.
- **`GLCanvas3D::do_mirror`** (`:6010-6092`): the instance transforms go to the model, then each instance drops onto the bed unless it stays sinking. That is `do_scale`'s rule, with the lowest points taken before the mirror.

## Ares destination
- **Core**, `crates/ares-core/src/plater/`:
  - `selection::scale` takes the transformation's absolute or relative kind, as `scale_and_translate` does (scale is absolute, mirror relative).
  - `PlaterSession::mirror(object, instance, axis)` is the menu action. It runs the same drop as `do_scale`, so `do_scale`'s body becomes the shared `do_scale` / `do_mirror` step.
- **WASM and web**:
  - `PlaterSession.mirror(object, instance, axis)`.
  - A context menu on an object row of the object list, with upstream's labels ("Mirror:", "Along X Axis", ...).

## Included and deferred
- **Included:** mirroring one instance about X, Y or Z, the instance sync, and the drop to the bed.
- **Deferred:**
  - The rest of the object menu (center, drop, align/distribute, instances, split, the modifiers).
  - The 3D-view right-click menu (the same menu; the object list carries it first).
  - Volume (part), text and SVG mirroring.
  - Multi-selection.
  - Undo snapshots.

## Oracle and RED
- **Oracle.** Native slicing the project the GUI exports for the session (the `plater_session` pattern), plus the instance matrix from the upstream math.
- **Fixture.** `plater_rotation_pivot.rs`'s asymmetric prism (the triangle (0, 0), (20, 0), (10, 16), 10 high) in `orca_cli_ender3.3mf`, so a mirror changes the G-code. The probe before this spec shows full byte parity of Native and the Ares CLI for the prism plain, mirrored in X (`-1 0 0 0 1 0 0 0 1`) and flipped in Z. These are evidence only.
- **RED** (a `plater_session` child module):
  1. Mirror X of the prism. It loads with its box centre (110, 108, 5) as its offset, so the mirror about that centre keeps the offset: diag(-1, 1, 1).
  2. Mirror Z of the prism: diag(1, 1, -1). The flipped prism keeps its z range, so it does not drop: offset z 5.
  3. Mirror Y on `tests/parity/plater-scale/two-instances.3mf`: both instances get diag(1, -1, 1) through the sync.
- **Web E2E** (with the UI commit), `plateMirror.spec.ts`: the context menu's "Along X Axis" on the prism project, then the browser slice against Native slicing the GUI export.
