# Plater edits: move, rotate, auto-arrange and multi-plate slicing

Status: spec for review. Branch `web/plate-move-multiplate` (from main
`05e4d6b97`). No implementation yet.

This slice ports the OrcaSlicer 2.4.2 (pinned `8500fcdcc`) plater model
edits: instance move and rotate, the plate list, auto-arrange, and per-plate
slicing. The edits are ports of `libslic3r` (`Model`, `Geometry::Transformation`,
`Arrange`, `deps_src/libnest2d`, NLopt 2.5.0 `sbplx`) and of the GUI-side
model logic that lives in `slic3r/GUI` (`Selection`, `PartPlateList`,
`ArrangeJob`). All of that logic goes into `ares-core`. `ares-wasm` exposes
it as a byte-oriented handle, and TypeScript only renders and forwards pointer
input. Paths below are relative to `OrcaSlicer/src/` unless noted.

## Upstream sources

### Move

| Upstream | Behavior |
| --- | --- |
| `slic3r/GUI/GLCanvas3D.cpp:4438-4453` (`on_mouse`, LeftDown) | A drag starts only on a hovered, selected instance, with the click inside its bbox enlarged by 1 mm; the drag start point is the 3D point under the cursor. |
| `GLCanvas3D.cpp:4459-4500` | Generic view: the new point is the cursor ray intersected with the plane `z = start.z`. Side view (`|forward.z| < EPSILON`): the drag projects onto the camera right/up axes. The selection is translated by `cur - start` (relative). |
| `GLCanvas3D.cpp:4648-4654`, `do_move` `:4910-5001` | On mouse up, `do_move` writes each instance's transformation to the `ModelInstance` (`set_transformation`), calls `Selection::notify_instance_update`, then drops an instance that floats more than `SINKING_Z_THRESHOLD` (`get_instance_min_z` > threshold → translate by `-min_z`). |
| `slic3r/GUI/Selection.cpp:1212` `setup_cache`, `:1253` `Selection::translate` (instance mode), `:1738` `translate(object, instance, displacement)` | Relative world translation of the instance offset. |
| `slic3r/GUI/Gizmos/GLGizmoMove.cpp:91-120` (`GLGizmoMove3D` start/stop/dragging) | Axis-constrained drag along X/Y/Z grabbers; the same `Selection::translate` and `do_move("Tool Move")` (`GLCanvas3D.cpp:3659`). |
| `slic3r/GUI/Gizmos/GizmoObjectManipulation.cpp:107-170` `update_settings_value`, `:309-341` `change_position_value` | The "Position" fields show the instance offset in world coordinates (`get_instance_offset`). Editing one axis translates by `new - cached` and runs `do_move`. |
| `libslic3r/Model.hpp:972-977` `ModelInstance::get_offset/set_offset`, `Model.cpp:1751-1758` `translate_instances/translate_instance`, `:2313` `get_instance_min_z` | The model data that changes. |

### Rotate

| Upstream | Behavior |
| --- | --- |
| `slic3r/GUI/Gizmos/GLGizmoRotate.cpp:102` `GLGizmoRotate::on_dragging`, `:527-640` `GLGizmoRotate3D` mouse/drag/stop | Per-axis ring; the angle follows the cursor around the ring centre, with snapping inside the inner ring. |
| `Selection.cpp:1331` `Selection::rotate` | Instance rotation about the instance (independent) or selection centre. `TransformationType` decides relative/absolute and world/local. |
| `GizmoObjectManipulation.cpp:343-398` `change_rotation_value` (relative, "Rotate (relative)") and `change_absolute_rotation_value` | Numeric rotation; the absolute angles come from `get_rotation_by_quaternion`. |
| `GLCanvas3D.cpp:5026-5100` `do_rotate` | Writes the instance transformation, calls `notify_instance_update`, then drops each instance whose min z changed so it sits on the bed (`auto_drop`; `Selection.cpp:3107` `ensure_on_bed`; `Model.cpp:1721` `ModelObject::ensure_on_bed`). |
| `libslic3r/Geometry.hpp:407-440`, `Geometry.cpp:471-500` `Transformation::get_rotation / get_rotation_by_quaternion / set_rotation` | The instance transform is a single `Transform3d` matrix; offset, rotation and scale are derived from it. |

### Plates and per-plate slicing

| Upstream | Behavior |
| --- | --- |
| `slic3r/GUI/PartPlate.cpp:3915-3962` `compute_origin`, `compute_shape_position`; `:4836-4849` `plate_stride_x/y` (`LOGICAL_PART_PLATE_GAP = 1/5`, `:55`); `PartPlate.hpp:38-48` `compute_colum_count` | Plate `i` sits on a grid (`cols = compute_colum_count(n)`), with origin `(col·stride_x, −row·stride_y)`. |
| `PartPlate.cpp:4407` `create_plate`, `:4554` `delete_plate`, `:4863` `update_plate_cols` | Adding or deleting plates. When the column count changes, every plate moves to its new origin and its instances move with it. |
| `PartPlate.cpp:4978-5036` `find_instance*`, `:5039-5170` `notify_instance_update`, `:2565-2700` `contain_instance / intersect_instance / check_outside` | After a move or rotate, an instance stays on its plate while its convex-hull bbox (`Model.cpp:1623`) still intersects it. Otherwise it goes to the first plate it intersects, else to the unprintable plate. |
| `PartPlate.cpp:2398, 2520` (`m_print->set_plate_origin`); `libslic3r/Print.hpp:1056-1057`; `GCode.cpp:5663-5668, 8127-8140` | A plate slices with world instance positions and the plate origin subtracted at G-code output. |
| `OrcaSlicer.cpp:1464, 1912-1938, 4571-4590, 4630` (`--slice <n>` / `plate_to_slice`; 0 slices all plates) | The CLI parity path: per plate, `plate_<n>.gcode`. |
| `libslic3r/Format/bbs_3mf.cpp:3268, 4404-4565, 4720` (load `plate` / `plater_id` / `model_instance`), `:7918, 8050-8073` (save) | Plate-to-instance map in `Metadata/model_settings.config`. Load is already ported; save is deferred (see below). |

### Auto-arrange

| Upstream | Behavior |
| --- | --- |
| `slic3r/GUI/Jobs/ArrangeJob.cpp:110` `prepare_selected`, `:188` `prepare_all`, `:284` `prepare_wipe_tower`, `:369` `prepare_partplate`, `:435` `prepare`, `:522-608` `process`, `:610-700` `finalize`, `:761-800` `init_arrange_params` | Arrange the selection, all plates, or the current plate. `finalize` maps `bed_idx` to plates, creates plates as needed (`postprocess_bed_index_*`), and applies the translations. |
| `GLCanvas3D.hpp:488-499` `ArrangeSettings`, `GLCanvas3D.cpp:1073-1127` `load_arrange_settings` | GUI defaults: distance 0 (auto), rotation off, multi-materials on the same plate on, avoid the calibration region on, align to the Y axis from `printer_structure` i3. |
| `libslic3r/Arrange.cpp:85-260` `update_arrange_params`, `update_selected_items_inflation`, `update_unselected_items_inflation`, `update_selected_items_axis_align`, `get_shrink_bedpts`; `:281-366` NFP placer config and `AutoArranger`; `:964-1120` `_arrange`, `arrange` | The arrangement itself: `_Nester<NfpPlacer, _FirstFitSelection>`, rotations `{0, π/4, π/2, 3π/4}` when enabled, alignment and starting point, and excluded/non-preferred regions. |
| `PartPlate.cpp:5655-5680` `postprocess_arrange_polygon` | Moves bed index `k` onto plate `k`'s grid cell; `bed_idx = -1` (too large) goes to a new plate. |
| `Model.cpp:3370` `ModelInstance::get_arrange_polygon`; `ArrangePolygon::apply` | The convex-hull footprint of an instance and the write-back of its translation and rotation. |
| `deps_src/libnest2d/include/libnest2d/` (`nester.hpp`, `placers/nfpplacer.hpp`, `selections/firstfit.hpp`, `geometry_traits*.hpp`, `utils/rotcalipers.hpp`, `backends/libslic3r/geometries.hpp`, `optimizers/nlopt/*`) and NLopt 2.5.0 (`deps/NLopt/NLopt.cmake`) `sbplx`/`nldrmd` | The nesting engine and its subplex optimizer. They are ported literally, because the arranged positions must match bit for bit. |
| `OrcaSlicer.cpp:4253, 4684-4800` (CLI `--arrange` with `ArrangeParams`) | The parity path for arrange. |

## Existing Ares scaffold and what changes

- `ares-core` already loads plates and slices one plate:
  - `project/domain/plate_selection.rs` has `Project::select_plate` and `plate_grid_origin`, which ports `compute_colum_count` and the stride.
  - `project/domain/placement.rs` has the `--center` placement, `translate_all_instances` and `drop_instances_to_ground`.
  - `project/load/*` loads the `bbs_3mf` plate metadata.
  - `project_slice/tests/plates.rs` holds the multi-plate parity tests.
  - What is missing: any edit of a loaded project, the `PartPlateList` instance-assignment rules, and arrange.
- `ares-wasm` slices from bytes on every call (`sliceProjectPlate(bytes, plate, overrides)`, `sliceWithProfiles(bytes, …, {plate, placement})`). `loadScene` returns instance matrices and plates.
- In the web app:
  - The viewport draws one bed and supports picking only; `Viewport.ts:303` `#pick` selects an object.
  - `actions.slicePlate` always slices `scene.plates[0]`, and Preview shows plate 1.
- This slice keeps loading, `select_plate`, plate origins and the slice pipeline unchanged. It adds an editable plater session in core that the slice entries read instead of re-parsing bytes. The "first plate only" web flow becomes a current-plate / slice-all flow.

## Design

### Core: `crates/ares-core/src/plater/` (new module)

`PlaterSession` owns the loaded `Project` (3MF or STL, through the existing
loaders) plus a `PartPlateList` port: plate count, columns, origins, the
per-plate instance sets and the unprintable plate. Its operations are each
ports of the upstream functions named above:

- `translate_instance(object, instance, displacement)`: `Selection::translate` (instance mode, relative), then `do_move` (drop above the threshold, `notify_instance_update`).
- `set_instance_position(object, instance, axis, value)`: `change_position_value`.
- `rotate_instance(object, instance, rotation, kind)`: `Selection::rotate` with the `TransformationType` the caller used (gizmo ring: relative world, independent; fields: relative or absolute), then `do_rotate` (drop to bed).
- `add_plate()` / `delete_plate(index)`: `create_plate` / `delete_plate` / `update_plate_cols`, including moving instances when the column count changes.
- `arrange(scope, params)`: `ArrangeJob::prepare_*` → `arrangement::arrange` → `finalize`. Its params are built like `init_arrange_params`, with the GUI defaults above as defaults.
- `scene()`: the current instance matrices and plate list, in the existing `SceneData` shape plus plate origins.
- Slicing: the existing `slice_project_plate_validated` / profile entries take a project view from the session (`select_plate` on the edited project, plate origin from the session's plate list) instead of bytes. Output is unchanged.

`Transform3d` stays the instance representation; offset and rotation are read
and written the way `Geometry::Transformation` does.

The arrange port lives in `crates/ares-core/src/arrange/`:
- `nest2d/`: the geometry traits, NFP, the NFP placer, first-fit selection and rotating calipers;
- `nlopt_sbplx.rs`: NLopt 2.5.0 `sbplx` + `nldrmd`;
- `arrange.rs`: `Arrange.cpp`.

It reuses the existing Clipper/ExPolygon ports in core. The nester's
`parallel` evaluation is ported as a deterministic sequential loop that
gives the same result: upstream reduces candidates by best score, and ties
resolve by index (to be confirmed in the RED step).

### WASM and JS

- `crates/ares-wasm/src/plater.rs` adds a `PlaterSession` wasm-bindgen handle with: `load(bytes, filename)`, `scene()`, `translate`, `setPosition`, `rotate`, `addPlate`, `deletePlate`, `arrange(paramsJson)`, `sliceProcessed(plate, overridesJson)`, and `sliceWithProfilesProcessed(plate, catalog, selection, options)`.
- The worker holds sessions by id, the same pattern as `PresetCatalog`.
- `@ares-slicer/wasm` exports a `PlaterSession` class.

### Web

- `viewport/`: the drag interaction of `GLCanvas3D::on_mouse`. The view computes the cursor-ray / `z = start` plane intersection, which is rendering math, and sends the displacement to the session on each move. It also draws a move gizmo (X/Y/Z arrows), a rotate gizmo (three rings), and every plate of the plate list, each with its index label.
- `sidebar/ObjectManipulation.tsx` adds the "Object Operations" panel: Position X/Y/Z (world), Rotate (relative), and the absolute rotation, as in `GizmoObjectManipulation`.
- Plate bar: select the current plate, "Add plate", "Delete plate", "Arrange all" / "Arrange current plate", and the arrange settings popup with spacing, rotation and align to Y.
- Slicing:
  - "Slice plate" slices the current plate.
  - "Slice all" slices every plate in turn, as the CLI's `--slice 0` does.
  - Export saves `<name>_plate_<n>.gcode` for the current plate, or every plate as separate downloads.
  - Preview shows the current plate's G-code.
- State: `state/plater.ts` holds the session id, current plate, selected instance and per-plate slice results. All geometry comes from `session.scene()`.

## How a moved position reaches slicing

Edits change the session's `Project` instance transforms in world (plate-grid)
coordinates and update the plate instance sets, exactly as `ModelInstance` and
`PartPlateList` change in the GUI. Slicing plate `n` takes `select_plate(n)` of
the edited project, with the origin from the session's plate list. This is the
same code path a 3MF holding those transforms and that plate map already takes.
So a 3MF with the same transforms and `model_settings.config` plate map is the
parity input for OrcaSlicer.

## Included / deferred

Included:
- move by drag (bed plane and side view), by the move gizmo and by the Position fields, with drop-to-bed;
- rotate by gizmo rings and fields (relative and absolute), with drop-to-bed after rotation;
- plate grid, add plate and delete plate, and reassigning instances to plates after edits;
- per-plate slice and slice-all;
- arrange of all plates, the current plate, or the selection, with the params:
  - spacing (auto or explicit);
  - rotation enabled;
  - align to the Y axis;
  - best_object_pos alignment;
  - by-object sequential clearance;
  - multi-material grouping;
  - avoid the calibration region;
  - bed exclude areas;
  - wipe tower as an obstacle.

Deferred:
- scale and size fields;
- mirror;
- lay on face;
- cut;
- auto-orient (`Orient.cpp`);
- instance copies (`duplicate`);
- plate locking and plate-specific settings edits;
- per-plate bed type and filament map;
- the sequential-print collision warnings shown while dragging (`update_sequential_clearance`);
- snapping to the grid (Ctrl);
- undo/redo snapshots;
- volume-mode (part) moves;
- multi-selection by rectangle;
- the assemble view;
- 3MF project save (`bbs_3mf` export) and project persistence of edits;
- SLA arrange.

Scale stays deferred. Its upstream path (`change_scale_value` / `change_size_value`, uniform and non-uniform scaling, scaled drop-to-bed) is not trivial.

## Parity and tests

Native parity uses both legs: official (`scripts/orca-parity.sh`,
`ORCA_APPDIR`) and seeded (`scripts/orca-seeded-parity.sh`,
`ORCA_REPLAY_BIN`/`ORCA_REPLAY_SEED=0`/`ORCA_REPLAY_CPU`; the web slices with
seed 0). Every comparison is the normalized full G-code of each plate. Native
outputs are generated per run under `target/web-e2e/`, never committed.

**Move / rotate oracle.** The E2E edits through the UI with deterministic
inputs: the Position and Rotation fields, and a drag replayed at fixed screen
points. The resulting transforms are then read from `session.scene()`.

The oracle 3MF is the fixture with those transforms written into
`3D/3dmodel.model` build items and the plate map into
`Metadata/model_settings.config`. OrcaSlicer slices it with `--slice <n>`. The
test derives the expected transforms independently from the upstream
formulas:
- offset change;
- `Transformation` rotation composition;
- drop by the min z of the transformed mesh vertices read from the fixture;
- plate assignment by bbox-plate intersection.

The test asserts that the derived transforms equal the session's.

**Arrange oracle.** OrcaSlicer CLI `--arrange 1` on the same input, with the
CLI flags set to the web's params (`allow_rotations`,
`avoid_extrusion_cali_region`; spacing auto). The arranged per-plate G-code is
compared on both legs. The E2E also asserts that the arranged instance
transforms and plate count equal OrcaSlicer's, read from the 3MF that the
same run writes with `--export-3mf` (CLI option, `PrintConfig.cpp:10611`;
`--arrange` at `:10780`).

Web E2E specs (system Chrome, screenshots and `result.json` per spec):

1. `plateMove.spec.ts`, on `tests/parity/orca_cli_ender3.3mf`:
   - drag the object, check the position fields and the moved mesh in the viewport;
   - set X/Y in the fields;
   - check the drop to bed after a Z raise;
   - slice and compare with the oracle 3MF on both legs.
2. `plateRotate.spec.ts`, on the same fixture:
   - rotate Z 45° by the ring and X 90° by the field;
   - drop to bed;
   - slice and compare on both legs.
3. `multiPlate.spec.ts`, on `tests/parity/custom-gcode-overview/two-plates.3mf` (Ender-3, two plates):
   - both plates render on the grid;
   - slice each plate, then slice all;
   - move an object from plate 1 to plate 2, so it is reassigned;
   - add and delete a plate, with a column change moving the instances;
   - per-plate G-code is compared on both legs.
4. `arrange.spec.ts`:
   - Arrange all on a multi-object project that overflows one plate, so arrange creates plate 2;
   - Arrange all with rotation enabled;
   - compare with CLI `--arrange` on both legs.
5. Regression: the existing specs (`slice`, `presetSlice`, `persistence`, `dualExtruder`, the mobile specs) still pass. The plate bar and gizmos get 44 px touch targets.

Rust: the core plater and arrange ports follow the repo TDD rules:
- a RED parity test per behavior in `crates/ares-cli/tests/orca_parity/` (official and seeded), using input-only fixtures;
- the full ares-core suite;
- the dynamic-value guards;
- fmt and clippy `--workspace --all-targets -D warnings`.

## Commit plan

1. `docs`: this spec.
2. `feat(core)`: `PlaterSession` with transforms (translate, position, rotate, drop to bed), the `PartPlateList` port (grid, add/delete plate, column change, `notify_instance_update`), and slicing from the session. Native parity tests come first.
3. `feat(wasm)`: the `PlaterSession` binding and the JS client.
4. `feat(web)`: plates on the canvas, the plate bar, current-plate and slice-all slicing, per-plate preview and export. E2E 3.
5. `feat(web)`: drag move, the move gizmo and the Position fields. E2E 1.
6. `feat(web)`: the rotate gizmo and rotation fields. E2E 2.
7. `feat(core)`: the NLopt sbplx and libnest2d port with `Arrange.cpp`. Native parity on CLI `--arrange`.
8. `feat(core+wasm+web)`: ArrangeJob prepare/finalize, the arrange button and settings. E2E 4.

Every file stays at 400 lines or fewer. The arrange port is the largest piece:
libnest2d is about 8.5k lines of headers, with only the used placer,
selection and traits ported, plus about 0.8k lines of NLopt.

## Decisions (coordinator review)

- D1 (Q1). Arrange (commits 7-8) is a separate follow-up PR. It is not started
  until the upstream version arrange should track is decided (the 2.4.2 oracle
  or later Orca main changes, e.g. `4c85d0e80`, IMEXArrange, exclusion volumes).
  This PR is commits 1-6.
- D2 (Q2). The web keeps the GUI arrange defaults (`GLCanvas3D.hpp:488-499`):
  distance 0, rotation off, `avoid_extrusion_cali_region` on. The CLI defaults
  differ: `allow_rotations` true (`OrcaSlicer.cpp:1439, 1486-1488`) and
  `avoid_extrusion_cali_region` false (`:1439`). So the arrange E2E passes
  CLI flags equal to the web values.
- D3 (Q3). The Position fields show world coordinates, as upstream does.
- Conditions:
  - every core commit starts with a Native parity RED;
  - the derived-transform check in the E2E imports no Ares code;
  - files stay at 400 lines or fewer;
  - Native runs are serial on CPUs 4-7;
  - the full ares-core suite runs whenever core changes.

## Auto-arrange on OrcaSlicer v2.5.0-beta (supersedes the 2.4.2 arrange design)

Coordinator decision: auto-arrange ports OrcaSlicer **v2.5.0-beta at
`345efec64c7b95a05ce5cf96df0dde66a158835c`** (release/v2.5.0), not 2.4.2 and
not main. This section replaces the arrange parts above: "Upstream sources >
Auto-arrange", the arrange port under "Design", the arrange oracle under
"Parity and tests", D2, and commits 7-8. Every `file:line` below is at
`345efec64`. Arrange ships as its own PR, stacked on the move/rotate/plates
branch. The move/rotate/plates work stays on 2.4.2 parity until the Ares
baseline migration to v2.5.0-beta lands.

### Upstream sources (at 345efec64)

| Upstream | What it does |
| --- | --- |
| `src/libslic3r/Arrange.hpp:58-124` `ArrangePolygon`, `:128-205` `ArrangeParams` | Arrange input and output, including the new exclusion fields `z_min`/`z_max`/`has_z_range`/`is_bed_exclusion`/`bed_exclusion_extruder_ids`/`bed_exclusion_extruder_id` and the belt fields. |
| `src/libslic3r/Arrange.cpp:108-146` `z_ranges_overlap`, `bed_exclusion_applies`, `has_bed_exclusion_regions` | Exclusion volumes: a fixed bed-exclusion item applies only when the Z ranges overlap (touching counts) and only to its owning physical nozzle. |
| `Arrange.cpp:148` `update_arrange_params`, `:163` `update_selected_items_inflation`, `:213` `update_unselected_items_inflation`, `:234` `update_selected_items_axis_align`, `:339` `get_shrink_bedpts` | Parameter and inflation setup: auto spacing, sequential-print clearance, brim/skirt, align to Y. |
| `Arrange.cpp:356` `fill_config`, `:416-452` overfit scores, `:455-1003` `AutoArranger`, `:1005/1034/1063` objective functions per bed type, `:1092` `remove_large_items`, `:1105` `min_area_boundingbox_rotation`, `:1119` `fit_into_box_rotation`, `:1125` `_arrange`, `:1221` `process_arrangeable`, `:1259` `call_with_bed`, `:1280-1316` `arrange` | The packing kernel. |
| `deps_src/libnest2d/include/libnest2d/`: `placers/nfpplacer.hpp`, `selections/firstfit.hpp`, `geometry_traits.hpp`, `geometry_traits_nfp.hpp`, `nester.hpp`, `utils/rotcalipers.hpp`, `optimizers/nlopt/subplex.hpp`, `parallel.hpp`; `backends/libslic3r/geometries.hpp` | Only the placer, selection, NFP and traits that `Arrange.cpp` instantiates. |
| NLopt **2.5.0** (`deps/NLopt/NLopt.cmake`, tarball `v2.5.0`): `sbplx` + `nldrmd` | The subplex optimizer `NfpPlacer` uses. |
| `src/libslic3r/IMEXArrange.hpp:15-101`, `IMEXArrange.cpp:33-220` `imex_arrange_zones`, `ImexArrangeInput::read_config`, `ImexArranger` (`bed_shape`, `add_keep_outs`, `finish`) | IDEX/IQEX plates stay inside their primary zone and out of the carriage collision strips. |
| `src/libslic3r/IMEXZones.hpp:25` `ImexZoneLayout`, plus the `PartPlate::imex_primary_zone`/`imex_collision_zones` it feeds | Zone layout per plate. |
| `src/slic3r/GUI/Jobs/ArrangeJob.cpp:53-104` bed-exclusion extruder mapping, `:156-175` wipe tower polygon, `:249` `prepare_arrange_polygon`, `:259` `prepare_selected`, `:342` `prepare_all`, `:416-531` `estimate_wipe_tower_info`/`prepare_wipe_tower`, `:532` `prepare_partplate`, `:605-630` `prepare_imex_zones`, `:632` `prepare`, `:697` `check_unprintable`, `:722-805` `process`, `:819-965` `finalize`, `:989-1043` `init_arrange_params` | The GUI arrange job that the web arrange buttons run. |
| `src/slic3r/GUI/PartPlate.cpp:7617` `preprocess_arrange_polygon`, `:7663` `…_other_locked`, `:7692` `preprocess_exclude_areas`, `:7799` `preprocess_nonprefered_areas`, `:7913` `postprocess_bed_index_for_current_plate`, `:7931` `postprocess_arrange_polygon` | Plate list input and output; arranged items past the last plate create plates. |
| `src/slic3r/GUI/GLCanvas3D.hpp:603-614` `ArrangeSettings`; `GLCanvas3D.cpp:1549-1557` loaded settings, `:6679-6770` arrange menu | GUI defaults and the settings popup: spacing, auto-rotate, multi-material, calibration region, align to Y (off while rotation is on; default from `printer_structure == i3`, `:6763`). |
| `src/OrcaSlicer.cpp:5900-6040` CLI arrange (`--arrange`, `PrintConfig.cpp:12962`; `--allow_rotations` `:13231`, `--align_to_y_axis`, `--avoid_extrusion_cali_region` `:13242`; `--export-3mf` `:12763`) | Native oracle path. |
| Upstream tests: `tests/libslic3r/test_arrange.cpp`, `test_imex_arrange.cpp` (287 LOC), `test_exclusion_volumes.cpp`, `tests/cli/test_cli_arrange_rotations.sh` | Literal expectations to port as Ares RED cases. |

### GUI versus CLI at 345efec64

Both paths call the same `update_*` / `get_shrink_bedpts` /
`preprocess_exclude_areas` / `arrangement::arrange` sequence. They differ as
follows:

- **Defaults:** the CLI defaults are `allow_rotations` false (changed from
  2.4.2), `avoid_extrusion_cali_region` false, `min_obj_distance` 0 (auto), and
  `align_to_y_axis` from `printer_structure` when not given
  (`OrcaSlicer.cpp:5978-5982`). The GUI defaults (`GLCanvas3D.hpp:603-614`)
  are distance 0, rotation off, cali region on (forced off when the checkbox is
  hidden, `GLCanvas3D.cpp:6726-6734`), and align to Y from `printer_structure`.
  The E2E passes CLI flags equal to the web's settings (D2 stands).
- **GUI-only steps:** `ImexArranger` (`ArrangeJob.cpp:744-746, 776`), the
  per-nozzle bed-exclusion extruder mapping (`:53-104`), the selection and
  current-plate scopes, and `preprocess_nonprefered_areas` for BBL
  `scan_first_layer` (`:733-734`). The CLI does not exercise any of these.

### Rust destination

**`crates/ares-core/src/arrange/`:**
- `arrange.rs`: `Arrange.cpp`.
- `params.rs`: the `update_*` functions and `get_shrink_bedpts`.
- `nest2d/`: the listed libnest2d subset.
- `nlopt/`: sbplx and nldrmd 2.5.0.
- `imex.rs`: `IMEXArrange.cpp` plus the `IMEXZones` layout it reads.
- The public API is upstream's: `arrangement::arrange(items, excludes, bed, params)` with `ArrangePolygon` and `ArrangeParams`.

**`crates/ares-core/src/plater/arrange_job.rs`:**
- ArrangeJob `prepare*`, `process`, `finalize` and `init_arrange_params`, plus the PartPlateList pre/post-processing.
- Exposed as `PlaterSession::arrange(scope, settings)`, with `scope` all or current plate. The GUI arranges no selection; see A3c.

**ares-wasm:** `PlaterSession.arrange(scope, settingsJson)` returns the layout.

**Web:** the "Arrange all" and "Arrange current plate" buttons and the settings popup of `GLCanvas3D.cpp:6679-6770`.

The parallel candidate evaluation becomes a deterministic sequential loop that keeps libnest2d's reduction order.

### Included / deferred

**Included:**
- the whole kernel, with box, circle and polygon beds;
- auto spacing and fixed spacing;
- rotations: `fit_into_box_rotation`, `min_area_boundingbox_rotation`, and allowed angles;
- align to Y;
- `best_object_pos` alignment;
- sequential-print clearance and bed shrink;
- multi-material and temperature separation (`extrude_ids`, bed and print temperatures, vitrify);
- the wipe tower as a fixed item;
- the extrusion-calibration region;
- `bed_exclude_area` and exclusion volumes with Z ranges and per-nozzle ownership;
- IDEX/IQEX zones (`ImexArranger`);
- the all and current-plate scopes;
- unarrangeable items going to the unprintable plate;
- new plates for overflow.

**Deferred:**
- **Belt printers** (`prepare_belt_regions` `:176-225` and the `is_belt` params): Ares has no belt slicing to verify against.
- **Wrapping-detection exclude areas** (`enable_wrapping_detection`).
- **Locked plates:** plate lock is deferred, so no plate is locked.
- **Not ported:** duplicate/fill-bed (`ModelArrange.cpp:77-106`), auto-orient, SLA arrange, the assemble-view arrange, and the CLI-only assemble path.

**Dependency:** the v2.5.0-beta `PrintConfig` keys must exist in Ares first:
- `bed_exclude_volumes` (`PrintConfig.cpp:1063`), `bed_exclude_volume_mode` (`:1073`) and `extruder_bed_exclude_volumes` (`:1083`);
- `imex_*` (`:7035-7106`).

These come from the baseline migration, not from this PR.

### Parity oracle

1. **End-to-end Native: official v2.5.0-beta AppImage.**
   - **Where it comes from:** the extraction is named by `ORCA_APPDIR` and run through `scripts/orca-parity.sh`, as the 2.4.2 official leg is.
   - **Run:** each case runs `--arrange 1 --allow_rotations <b> --avoid_extrusion_cali_region <b> --align_to_y_axis <b> --export-3mf arranged.3mf <input.3mf>`, pinned to one CPU like the existing parity runs.
   - **Compared output:** Ares `PlaterSession::arrange(All, settings)` must produce the same build item transforms (as strings, the way `bbs_3mf` writes them) and the same `model_settings.config` plate map.
   - **Cases:** overflow onto new plates; rotation on and off; align to Y on an i3 printer; by-object clearance; a multi-material temperature split; `bed_exclude_area`; exclusion volumes with Z ranges; a circular (delta) bed; the wipe tower.
   - **Not compared yet:** per-plate G-code waits until Ares slices at v2.5.0-beta parity. Arrange itself does not slice.
2. **Kernel oracle for the GUI-only paths** (IMEX, per-nozzle exclusions, partial scopes): `arrange-kernel`, a small driver linked against `libslic3r` at `345efec64`.
   - **Driver:** it reads `ArrangePolygons`/`ArrangeParams`/bed as JSON and prints `translation`/`rotation`/`bed_idx` after `arrangement::arrange`, optionally with `ImexArranger`, single-threaded.
   - **Source in the repo:** the driver source and its nix recipe live in `tools/orca-probes/arrange-kernel/`, like the other patched probes.
   - **Build:** the recipe builds from the pinned `345efec64` tarball and is added as an `arrange-kernel` variant of `scripts/ci/build-oracle.sh`, whose source revision becomes per variant.
   - **Tests:** the binary is named by `ORCA_ARRANGE_KERNEL_BIN`, and missing oracles follow `oracleEnv`/`reference_enabled`. The Ares tests call the public `arrangement::arrange` with literal inputs, and the expected outputs are the driver's recorded bytes.
   - **Kept out of git:** the built binary and the recorded Native outputs (no committed Native artifacts).
3. **Literal upstream cases:** the assertions of `test_imex_arrange.cpp`, `test_arrange.cpp`, `test_exclusion_volumes.cpp` (arrange parts) and `test_cli_arrange_rotations.sh`, ported as Ares RED tests against the public API.
4. **Property tests (proptest)** through `arrangement::arrange`:
   - arranged items do not overlap closer than the spacing;
   - items stay inside the shrunk bed;
   - the same input gives the same output.

There is no seeded leg: arrange has no random stream. Tie order comes from running single-threaded, matched by the sequential port.

### Commit plan (arrange PR)

| # | Commit | RED evidence |
| --- | --- | --- |
| A1 | Kernel: NLopt sbplx 2.5.0, the libnest2d subset, `Arrange.cpp` | Kernel-oracle and literal cases; proptest |
| A2 | `IMEXArrange` + zones, after the migration adds `imex_*` | Kernel-oracle and literal `test_imex_arrange` cases |
| A3 | ArrangeJob and the PartPlateList pre/post-processing on `PlaterSession` | Official v2.5.0-beta CLI end-to-end cases |
| A4 | wasm + web arrange buttons and settings | `arrange.spec.ts` against the same CLI oracle |

### Kernel port boundaries (A1, coordinator review)

Where upstream has no defined behaviour, or a precision Rust does not offer, the port does the following:

- **`fixed_bins` read past its end (undefined behaviour upstream).**
  - Where: `_FirstFitSelection::packItems` (`firstfit.hpp:169-180`).
  - When: an item does not fit even after its retry with only the excluded regions, and the last bed is still empty.
  - What upstream does: it preloads `fixed_bins[placers.size() - 1]`. When that bed has no fixed items, this indexes past the end of `fixed_bins`.
  - Port: preloads nothing on that bed (`nest/selection.rs`, `retry_with_excluded`).
  - The other `fixed_bins` reads in the same function are bounds-guarded upstream and are ported as-is.
- **`boost::bad_rational` in the rotating calipers.**
  - When: dividing by an edge's squared length throws when the contour has a zero-length edge.
  - `min_area_boundingbox_rotation` (`Arrange.cpp:1105-1116`): upstream catches the throw and returns rotation 0. The port does the same.
  - `fit_into_box_rotation` (`Arrange.cpp:1118-1122`): upstream has no catch, so the exception leaves `arrange`. The port instead stops the caliper walk and bisects the angle interval found so far.
- **`long double` versus `f64`.**
  - Upstream: `RotatedBox::width`/`height` (`rotcalipers.hpp:27-33`) are `long double`, which is 80-bit x87 on x86-64 Linux.
  - Port: computes them in `f64`.
  - Only `fitIntoBoxRotation`'s aspect-ratio comparison against the bed reads them. `minAreaBoundingBox` compares exact rationals and is not affected.
  - Residual risk: when a box's aspect ratio is within `f64` rounding of the bed's, the bisection may start from the neighbouring caliper interval.

### A3 slice plan: ArrangeJob on `PlaterSession`

A3 ports the GUI arrange job, which the web buttons run, onto the existing session. It ships in three source-cited slices. Each slice is RED first, then GREEN. Every `file:line` is at `345efec64`.

**A3a: whole-scene arrange (`ArrangeScope::All`), no wipe tower.**
- Upstream:
  - `ArrangeJob::prepare_all` (`ArrangeJob.cpp:342-414`) and `prepare_arrange_polygon` (`:249-257`);
  - `get_arrange_poly` and `get_instance_arrange_poly` (`ModelArrange.cpp:107-202`), `ModelInstance::get_arrange_polygon` (`Model.cpp:3920-3971`) and `ModelObject::convex_hull_2d` (`:1819`);
  - `check_unprintable` (`ArrangeJob.cpp:697-720`);
  - `process` (`:722-805`), without IMEX and without the BBL `scan_first_layer` non-preferred areas;
  - `finalize` (`:819-965`) and `init_arrange_params` (`:989-1043`), with `GLCanvas3D::ArrangeSettings` (`GLCanvas3D.hpp:603-614`) as the settings;
  - the `PartPlateList` steps: `preprocess_arrange_polygon` (`PartPlate.cpp:7617`), `preprocess_exclude_areas` (`:7692`), `postprocess_bed_index_for_selected`/`_unselected` (`:7833`, `:7882`), `postprocess_arrange_polygon` (`:7931`), `clear` (`:6454`) and `rebuild_plates_after_arrangement(false, true)` (`:8382`);
  - `ModelInstance::apply_arrange_result` (`Model.hpp:1531-1538`).
- Destination:
  - `crates/ares-core/src/plater/arrange_job/` (prepare, process, finalize, params);
  - plate-list pre/post-processing next to the existing `plater/plate_list.rs`.
- Public API: `PlaterSession::arrange(ArrangeScope::All, ArrangeSettings)`. It reuses `arrangement::arrange` from A1.
- Existing scaffold: the session's plate list and instance transforms are reused as `PartPlateList` and `ModelInstance`. No Ares-owned arrange step is added.
- Deferred: the wipe tower (`prepare_wipe_tower`, A3b), locked plates (no plate lock in Ares), IMEX and exclusion volumes (A2, after the migration), belts.

**A3b: wipe tower.**
- Upstream:
  - `ArrangeJob::prepare_wipe_tower` (`:442-531`) and `estimate_wipe_tower_info` (`:416-440`);
  - `PartPlate::estimate_wipe_tower_polygon` and the v2.5.0-beta tower footprint estimate it calls.
- Scoped by its own spec addendum once A3a lands, because the footprint estimate is a separate upstream boundary.

**A3b addendum: the wipe tower in "Arrange all".** Every `file:line` is at `345efec64`.

This slice ports the prime tower items `prepare_wipe_tower` adds to the fixed items, and replaces A3a's rejection of projects that need a tower. It stacks on A3a GREEN. That GREEN waits for the Clipper2 `ClipperUtils` port (see A3a above), so A3b's GREEN does too.

Upstream, in the order the job reaches it:

1. **`ArrangeJob::prepare_wipe_tower`** (`ArrangeJob.cpp:442-528`).
   - No tower when the prime tower is disabled, the print is by object, or the printer is a belt printer.
   - Otherwise a tower is needed for a smooth timelapse, an item with several filaments, or several filaments sharing a bed temperature while multi-material plates are allowed.
   - For each of the 36 beds:
     - a bed past the plate count reuses the last plate's settings;
     - a plate that already shows a tower gets that tower (`get_wipe_tower` -> `get_wipetower_arrange_poly`, `:156-167`);
     - otherwise, when a tower is needed, it gets `estimate_wipe_tower_info` (`:416-430`), sized for the extruders of all plates (`PartPlateList::get_extruders(true)`, `PartPlate.cpp:6943-6954`).
   - Locked plates are skipped (`bedid_unlocked`); Ares has none.
2. **The tower a plate already shows.**
   - `GLCanvas3D::reload_scene` (`GLCanvas3D.cpp:3564-3663`) adds a tower volume per plate:
     - only when the prime tower is enabled, and only when two or more filament colours exist or a tower is needed (`need_wipe_tower`);
     - a by-object plate gets one only with exactly one printable instance;
     - a plate with no objects or a zero estimated depth gets none.
   - The stored `wipe_tower_x/y` is re-clamped first, with relocation (`estimate_wipe_tower_polygon(..., &relocation)` -> `nearest_clear_wipe_tower_anchor`, `GCode/WipeTowerEstimate.cpp:247-285`; obstacles from `PartPlate::wipe_tower_obstacles`, `PartPlate.cpp:4036-4063`).
   - The volume is the preview cube plus its brim (`GLVolumeCollection::load_wipe_tower_preview`, `3DScene.cpp:987-1052`).
   - `GLCanvas3D::get_wipe_tower_info` (`GLCanvas3D.cpp:6149-6185`) turns that volume into the item:
     - the volume's box, offset by the brim twice;
     - placed at the stored position, clamped onto the plate;
     - a fixed virtual item with `++priority`.
   - The session models this state from the project; it has no canvas.
3. **`PartPlate::estimate_wipe_tower_polygon`** (`PartPlate.cpp:4065-4155`): the footprint clamped onto the plate, by `WIPE_TOWER_MARGIN` and the brim, or re-placed with the comfort margin. The arrange job calls it without relocation.
4. **`PartPlate::estimate_wipe_tower_footprint`** (`PartPlate.cpp:3982-4034`):
   - the plate's filaments (`PartPlate::get_extruders(true, config, config)`, `:3075-3250`), padded to the requested count and including `wipe_tower_filament`;
   - the tallest instance and the thinnest object layer height over the plate's instances (`contain_instance_totally`, `:4473-4521`).
5. **`GCode/WipeTowerEstimate.cpp`**:
   - `resolve_wipe_tower_type` (`:62-73`), `estimate_wipe_tower_first_layer_outline` (`:75-84`) and `estimate_wipe_tower_footprint` (`:86-245`);
   - with the statics they call: `WipeTower::get_limit_depth_by_height`, `get_auto_brim_by_height`, `estimate_brim_real_width`, `get_wrapping_detection_depth`, `estimate_tower_blocks_depth`, `rib_footprint_side` and `estimate_rib_tower_bbox_side` (`GCode/WipeTower.cpp:1697-1824`); `WipeTower2::cone_base_polygon` and `estimate_semm_flush_volume` (`GCode/WipeTower2.cpp:2167-2245`); and the mixed-filament expansion `has_any_mixed_filament` / `expand_mixed_filaments` (`FilamentMixer.cpp:486-538`).
6. **`PartPlateList::set_default_wipe_tower_pos_for_plate(new_index, true)`** (`PartPlate.cpp:6339-6428`), which GUI-mode `create_plate` calls (`:6646`): the tower position of a plate the arrange adds.

Destination:
- `plater/arrange_job/wipe_tower.rs`: steps 1 and 2.
- `plater/plate_list/wipe_tower.rs`: steps 3, 4 and 6, plus the obstacles and relocation of step 2.
- `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/estimate.rs`: step 5. Ares' existing 2.4.2 wipe tower geometry is not reused for these beta statics. If the beta `WipeTower` migration ports any of them first, A3b reuses those.

Dependencies and deferrals:
- **Config keys.** The estimate reads keys that must exist in Ares' typed settings at the beta baseline: `wipe_tower_type`, `wipe_tower_wall_type`, `wipe_tower_cone_angle`, `wipe_tower_rib_width`, `wipe_tower_extra_rib_length`, `prime_volume_mode`, `filament_prime_volume`, `filament_adhesiveness_category`, `filament_change_length`, `filament_is_mixed` and `filament_mixed_components`. Any still missing come from the baseline migration, not from this slice.
- **A sliced plate's real tower mesh** (`load_real_wipe_tower_preview`, after `psWipeTower`) is deferred. A session keeps no per-plate `Print`, so a plate's tower is always the estimate.
- **Belt purge prisms and IMEX** are deferred as in A3a.

Oracle: the CLI global arrange (`OrcaSlicer.cpp:5729-5810`), compared as in A3a. The CLI and the GUI job differ here, and the A3b cases keep to the path they share:
- **Which tower.**
  - The CLI estimates a tower for every bed whose project plate prints one: prime tower enabled, not by object unless smooth timelapse, and two or more filaments or a smooth timelapse (`plate_needs_wipe_tower`).
  - The GUI uses the tower the plate shows.
  - The cases use valid stored tower positions, so neither path re-places them.
  - The cases use `prime_tower_brim_width = 0`. The GUI's twice-offset volume box then equals the CLI's polygon in millimetres, but not always in scaled units, because the two paths round differently:
    - The GUI item is the shown volume's `float` box scaled in `double`, plus the scaled stored position, with `++priority` (`ArrangeJob.cpp:122-145`, `GLCanvas3D.cpp:6149-6185`).
    - The CLI item is `estimate_wipe_tower_polygon`'s absolute polygon, each corner a `float` sum scaled in `float`, with priority 0 (`PartPlate.cpp:4139-4152`).
    - For example, a 32.0841 mm tower at x = 150 has its far edge at 182084110 in the GUI and at 182084112 in the CLI.
  - The port follows the GUI. The difference only decides a result where two placements score exactly the same: libnest2d keeps the first of equal scores (`std::min_element` and the strict `<` in `nfpplacer.hpp` `_trypack`), and a tower on the bed diagonal makes mirror placements tie.
  - The tower cases therefore use a rectangle-wall tower of `prime_tower_width` 20 mm that purges the prime volume (`purge_in_prime_tower = 0`), off the diagonal, at sizes and positions where both paths give the same scaled corners: 20 x 11.25 mm at (1, 1) for two filaments, and 20 x 12.6 mm at (120, 95) for the smooth timelapse. No placement mirrors another: the second object, and in the rotated case the turned one, is 15 x 10 mm.
  - Each tower case gives the same Native result on two fresh runs, fails when the port drops the tower items, and places every object with a score at least 8.7e-4 below the next-best placement.
- **Footprint inputs.**
  - The CLI sizes with all objects (`use_global_objects = true`) and the plate's filament count; the GUI preview sizes with the plate's own objects.
  - The cases put all objects on the plate the tower is for, or use objects of equal height.
- **Overflow beds.**
  - Both estimate on the last plate's settings: the CLI with the largest plate filament count, the GUI with every plate's extruders.
  - The overflow case uses one filament set across plates.

A3b cases:
1. A two-colour object beside the corner tower.
2. Two single-colour objects in different filaments sharing a bed temperature, with multi-material plates allowed, beside the corner tower.
3. The same as case 2 with rotation.
4. Overflow onto a new plate with two colours and the corner tower.
5. A smooth timelapse with one filament, the tower where the centred cube would be.
6. A by-object print with one object.
7. The prime tower disabled (control).

**A3c addendum: "Arrange" on the current plate.** Every `file:line` is at `345efec64`.

- **Selection scope: not ported.** `ArrangeJob::prepare_selected` (`ArrangeJob.cpp:259-340`) is defined but nothing calls it: `ArrangeJob::prepare` (`:650-657`) runs `prepare_all` for `PREPARE_STATE_DEFAULT` and `prepare_partplate` for `PREPARE_STATE_MENU`.
  - The toolbar Arrange (`GLCanvas3D.cpp:6751`) and arrange-after-import (`GUI_Factories.cpp:668`) set DEFAULT.
  - The plate context-menu "Arrange current plate" (`GUI_Factories.cpp:1993-1999`) sets MENU.
  - A selection-only arrange would be an Ares design with no upstream behaviour, so `ArrangeScope` gets only `All` and `CurrentPlate(index)`.
- **Upstream:**
  - `prepare_partplate` (`ArrangeJob.cpp:532-599`):
    - the current plate's instances (`contain_instance || intersect_instance`) are selected;
    - every other instance is locked by `preprocess_arrange_polygon_other_locked` (`PartPlate.cpp:7663-7690`);
    - only the tower the plate already shows is added; no estimate is made;
    - exclusion areas cover beds `0..=current` (`preprocess_exclude_areas(..., current + 1)`);
    - an empty plate arranges nothing.
  - `init_arrange_params` for `PREPARE_STATE_MENU` (`:1026-1034`): the plate's own print sequence (`PartPlate::get_real_print_seq`, `PartPlate.cpp:370`), and an automatic distance when that sequence differs from the global one.
  - `finalize` with `only_on_partplate` (`:851-856, 860-863, 877-878, 946-951`):
    - `clear` empties only the current plate;
    - `postprocess_bed_index_for_current_plate` (`PartPlate.cpp:7913-7928`) maps bed 0 to the current plate, and any other bed to the plate count;
    - locked instances are applied back unchanged, through `set_rotation(Z)` and taking an arrange order;
    - `rebuild_plates_after_arrangement(false, true, current)` sorts the objects and reloads the current plate.
- **Destination:**
  - `ArrangeScope::CurrentPlate(usize)`, with the A3a modules extended: `prepare_partplate` next to `prepare_all`;
  - the plate-list steps `preprocess_arrange_polygon_other_locked` and `postprocess_bed_index_for_current_plate` in `plater/plate_list/arrange.rs`;
  - plate-scoped `clear` and `reload_all_objects`.
- **A3a rejections that no longer apply:**
  - A plate's own print sequence is supported here: the MENU job reads it and locks nothing.
  - The tower is supported here as the existing-tower part of A3b.
- **Oracle:** the CLI global-arrange path with `--slice <plate>` (`OrcaSlicer.cpp:5815-5960, 6022-6035, 6091`), started as `--slice <n> --arrange 1 <flags> --export-3mf`.
  - Probed: `custom-gcode-overview/two-plates.3mf` with plate 2's cube moved off-centre arranges it to plate 2's centre, slices plate 2 and returns 0.
  - The CLI then exports only that plate (`store_params.export_plate_idx`, `:8399-8404`). The comparison therefore covers the current plate's build items and its plate entry, keyed by object name in build order as in A3a. The locked instances on other plates are not in Native's output.
- **Where the CLI and the GUI job differ, and how the cases avoid it:**
  - **Tower.** The CLI estimates its own tower for the plate (`:5855-5955`); the GUI uses the tower the plate shows. The cases print one filament with no smooth timelapse, so neither path adds a tower.
  - **An item that does not fit.** The CLI stops with `CLI_OBJECT_ARRANGE_FAILED` (`:6026-6031`); the GUI moves it past the plates. The cases fit on the plate.
  - **Unprintable instances** move to bed `current + 1`, outside the exported plate. The cases have none on the current plate.
  - **Distance.** The CLI always keeps the distance automatic; the GUI uses the setting unless the plate's sequence differs. The cases use distance 0.
  - **Slicing.** The CLI slices the plate before exporting, so the case projects must slice at v2.5.0-beta.
- **A3c cases:**
  1. An off-centre object on plate 2 of a two-plate project, centred on plate 2.
  2. Several objects on plate 2, with plate 1's object left in place.
  3. As case 2, with rotation.
  4. Align to Y on the i3 printer.
  5. `bed_exclude_area` on plate 2.
  6. Plate 2 printed by object while the global sequence is by layer.

**Oracle and comparison (A3a).**
- Native run: the official v2.5.0-beta CLI global arrange (`OrcaSlicer.cpp:5619-6098`), started as `--arrange 1 --allow-rotations=<0|1> --avoid-extrusion-cali-region=<0|1> --align-to-y-axis=<0|1> --export-3mf <out> <input.3mf>` through `scripts/orca-parity.sh` with `ORCA_APPDIR`.
- Booleans take `=<0|1>` (`Config.cpp:1750-1773`). Upstream disables the `--no-` prefix (`:1822`), and a separate value (`--align-to-y-axis 0`) is read as an input file. All three flags are always passed, so no CLI default applies.
- Ares side: `PlaterSession::arrange(All, settings)` with the same settings.
  - Each instance's transform is written the way `_add_build_to_model_stream` writes it: `%.9g`, from `std::setprecision(float max_digits10)` at `bbs_3mf.cpp:7150-7151`.
  - The result must equal the Native build-item `transform` strings and the plate→object/instance map in `Metadata/model_settings.config`.
- Per-plate G-code is not compared until slicing is at v2.5.0-beta parity.

**Where the CLI and the GUI job differ, and how the A3a cases stay on the shared path.**
- `clearance_radius`: the GUI adds `2 * Print::object_skirt_offset` (`ArrangeJob.cpp:1000`). Fixtures have no per-object skirt, so the offset is 0.
- `check_unprintable` (GUI only): no fixture has a zero-area item or an item taller than `printable_height`.
- Plates locked for a print sequence that differs from the global one (GUI `prepare_all` `:345-354`): no fixture overrides a plate's print sequence.
- `preprocess_exclude_areas` plate count: the GUI passes 36 and the CLI defaults to 16 (`PartPlate.hpp:1047`). Fixtures stay within 16 plates.
- Plate recycling: the CLI calls `rebuild_plates_after_arrangement()`, which recycles plates; the GUI passes `false, true`. No fixture leaves a plate empty.
- The wipe tower: every fixture prints one filament per plate, so neither path adds a tower (`OrcaSlicer.cpp:5733-5748`).
- Unprintable instances:
  - The CLI arranges them on their own before moving them past the last plate (`OrcaSlicer.cpp:6012`). The GUI job moves them without arranging (`ArrangeJob.cpp:912-920`).
  - The unprintable fixture's instance sits at the bed centre, where an arrange of it alone leaves it.
- Object order: both paths sort the model objects by their first instance's `arrange_order` (`PartPlate.cpp:8389`), which renumbers the exported object ids. The comparison therefore keys build items and plate entries by object name, and the fixture edits give the two cubes distinct names.

The ported code follows the GUI job. These differences are listed so that a mismatch can be traced back to them. They are not fixed up on the Ares side.

**A3a unported paths are rejected.** `PlaterSession::arrange` returns `Result<(), SliceError>`. It returns `SliceError::InvalidInput` instead of arranging when the project needs any of these:
- a prime tower (A3b);
- a per-object skirt clearance (`Print::object_skirt_offset` past its no-skirt guard);
- the BBL first-layer-scan strip (`preprocess_nonprefered_areas`);
- a plate with its own print sequence, which the GUI locks;
- the shared bed of several nozzles (`get_shared_poly`).

**Kernel position parity depends on Clipper2.**
- At `345efec64`, `ClipperUtils::_clipper_ex` clips with Clipper2 (`C2::PolyTree64`, `c2_clip`, `ClipperUtils.cpp:823-829`). libnest2d's `nfp::merge`/`nfp::subtract` are `union_ex`/`diff_ex` (`backends/libslic3r/geometries.hpp:259-264`). Their output vertex order sets where each NFP outline starts.
- `EdgeCache` reads an open outline without its closing edge (`nfpplacer.hpp`, `createCache`). So that start decides which edge the placer never searches, and which corners subplex starts from.
- The A1 kernel still uses Ares' 2.4.2 Clipper1 `difference_ex`/`union_expolygons`.
- Evidence: a scene with a 30 x 10 mm bar, a cube turned 30° and a cube, on the Ender-3 bed.
  - The v2.5.0-beta CLI logs its first-fit scores with `--debug 5 --logfile`. It places the second item at score 0.155280; the port places it at 0.214060.
  - The port's objective at Native's position is 0.15528036, so the objective agrees.
  - Native's position lies on the port's skipped closing edge.
- A3a GREEN waits for the Clipper2 `ClipperUtils` port. The kernel's merge and subtract then switch to it, and so does its inflation offset if the beta offsets through Clipper2. All eight A3a cases are then re-run.

**A3a cases.**
1. Ender-3 project with a single object: centring.
2. Several objects on one plate, rotations off and then on.
3. Overflow onto new plates.
4. Align to Y on an i3 printer, plus the `--align-to-y-axis=0` control.
5. `bed_exclude_area`.
6. By-object print clearance (`print_sequence = by object`).
7. A circular (delta) bed.
8. An unprintable instance going to the plate after the last one.

## STL plater session (core slice before commit 5)

Coordinator decision: approved as its own source-cited core slice, RED then
GREEN.

**Upstream (2.4.2, `8500fcdcc`):**
- `Plater::priv::load_files` loads an STL into the `Model`.
- `Plater::priv::load_model_objects` (`Plater.cpp:7056-7058, 7111-7112, 7149-7160`) places an object loaded onto an empty plate at the bed's centre and drops it onto the bed.
- The selected presets' `printable_area` sets the bed, and with it the plate list (`PartPlateList::reset_size`).
- The object is then edited and sliced like any other model; slicing never re-places it.

**Ares destination:**
- `PlaterSession::load_with_profiles(input, ProfileSliceRequest)` loads through the existing `profile_placed_project` (`Placement::Center`, `OrcaSlicer.cpp:4376-4392`, `Model.cpp:704-722`). Its plate list comes from the composed settings.
- `slice_plater_with_profiles` reloads the STL with the presets, skips the placement, and applies the session edits; the session transforms already hold the centring.
- STL input without selected presets has no bed, so it keeps today's `sliceStl` path and gets no session.

**Native oracle:** the official and seeded CLI with `--load-settings`/`--load-filaments` and `--arrange 0`, on cube10 pre-positioned where the session puts it (the `orca_parity_profile_entry_stl_center_placement` pattern). The case is first unedited, then after a raised move.

## Follow-ups (accepted deviations, coordinator review of commit 6)

- **Rotate ring snapping:** `GLGizmoRotate::on_dragging` snaps the angle within the inner snap regions (`SnapRegionsCount` 8, `GLGizmoRotate.cpp:24, 102-120`). The web ring turns freely.
- **Relative rotation fields:** upstream "Rotate (relative)" shows the rotation accumulated since the tool opened and rotates by the edit's difference (`GizmoObjectManipulation.cpp:960-1000`, `change_rotation_value`). The web fields rotate by the entered value and show 0 again.
- **Move and rotate drops:** a gizmo or drag step runs `do_move`/`do_rotate` on every step rather than once at mouse-up, so a Z drag shows no lift while dragging. The final state is the same.
- **Plate edits invalidate every plate's slice:** upstream invalidates only the affected plates.
- **Per-plate wipe tower position on delete:** `PartPlateList::delete_plate` erases that plate's `wipe_tower_x`/`wipe_tower_y` entries (`PartPlate.cpp:4580-4590`); this is not ported.
- **Bed size change:** the plate grid keeps the bed size the session loaded with; upstream re-lays out the plates.
