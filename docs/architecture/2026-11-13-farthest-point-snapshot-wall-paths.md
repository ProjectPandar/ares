# Farthest-point snapshot on arc, segmented, sloped and z-contoured paths

Spec for porting the remaining call sites of upstream `check_and_insert_timelapse`.
It extends `2026-11-04-beta-inline-farthest-point-snapshot-ard.md`, which ported the hook
itself and its calls after straight G1 moves. That ARD listed these call sites as deferred.
Citations refer to OrcaSlicer 2.5.0 (`40ecc66f7`), `src/libslic3r/GCode.cpp` unless noted.

## Upstream boundary

`GCode::_extrude` defines the hook lambda `check_and_insert_timelapse` at `:9974-9996`.
It returns early unless all of these hold:

- the subsystem is enabled (`:6711-6713`: `farthest_point_timelapse`, traditional
  timelapse, not an I3 printer);
- the farthest point is on the photo head;
- no snapshot has been taken on this layer yet;
- the active extruder is the photo head;
- the endpoint (global print frame) is less than 0.5 mm from `farthest_gcode_pos`.

If they all hold, it appends `generate_timelapse_gcode(..., true)` and marks the layer as
inserted. `_extrude` calls the hook after every emitted move, on four branches:

| Upstream call | Branch | Ares destination | Before this slice |
|---|---|---|---|
| `:10118` | no fitting data (arc fitting off, no fitting result, spiral vase, or a sloped path). Per line, one of: z-contoured `extrude_to_xyz` (`:10083`), plain `extrude_to_xy`, sloped `extrude_to_xyz` | `motion/path/contour.rs` `emit`; `motion/path/constant.rs` line branch; `motion/scarf/emit.rs` `segments` | constant line only |
| `:10148` | fitted `Linear_move` | `motion/path/constant.rs` line branch | ported |
| `:10174` | fitted `Arc_move`, at `arc.end_point` | `motion/path/constant.rs` arc branch | missing |
| `:10321` | segmented extrusion (`requires_segmented_extrusion`, `:9765-9779`: a point speed more than 1 mm/s from the path speed, or an overhang-fan state change), at `processed_point.p` after each emitted segment | `motion/path/variable.rs` `emit` | missing |

The hook's endpoint convention is unchanged from the existing port: local path millimetres
plus `EmitState::origin`, without the extruder offset. For each new call:

- the arc branch passes the fitted end point (`local_points[endpoint]`, the index the
  fitting result carries);
- `variable.rs` passes `processed[index]`;
- `contour.rs` passes the unscaled `Point3` XY;
- the scarf emitter passes the unclipped local point of each line end.

All four calls come after the move is emitted, as upstream does. Every caller already
returns `Result`, and the hook's `?` propagates the generator's error unchanged.

### Layer change after an in-layer snapshot

`generate_timelapse_gcode` marks the writer position as not clear (`:6509`). When the
snapshot fires during a layer's last path, nothing travels before the layer change, so
the position is still not clear there. That only happens through the inline hook.

Upstream `change_layer` (`:8458-8470`) still retracts and schedules the lazy lift
whenever `retract_when_changing_layer && will_move_z(z)`. It does not look at the
clear flag. `GCodeWriter::travel_to_xyz` then consumes that lift
(`GCodeWriter.cpp:1001-1095`). With an unclear position the spiral form is skipped, and
the move is XY first, then `_travel_to_z(z + hop)`, then the unlift before the first
extrusion:

```gcode
G1 X170.645 Y155.645 F60000
G1 Z9.8
G1 Z9.4
```

Ares `motion.rs` `defer_layer_retraction` requires `state.positioned` before it defers
the layer-change retraction. After an in-layer snapshot on the last path, Ares therefore
schedules no lift and emits no Z move at all. The next layer is printed at the previous
layer's height. This slice drops that condition to match `:8466`. Before this slice no
Ares path reached `defer_layer_retraction` with an unclear position: the layer-end
snapshot runs after it, and a layer-start snapshot is followed by a travel.

## Included

- Hook calls after arc moves (`:10174`), after each segmented-extrusion segment
  (`:10321`), and after each z-contoured or sloped line (`:10118`).
- The layer-change retraction and lift after an in-layer snapshot, without the
  position condition (`:8466`).

## Deferred

- The farthest point on a non-photo head (`is_case_b`, `:7521-7522`, `:7551-7552`). It belongs to the
  multi-head placement work, not to the per-move hook.
- `end_layer_for_timelapse` keeps its `state.positioned` condition. Its callers run only
  when no snapshot fired in the layer, so the position is clear there.
- The remaining difference on `tests/ksr_fdmtest_v4`: the project lacks
  `farthest_point_timelapse`, while the system preset sets 1. That is the project-preset
  refresh at load (`src/OrcaSlicer.cpp:3164-3195`), not this slice.

## Measurements behind the boundary

A diagnostic Native build (local only, not committed) printed the call site of every
inline insert:

| Input | Native call sites | Ares before this slice |
|---|---|---|
| `ksr_fdmtest_v4` with `farthest_point_timelapse=1` | 360 × `:10148`, 99 × `:10321`, 1 × `:10174` | layer-end fallback on exactly the 100 layers that used `:10321`/`:10174` |
| lens prism (two 64-segment arcs) | 50 × `:10174` | layer-end fallback on every layer |
| box widening to a far overhang | 49 × `:10321`, 1 × `:10148` | layer-end fallback from layer 2 |
| box with an entire-loop scarf seam | 49 × `:10118` sloped, 1 × `:10148` | layer-end fallback from layer 2 |
| ZAA wedge (top falls toward the far corner) | 13 × `:10118` (12 plain, 1 z-contoured), 36 × `:10148` | fallback on layers 26 and 38-48; no layer-start Z on 47-49 |

The first KSR insert after the straight-move port is on layer 2 at `:10321`.

## Verification

Parity cases in `crates/ares-cli/tests/orca_parity/option_coverage/printer_physical_identity/farthest_timelapse_paths.rs`.
Each case:

- builds P0 (H2D) with its object mesh replaced, `farthest_point_timelapse = 1`, and every
  feature on filament 1;
- asserts the number of Native snapshots followed by an extrusion in the same layer;
- compares the whole output of Native, the Ares API and the Ares CLI as ordered bytes.

The cases are the four synthetic inputs above:

- `farthest_snapshot_fires_after_fitted_arc_wall`
- `farthest_snapshot_fires_inside_slowed_overhang_wall`
- `farthest_snapshot_fires_on_scarf_sloped_loop`
- `farthest_snapshot_fires_on_zaa_contoured_wall`

The ZAA wedge also covers the layer change after an in-layer snapshot. All four run in
official and seeded Native modes.
