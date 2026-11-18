# Farthest-point timelapse target parity

## Decision

`GCode::compute_farthest_point` (`OrcaSlicer/src/libslic3r/GCode.cpp:6326-6483`) and the
per-extrusion inline hook `check_and_insert_timelapse` (`GCode.cpp:9974-9996`, call sites
`:10118`, `:10148`, `:10174`, `:10321`) must agree on which extrusion vertex is the snapshot
target, and the selected target must be checked on the same move Native checks it. On the
arc-fitted X1 Carbon fixture the emitted fitting split matches, but the target's squared norm
differs between Ares and Native on the divergent layer. The symptom is a snapshot
after, rather than before, the closing Inner wall arc.

This document records the parity boundary and evidence; it does not yet assign the implementation.

## Upstream boundary

| upstream | role |
| --- | --- |
| `GCode.cpp:6326-6483` `compute_farthest_point` | selects `farthest_point` / `farthest_gcode_pos` once per layer |
| `GCode.cpp:6342-6368` `for_each_path` | enumerates leaf `ExtrusionPath`s through collections, loops, multipaths |
| `GCode.cpp:6370-6377` role predicates | external pass is `erExternalPerimeter`; fallback is `erSolidInfill`/`erInternalInfill`/`erTopSolidInfill`; support pass is support roles |
| `GCode.cpp:6379-6387` `update_max` | metric is `dsq = global.x² + global.y²` with `global = point + instance shift`; strict `>` so the first wins ties |
| `GCode.cpp:6394-6420` `collect_from_path` | with fitting data: first segment's `start_point_index`, `Linear_move` walks `start..=end`, arc moves contribute `arc_data.end_point` only; without fitting: every point |
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

## Remaining boundary

Native runs its scan once per layer (`GCode.cpp:7259-7262`). The prior comparison used hit 46
(zero-based P45), not hit 45 (P44). On P44, the logged Native maximum is exactly
`170329206² + 143980694²` in scaled units and does not equal the squared norm of Ares' logged
target. The later Outer wall endpoint `(170.329206,143.980694)` is consistent with that Native
maximum and with a hit at the preceding Inner wall G1 (0.435641 mm from it); the scalar maximum
alone does not prove Native's XY. Both endpoint checks use the global print frame, not the
G-code frame: on this fixture G-code XY subtracts nozzle offset `(0,2)` and writer XY offset is zero
(`GCode.cpp:9981-9985,11644-11648`).

Compare candidate membership at the **pre-emission target scan** (`GCode.cpp:6394-6414,6464-6483`
versus `layers/farthest/target.rs` and `target/object.rs`). The equal rendered split does not
establish equal fitting data at that earlier point. The cause of the target difference is still
open; neither a change to arc fitting nor a reordering of the inline hook is justified by this
trace.

## Deferred behaviour

- `timelapse_pos.rs` pos-picker semantics and `printed_objects` label ids;
- the layer-end fallback insert (`GCode.cpp:8193-8227`);
- non-inline traditional inserts and their position picking.

## Existing scaffolding

Retain the existing target scan and inline hook while investigating the candidate input. No
replacement or arc-fitting ownership change is prescribed.

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
