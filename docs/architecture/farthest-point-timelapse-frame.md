# Farthest-point timelapse target parity

## Decision

Match `GCode::compute_farthest_point` (`OrcaSlicer/src/libslic3r/GCode.cpp:6332-6483`) by selecting
from fitted paths before seam placement. The per-extrusion inline hook then uses that target
(`GCode.cpp:9974-9996`, call sites `:10118`, `:10148`, `:10174`, `:10321`). On the arc-fitted X1
Carbon fixture the emitted fitting split is identical, but Ares scans after seam placement
while Native scans before it.

The pre-seam scan input is the scope of this rewrite slice.

## Upstream boundary

| upstream | role |
| --- | --- |
| `GCode.cpp:6332-6483` `compute_farthest_point` | selects `farthest_point` / `farthest_gcode_pos` once per layer |
| `GCode.cpp:6350-6363` `for_each_path` | enumerates leaf `ExtrusionPath`s through collections, loops, multipaths |
| `GCode.cpp:6365-6373`, `:6428-6445`, `:6466-6476` | only `erExternalPerimeter` enters the external pass; a positive external maximum takes precedence over the infill/support fallback |
| `GCode.cpp:6376-6385` `update_max` | metric is `dsq = global.x² + global.y²` with `global = point + instance shift`; strict `>` so the first wins ties |
| `GCode.cpp:6390-6412` `collect_from_path` | with fitting data: first segment's `start_point_index`, `Linear_move` walks `start..=end`, arc moves contribute `arc_data.end_point` only; without fitting: every point |
| `GCode.cpp:6470-6483` | `farthest_gcode_pos = unscale(farthest_point)` (`:6479`); `farthest_is_photo_head = single_nozzle || candidate extruder == most_used` |
| `GCode.cpp:9974-9996` `check_and_insert_timelapse` | early-returns unless enabled, photo head, not yet inserted this layer, current extruder is the photo head; compares `unscale(endpoint_scaled) + m_origin` against `farthest_gcode_pos` with a 0.5 mm radius; on success renders with `skip_pos_pick = true` and latches `inserted_this_layer` |
| `GCode.cpp:9981-9984` | the comment that states the frame rule: comparing in `point_to_gcode()` space would subtract `extruder_offset` on the endpoint side only |
| `GCode.cpp:6479` | `farthest_gcode_pos = unscale(farthest_point)`, which carries no `extruder_offset` |
| `GCode.cpp:6716-6719`, `:7410-7419` | `enabled` gate, and suppression of the BBL layer-start insert while the inline hook is active |
| `GCode.cpp:8193-8227` | layer-end fallback insert when the inline hook never fired |

## Ares destination boundary

| Ares | maps to |
| --- | --- |
| `crates/ares-core/src/project_slice/gcode_emit/layers/farthest.rs:36-66` `begin_layer` | enable gate, per-layer target storage |
| `crates/ares-core/src/project_slice/gcode_emit/layers/farthest.rs:68-96` `check_and_insert` | `GCode.cpp:9974-9996`: gate, radius test, `append_inline` |
| `crates/ares-core/src/project_slice/gcode_emit/layers/farthest/target.rs:16-72` `Maximum::collect` | `collect_from_path` |
| `crates/ares-core/src/project_slice/gcode_emit/layers/farthest/target/object.rs`, `target/support.rs` | `for_each_path` + role predicates |
| `crates/ares-core/src/project_slice/gcode_emit/motion/path/constant.rs:105-111`, `:138-148`, `motion/path/contour.rs:60-71` | the four upstream call sites |
| `crates/ares-core/src/project_slice/gcode_emit/timelapse.rs:55-61`, `:117-140` | `generate_timelapse_gcode(..., skip_pos_pick=true)` |

## Measured defect

Fixture `tests/parity/seam-position-random/x1c-seam-position-random.3mf`, seeded harness with
`ORCA_REPLAY_SEED=0`. The divergent layer is `M991 S0 P44` (Z=9.0). Its 2,308 motion commands
are byte-identical and in the same order. The 14 differing line positions (91888–91901) are the
snapshot block crossing one Inner wall move; the Outer wall starts after this block on both sides.

| Inner wall endpoint | Native line | Ares line | Ares inline check |
| --- | ---: | ---: | --- |
| `G1 X169.894 Y141.957 E2.02179` | 91887 | 91887 | fit 9, idx41, global `(169.894207,143.957057)`, d=2.020679 mm: miss |
| `G3 X168.873 Y143.299 I-1.382 J.008 E.06016` | 91901 | 91888 | fit 10, idx47, global `(168.873441,145.299101)`, d=0.437518 mm: fires |

Ares measures against target `(168.930142,145.732929)`. The closing G3 starts at the end of the
G1 in both streams. Its Ares fitting range is 41→47: points 42–46 are interior and not checked.
Native checks after each non-short fitted linear move (`GCode.cpp:10128-10150`), but only at
`arc.end_point` for an arc (`:10152-10175`). The observed block position is consistent with Native
firing after the G1. No Native endpoint or target XY was read at the inline call site.

The old winning idx43 at `(170.285920,144.375192)` is from an Outer wall scan dump, not an
endpoint of the emitted Inner wall on P44. At rendered precision on P44, the old indices
40/41/42 map to the same Outer wall endpoints on both sides: G1 `(170.293,80.636)`, G3
`(170.329,80.997)`, and G1 `(170.329,141.981)` (lines 91935–91937). The last is Ares emitter
idx47 and is already latched. Neither output emits a separate move for the old indices 43/44;
the G3 at 91938 instead runs from `(170.329,141.981)` to `(168.957,143.726)`. These matches
do not identify Native's hidden fitting indices.

## Resolved boundary: scan fitted paths before seam placement

Native scans `ltp.object_layer` → `region->perimeters.entities` with the shifts from
`ltp.original_object->instances()` (`GCode.cpp:6419-6435`). It calls the scan after grouping the
layer's extrusions (`:7259-7262`), before `extrude_loop` copies the loop (`:8526`), places the seam
on the copy (`:8547`), and clips the loop tail (`:8583`). Precomputing seam choices is not the same
as mutating the scanned path. The ExternalPerimeter path passes the role predicate; the
seam-created endpoint is absent from the pre-seam data, not excluded by role.

Ares simplifies/fits `ordered` and then applies seam placement to it
(`crates/ares-core/src/project_slice.rs:242-244`). `seam_placement.rs:187-189` applies the chosen
seam to each layer before emission. `gcode_emit/layers/drive.rs:161-172` passes those same objects
to `farthest::begin_layer`; `target.rs:114-122` forwards `objects[object][layer]`,
`target/object.rs:56-67` passes the external path's polyline, and `target.rs:34-69` scans its
already-mutated points and fitting ranges. The divergence is in **the input to the target scan**.

The P44 winner on both Ares snapshots belongs to object 0, instance 0, region 0, perimeter
collection `source_order=2`, leaf 12 (`ExternalPerimeter`). An offline replay of all 14 external
paths at each phase gives:

| Ares scan input | Winning candidate | Local scaled XY | Global XY (mm) | Squared norm (mm²) |
| --- | --- | --- | --- | ---: |
| After path fitting, before seam placement | linear `7→8`, point 8 | `(37290001,27988589)` | `(170.329206,143.980694)` | 49742.478661312 |
| After seam placement, before export clipping | arc fit 12 `47→53`, endpoint 53 | `(35890937,29740824)` | `(168.930142,145.732929)` | 49775.479471059 |

The post-seam winner does not exist at that XY in the original fitted path. Its nearest original
point is idx14, an interior point of arc `8→15`; seam splitting creates a projected endpoint
(`seam_placement/split.rs:209-216`, `seam_placement/fitting.rs:19-31,151-162`). The pre-seam
winner's squared norm exactly matches the Native P44 scalar (hit 45); the scalar does not directly
expose Native XY. The pre-seam Ares winner is 0.435641 mm from the Inner wall G1, consistent with
Native's observed placement. The `(170.285920,144.375192)` candidate and hit 46 belong to P45.

Both checks use the global print frame, not the G-code frame: on this fixture G-code XY subtracts
nozzle offset `(0,2)` and writer XY offset is zero (`GCode.cpp:9981-9985,11644-11648`). Neither the
coordinate frame, arc emission, nor inline insertion order needs to change for this boundary.

## Included behaviour

- Select candidates from simplified and fitted paths before seam placement and loop-tail clipping,
  matching `GCode.cpp:6390-6435` and `:7259-7262`.
- Carry only the selection data needed across seam placement, not a second copy of the paths.
  Retain the existing role and tie rules, instance shifts, tool attribution, and photo-head gate.
- Keep candidate selection and output independent of available CPU count. The P44 public API
  comparison must place the snapshot before the unchanged Inner wall arc.

## Deferred behaviour

- Changes to seam placement, clipping, arc fitting/emission, and the inline insertion hook;
- `timelapse_pos.rs` pos-picker semantics and `printed_objects` label ids;
- the layer-end fallback insert (`GCode.cpp:8193-8227`);
- non-inline traditional inserts and their position picking.

## Existing scaffolding

The existing post-seam `OrderedExtrusionLayer` view is the mismatched input to
`GCode::compute_farthest_point` parity. The scope is to supply the fitted, pre-seam candidates to
the existing target scan; retain its role/metric rules, the inline hook, and emitted fitting.
The previous post-seam scan input is replaced, not retained as a fallback.

## Verification

The seeded reference lane must match the Native placement before the Inner wall G3, without
changing the motion commands or the 263837-line count. Lock the P44 regression on the non-zero
`extruder_offset` fixture and preserve byte-identical output on a zero-offset control.

## Citations to correct while implementing

- `layers/layer_end.rs:2` cites `GCode.cpp:5527-5546`, which is the M205 / bed-temperature region
  at `40ecc66f`, not the layer-end timelapse insert.
- `emit_helpers.rs:105-113` states the BBL block arrives at `GCode.cpp:5205-5210`; it is
  `:7413-7420`.
- `layers/boundary.rs` body ranges `:4667-4685`, `:6704-6711`, `:5690-5718` do not match the
  measured emission order.
