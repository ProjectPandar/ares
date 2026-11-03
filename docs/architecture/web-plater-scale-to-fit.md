# "Scale to build volume" on the plater session

All `file:line` at OrcaSlicer v2.5.0-beta `345efec64` (`ENABLE_ENHANCED_PRINT_VOLUME_FIT` is 1, `Technologies.hpp:45`). The object menu's "Scale to build volume" ("Scale an object to fit the build volume", `GUI_Factories.cpp:1189-1193`, appended by `create_common_object_menu` `:1732` and the multi-selection transform submenu `:1369`) scales the selection uniformly to the largest size the current plate's free volume takes, and centres it there.

## Upstream boundary
- **`Plater::priv::scale_selection_to_fit_print_volume`** (`Plater.cpp:11092-11112`).
  - `bed_center` is the current plate's `get_center_origin()`.
  - `bed_box` is `PartPlate::get_build_volume(true)` (`PartPlate.cpp:5263-5282`). That is the shared polygon's box of `m_extruder_areas` when the printer has several nozzle areas (`get_shared_poly`), else the plate box. Either way it is widened by `SceneEpsilon` (`BuildVolume.hpp:110`, `EPSILON`) and runs in Z from `-eps` to `height + eps`.
  - With exclusion boxes (`get_exclude_areas`: every four `bed_exclude_area` points, Z `GROUND_Z..m_depth`, `PartPlate.cpp:626-643`), the candidate volumes are `free_print_volumes(bed_box, excludes)` (`Plater.cpp:11049-11090`).
    - Each candidate is a rectangle between two bed or exclusion edges on each axis that overlaps no exclusion.
    - Slivers of width at most `2 * SceneEpsilon` are dropped.
    - Candidates are ordered by the sorted edge indices `i < j`, `k < l`.
  - Otherwise the only candidate is `bed_box`.
- **`Selection::scale_to_fit_print_volume`** (`Selection.cpp:1481-1580`).
  - An empty selection or volume mode returns.
  - It switches on `BuildVolume::type()`:
    - **Rectangle** runs `fit_rectangle` (`:1511-1532`). `box_size` is the selection box plus 0.02 per axis. The best volume has the strictly largest `min(sx, sy, sz)`, every ratio non-zero, and the first such volume wins.
    - **Circle** runs `fit_circle` (`:1534-1569`).
    - **Convex and Custom** do nothing.
  - `fit(s, offset)` (`:1483-1509`) returns for `s <= 0` or `s == 1`. Otherwise:
    1. `scale(s, world | relative | joint)` (the ported `Selection::scale`), then `do_scale` (the ported drop and notify).
    2. `offset.z = -get_bounding_box().min.z`, taken after the scale.
    3. A relative world `translate(offset)` (the ported `Selection::translate`), then `do_move`.

    The offset is `best.center() - bbox.center()`, taken before the scale.
- The bed type is `BuildVolume`'s classification (`BuildVolume.cpp:41-80`, in the constructor `:25-194`), ported as `BedShape` in `project_slice/state/print_volume.rs`.

## Ares destination
- **`PlaterSession::scale_to_fit(object, instance)`** in a new `plater/scale_to_fit.rs`. It acts on a single full instance, the selection the web makes: the selected instance, with the object's other instances synchronized through the ported scale.
  - It ports `free_print_volumes` literally.
  - It ports the `get_build_volume(true)` box over the existing plate box and the arrange's `shared_poly`.
  - It ports `fit_rectangle` and `fit`.
  - The selection bounding box (`get_bounding_box`) is the existing transformed-box merge, which already gives the centre (`selection.rs`), now returned whole.
- **wasm:** `scaleToFit(object, instance)`.
- **Web:** the object menu item "Scale to build volume", with title "Scale an object to fit the build volume".

## Included and deferred
- **Included:** Rectangle beds, with or without exclusion boxes and several nozzle areas; Convex and Custom beds, where nothing happens.
- **Deferred:**
  - `fit_circle`. It needs `smallest_enclosing_circle_welzl` over the volumes' 3D convex hulls. On a Circle bed the session returns `InvalidInput("scale to build volume is not ported for a circular bed")` and the web disables the item, until a follow-up slice ports it.
  - Multi-object selections (the `:1369` submenu).
  - The undo snapshot.

## Oracle and RED
In `plater_scale_to_fit.rs`, a `plater_session` child. Each case checks the session's instance transform against the upstream math, recomputed in-test from the fixture: scale `s`, offset, drop. It also slices the plate like Native slicing the session's GUI export.

The objects are first made tall (the ported Size Z edit, uniform off), so the fit is Z-limited and the slices stay small.
1. **Plain Ender-3.** `orca_cli_ender3.3mf`'s cube at 10 x 10 x 200 mm becomes `s = (h + 2eps) / (200 + 0.02)` uniformly. It is centred at the bed box centre in X and Y and sits on the bed.
2. **Exclusion.** The same, with `bed_exclude_area` over X 150..220. The first free volume is X `-eps..150`, Y `-eps..220`, so the cube centres at about X 75.
3. **Shared nozzle area.** The X2D (`generated-printer-ids/input.3mf`, object moved to (60, 60)) centres on the shared area X 20.5..256, at X 138.25 in its box.
4. **Synchronized instances.** `plater-scale/two-instances.3mf`: fitting instance 0 scales instance 1 by the same factor. Only instance 0 moves to the centre.
5. **Preservation control on a Convex bed** (no parity claim beyond the slice). The Ender-3 with a hexagonal `printable_area` changes nothing.
6. **Circular bed.** The Ender-3 with a 72-gon `printable_area` gets the `InvalidInput` error.

Web E2E `plateScaleToFit.spec.ts`: on case 1's project, right-click the object, choose "Scale to build volume", and plate 1 slices like Native on the exported session state.
