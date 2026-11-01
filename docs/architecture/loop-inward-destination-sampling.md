# Loop inward destination sampling

Status: source-cited public RED qualification; no production change authorized.

## Upstream boundary

Pinned beta `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `GCode.cpp:8730-8736` selects the post-loop destination from actual extrusion
  paths when wipe-on-loops, external-wall role and multiple walls are eligible.
- `GCode/WipePathHelpers.cpp:886-926::wipe_on_loops_destination`, with its
  `sample_path_at_distance` helper, samples forward/backward over path length,
  clamps sample distance to a quarter of total length, rejects degenerate
  samples/angles, rotates the short forward sample and returns a destination.
- `GCode.cpp:8790-8794` emits that move and updates the logical last position.

The Rust destination is
`project_slice/gcode_emit/motion/loop_paths/inward.rs::{append_inward_move,inward_point}`,
its existing loop-path caller, and the existing logical-cursor fields in
`EmitState`. Current inward code still uses the old first/second/end-3vertex
angle construction and explicitly updates only writer x/y. It is a temporary
compatibility shell around the upstream loop-end concept, not authority for
an independent Ares travel policy. This source gap is not yet causal GREEN.

## Public RED seam

Reuse `tests/avoid_crossing_travel/two-walls.3mf` unchanged, SHA256
`ff7c08c7dc1339fe38fc24d23b207bf6edd686c8263c0531d46c9c5372fd644a`.
No new input or variant is introduced. The ordinary official beta Native
producer runs twice through `native_stream_case::qualify`, which asserts
whole-export stability after the existing single-generator-line normalization.
The clarified budget is official double replay plus one public API call, not
a seeded mode or an API/CLI probe matrix.

Assert Native effective Classic/two-wall/wipe-on-loops settings and the
observed inward destination X164.855Y179.87. Then invoke
`ares_core::slice_project_plate_validated` with deterministic metadata,
default overrides, default plate selection and context0.

Compare the ENTIRE ordered Z0.34layer, using the same layer boundaries as the
existing runtime travel test: from its Zmarker until the next layer marker.
Retain every XYZ/E, travel, feed, width/type, M73, retraction, wipe, object and
terminal record. No line filtering, tolerance, coordinate compensation, forced
corner, normalization inside the layer or alternative expectation generator.
The original three-layer/whole-stream travel tests remain unchanged.

The prior filtered observation first differs at layer line40:
Native `G1 X164.855 Y179.87`, Ares `G1 X164.851 Y179.88`.
The public API must independently expose the complete-layer discrepancy;
a failed Native premise, unstable export, API admission failure or unrelated
first difference is a failed qualification, not this production RED.

## Included and deferred behavior

This RED adds only the public test, option-domain registration and this record.
Production, ROOT test entrypoint, producer selection and fixture bytes are frozen.
No probe, private planner test, mock or copied solver expectation is introduced.

A later separately approved source port may cover literal destination sampling
and caller/cursor semantics only. Support-offset wipe-inward optimization,
generic routing, travel-count differences, M73/statistics, raw junction widths,
seam compensation and geometry-kernel policies are deferred. Do not attribute
those separate observations to this wall slice. No rounding/EPS tuning or
legacy fallback is permitted, and a RED requires review before any GREEN work.
