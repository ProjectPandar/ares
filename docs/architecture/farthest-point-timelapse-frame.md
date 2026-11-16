# Farthest-point timelapse snapshot frame mismatch

## Decision

`GCode::compute_farthest_point` (`OrcaSlicer/src/libslic3r/GCode.cpp:6326-6483`) and the
per-extrusion inline hook `check_and_insert_timelapse` (`GCode.cpp:9974-9996`, call sites
`:10118`, `:10148`, `:10174`, `:10321`) must agree on which extrusion vertex is the snapshot
target, and the selected target must be checked on the same move Native checks it. On the
arc-fitted X1 Carbon fixture the target values already agree bit-for-bit, yet the rendered block
lands one move later in Ares — after the arc that closes the perimeter loop instead of before it.

This document records the upstream boundary, the measurements that localise the difference, and
what is still open. It does not yet prescribe the implementation.

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

Fixture `tests/parity/seam-position-random/x1c-seam-position-random.3mf`, harness with
`ORCA_REPLAY_SEED=0`. Native emits the timelapse block **before** the arc that closes the
external-perimeter loop; Ares emits it **after**, a 14-line / 5-byte divergence inside lines
91886-91899 of a 263837-line file.

Ares-side dump of the winning polyline shows the target is the Outer wall closed loop
(`points = 55`, `fitting = 16`, point 54 == point 0). The winning vertex is index 43 at global
`(170.2859, 144.3752)`, which is the **end** of the linear fit `start=41 end=43` and the
**start** of the arc fit `start=43 end=49`, so it lies inside `Linear_move`'s
`start_point_index..=end_point_index` range.

Resolved runtime offsets on this fixture (`@@OFFS`, throwaway build):

| value | measurement | config source |
| --- | --- | --- |
| planning origin `state.origin` | `(133.0392, 115.9921)` | instance shift, **not** the plate origin |
| `state.extruder_offset` | `(0.0000, 2.0000)` | `extruder_offset = ["0x2"]` in `Metadata/project_settings.config` |
| `state.xy_offset` | `(0.0000, -0.0000)` | zero on this fixture |
| `plate_origin` | `(0.0000, -0.0000)` | zero on this fixture |

Two representative near-miss pairs:

| endpoint | target | delta | distance |
| --- | --- | --- | --- |
| `(169.8942, 143.9763)` | `(170.3292, 143.9870)` | `(0.435, 0.0107)` | `0.4351` |
| `(170.3292, 143.9806)` | `(170.2859, 144.3752)` | — | `0.5591` |

Because `line_width 0.42 + gap 0.015 = 0.435` exactly, the first pair is one **Inner wall**
endpoint compared against an **Outer wall** target: a genuine geometric separation, not a frame
mismatch. `local_points[endpoint] + state.origin` already has the upstream form
`unscale(endpoint_scaled) + m_origin`, and `xy_offset` contributes zero here, so rewriting the
endpoint expression would not change any number.

## What is actually open

The target itself is settled: breaking `GCode.cpp:6479` on the seeded oracle with its split
debuginfo (build id `4ad7c739…`, firing once per layer) gives
`max_dist_sq_ext = 49841490615283264` for the layer that owns the divergent block, which is
**bit-identical** to Ares' winning candidate `d2` on the same layer; all 460 layers agree to the
last printed digit. So the scan rule, the role set, the metric and the target vertex all match.

Both sides also emit and check **one move per source point**: Native walks
`point_index = start_index + 1 .. end_index` (`GCode.cpp:10128-10141`), and Ares emits
`points[start..=end].windows(2)` (`motion/arc/retained.rs:8-37`) zipped with endpoints
`fitted.start + 1 ..= fitted.end` (`motion/path/constant.rs:86-94`) — the same count.

The fire table for the winning Outer-wall polyline (target = vertex 43
global `(170.2859, 144.3752)`):

| index | global | d to target | verdict |
| --- | --- | --- | --- |
| 41 | `(170.3292, 82.9973)` | 61.3779 | miss |
| 42 | `(170.3292, 143.9806)` | **0.3970** | **fires** |
| 43 | `(170.2859, 144.3752)` | 0.0000 | suppressed by the once-per-layer latch |

Ares fires at index 42, and Native must fire at index 42 too, because it selects the identical
target over the same per-point endpoint stream. **That is the unresolved contradiction**: with the
fire endpoint and the target identical on both sides, the block should be spliced at the same
point, yet Native emits it before the closing arc and Ares after it. The remaining candidate is
where the rendered block is spliced relative to the adjacent arc-fit emission
(`constant.rs:138-148` versus `GCode.cpp:10156-10180`), and it could not be confirmed at runtime:
`GCode.cpp:9986` resolves once the new image is loaded (`catch exec`) and hits 31 967 times, but
`endpoint_scaled`, `m_origin`, `endpoint_mm` and `farthest_gcode_pos` are all optimised out at
that inlined site, so no runtime state is recoverable without reading ABI registers by hand.

Earlier drafts of this document attributed the divergence to a frame mismatch (citing
`GCode.cpp:6507-6512`) and then to a scan-set difference. Both are withdrawn: the resolved
offsets show the endpoint frame is correct, and the GDB measurement shows the targets are
numerically equal.

## Deferred behaviour

- `timelapse_pos.rs` pos-picker semantics and `printed_objects` label ids;
- the layer-end fallback insert (`GCode.cpp:8193-8227`);
- non-inline traditional inserts and their position picking.

## Existing scaffolding replaced

`layers/farthest.rs` and `layers/farthest/target.rs` are already ported to this upstream boundary
and are kept; the change is confined to target selection inside `target.rs` once the open question
above is settled. No Ares-owned pipeline concept is introduced, and no new module is created.

## Verification

The seeded reference lane must show the block preceding the closing arc, i.e. the 14 differing
line positions inside 91886-91899 disappear while the total line count stays 263837. Two cases
must be locked:

- a printer whose `extruder_offset`/`xy_offset` are non-zero asserts the layer-44 block position
  matches Native;
- a zero-offset printer asserts byte-identical output before and after the change.

## Citations to correct while implementing

- `layers/layer_end.rs:2` cites `GCode.cpp:5527-5546`, which is the M205 / bed-temperature region
  at `40ecc66f`, not the layer-end timelapse insert.
- `emit_helpers.rs:105-113` states the BBL block arrives at `GCode.cpp:5205-5210`; it is
  `:7413-7420`.
- `layers/boundary.rs` body ranges `:4667-4685`, `:6704-6711`, `:5690-5718` do not match the
  measured emission order.
