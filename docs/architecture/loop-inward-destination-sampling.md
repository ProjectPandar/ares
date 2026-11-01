# Loop inward destination sampling

Status: tagged OPEN — source-correct NON-GREEN port, blocked on M73 placement.

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
`project_slice/gcode_emit/motion/loop_paths/inward.rs::{append_inward_move,inward_destination,sample_path_at_distance}`
and its existing loop-path caller and logical-cursor fields in `EmitState`.
The old first/second/end-3vertex angle construction and writer-only cursor
update are replaced by the cited path sampling and logical-position update.
The port reuses existing clipping helpers, preserves original loop winding,
truncates interpolated coordinates, and retains source rotation/epsilon rules.
It does not introduce an independent Ares travel policy or legacy fallback.

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

The RED originally added only the public test, option-domain registration and
this record. Its test, ROOT entrypoint, producer selection and fixture bytes
remain unchanged by the source port. No probe, private planner test, mock or
copied solver expectation is introduced.

The separately approved source-correct port covers only literal destination
sampling and caller/cursor semantics. Support-offset wipe-inward optimization,
generic routing, travel-count differences, M73/statistics, raw junction widths,
seam compensation and geometry-kernel policies remain deferred. No rounding
or EPS tuning is permitted.

## Tagged OPEN: blocked on M73 placement

The source-correct port aligns the inward XY, wipe endpoint and connector
records with Native. The complete Z0.34layer remains FAIL: both sides contain
4931bytes/197lines, but `M73 P10 R12` appears across the neighboring extrusion
at layer line118. This belongs to the writer/time boundary and remains OPEN;
it is not attributed to inward destination geometry. The RED and its strict
whole-layer assertion stay unchanged, including M73. No mask or GREEN claim.

The adjacent anchor travel-count observation changes from Ares774 to846,
with Native836: FAIL-to-FAIL. This is an accepted observed consequence, not a
travel-count fix, clean parity result or root-cause claim. The original travel
assertions remain intact. Publication of the source-correct NON-GREEN port
requires the authorized broader before/after audit to show zero PASS-to-FAIL
transitions and to list every status change.

Neither this source-correct acceptance nor the tagged OPEN status closes the
whole layer, whole file, routing family or writer/time issue.
