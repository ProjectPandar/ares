# G-code writer plate offset

Rewrite slice of the G-code writer's plate offset, at OrcaSlicer v2.5.0-beta `345efec64`.

## Problem
A plate other than the first sits at a grid origin in the global frame (plate 2 of a 220 mm bed at X 264). Native keeps every emit position global and subtracts the plate origin only when the writer formats a move. Ares subtracts it first, from each instance's origin. The two orders round differently across a 0.0005 mm formatting boundary.

A 12.5 x 12.5 x 250 mm cube centred on the Ender-3's plate 2 slices to `G1 X116.892` at line 536 in Native, where Ares writes `X116.893`. The same cube on plate 1 gives `X116.893` in both.

## Upstream boundary
- **`GCodeWriter::m_x_offset`, `m_y_offset`** (`GCodeWriter.hpp:204-205`), set by `set_xy_offset` (`:134`).
- **`GCode::set_gcode_offset`** (`GCode.hpp:264`), through `set_gcode_offset_with_imex_shift` (`:270-273`): the writer and the G-code processor take the plate origin. `Print::export_gcode` passes `get_plate_origin()` (`Print.cpp:4000-4010`).
- **`GCode::point_to_gcode`** (`GCode.cpp:11482-11486`): `unscale(point) + m_origin - extruder_offset`, with `m_origin` the instance's global `shift` (`set_origin(unscale(offset))`, `:7643`). `set_origin` (`:8404-8414`) translates `m_last_pos` and the wipe path by the scaled origin change.
- **The writer functions that subtract the offset from the global `m_pos` or point** (`GCodeWriter.cpp`):
  - `travel_to_xy` (`:873-890`);
  - `eager_lift` (`:936-959`);
  - `travel_to_xyz` (`:980-1114`), including the first-layer speed test (`:994`) and the travel-length source and target (`:1015-1016`);
  - `_travel_to_z` (`:1175-1189`);
  - `_spiral_travel_to_z` (`:1200-1222`);
  - `extrude_to_xy` (`:1303-1314`);
  - `extrude_arc_to_xy` (`:1377-1401`);
  - `extrude_to_xyz` (`:1421-1435`).
- **Inputs made global before the writer:**
  - The skirt and brim emit at `set_origin(0, 0)` (`GCode.cpp:6053`; `generate_skirt` with `Point(0, 0)`, `:5987`, `:7446`).
  - Wipe-tower points add `plate_origin_2d` before `wipe_tower_point_to_object_point` (`:752-755`; `:1015-1945`).

## Ares destination
- **`EmitState`** (`project_slice/gcode_emit/motion/state.rs`) holds positions, `origin` and wipe points in the global frame, plus the writer's `xy_offset`, set once from `Project::plate_origin()` as `set_gcode_offset` does.
- **Each G1/G2/G3 XY format site in `gcode_emit`** subtracts `xy_offset` where its upstream writer function does, and nowhere else. These are `motion/extrusion.rs`, `path/travel_emit.rs`, `start_travel/approach/waypoints.rs`, `travel/lift.rs`, `lift/spiral.rs`, `travel/wipe_sequence.rs`, `loop_paths/inward.rs`, `layers/transition.rs`, and the wipe tower's `result.rs`, `type1/result.rs`, `tool_change.rs` and `postprocess.rs`.
- **The instance origin** in `layers/tool_entry.rs` becomes `unscale(shift)`, without the plate origin.
- **The skirt and brim** (`skirt.rs`, `brim/emit.rs`) set origin `(0, 0)` instead of `-plate_origin`.
- **Wipe-tower inputs** add the plate origin as upstream does.
- **Unchanged:**
  - The G-code processor's offset (`gcode_processor/printable_area.rs`) is already ported.
  - `footprint.rs` and `wipe_tower/extents.rs` already compute in the global frame and subtract the origin for their own outputs, as upstream does.
- **Plate 1** has offset `(0, 0)`, so every subtraction is exact and its output is byte-identical by construction.
- Files over 400 lines are split first, in a separate move-only commit.

## Included and deferred
- **Included:** the plate offset for every writer XY move, object paths, travels, lifts, arcs, skirt, brim and the wipe tower.
- **Deferred:** the IMEX zone shift (`imex_x`, `imex_y` in `GCode.hpp:270-273`; `Print::update_imex_slice_offset`, `Print.cpp:4522-4536`). It is non-zero only with `is_imex` and `imex_firmware_managed_zones`. Ares reads those keys but does not port the zone shift today, so this slice keeps the existing gap and the writer offset is the plate origin alone.
- **Deferred:** wipe-tower coverage on plate 2, both the whole output and the `;TYPE:Prime tower` section. Every candidate tower fixture (`print-order-as-obj-list/class-default.3mf`, with or without the rectangle-tower edits, and `bbl-type1-tower/enabled.3mf`) already slices differently from Native on plate 1, so no plate-2 case can isolate the offset. The tower inputs are still ported literally. The coverage comes back once a tower fixture slices byte-equal on plate 1.

## Oracle and RED
In a new `plater_plate_offset.rs`, a `plater_session` child. Each case slices a plate like Native slicing the session's GUI export.
1. **Rectangle bed, plate 2.** `orca_cli_ender3.3mf`'s cube set to 12.5 x 12.5 x 250 mm (Size edits, uniform off), on an added plate 2 at plate 2's origin + (110, 110). Plate 2 slices like Native; today it is off by 1 µm at line 536.
2. **Plate-1 control.** The same cube on plate 1 at (110, 110) slices like Native. This passes today and must keep passing.

GREEN gate: the paired FULL run has no PASS→FAIL, and every plate-1 case stays byte-identical.
