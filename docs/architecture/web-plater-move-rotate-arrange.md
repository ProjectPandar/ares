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
- Exposed as `PlaterSession::arrange(scope, settings)`, with `scope` all / current plate / selection.

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
- the all, current-plate and selection scopes;
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
