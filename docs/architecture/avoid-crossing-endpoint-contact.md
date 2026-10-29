# Avoid-crossing simplification endpoint-contact predicate

## Upstream ownership and Rust destination

Rewrite `GCode/AvoidCrossingPerimeters.cpp::FirstIntersectionVisitor::operator()`
(lines 92–119), consumed by `simplify_travel` (435–479), using its actual
`Geometry.hpp::segments_intersect` (123–173) rather than the intersection-point
routine used by `AllIntersectionsVisitor`.
Rust destination is `gcode_emit/motion/path/avoid_crossing/router/path_shape.rs`
(`crosses_boundary` and its private predicate). The existing route adapter is
only a temporary carrier around the upstream travel concepts, not an Ares
routing design. No generator, writer, cooling or public API shape changes.

## Included semantics

Preserve the upstream integer-vector cross signs, sign-product tests and
collinear projected interval test. A proper crossing is a collision. The
source predicate does not classify a sole non-collinear shared endpoint as a
collision when both sign products are zero and neither segment is collinear
with the other. Collinear overlap remains a collision. This is not equivalent
to asking whether the floating intersection-point routine returned a point.
Do not apply the predicate to `collect_intersections`: that visitor still
needs actual intersection coordinates and is outside this slice.

Included: the FirstIntersectionVisitor consumer in existing simplification.
Deferred: other geometry predicate consumers, intersection collection,
heuristic simplification, routing boundary construction, thin-gap feedrates,
orthogonal wipe discrepancy, UI/runtime. No general cleanup or broad API.

## Evidence, failure modes and E2E contract

On main `3b5ebb363`, Native double replay in official and seeded modes is
stable. The old odd-kernel input differs by exactly one extra 21-byte travel
line after the first raised XYZ waypoint. Native and Rust debugger traces
show identical 10 input TravelPoints and do_not_remove flags; Native removes
the nearby unprotected vertex, Rust retains it. Native writer sees the
protected waypoint only once. Quantized-output deduplication would hide the
routing error and is prohibited.

Minimization: two-extruder beam0 with only `layer_height=0.25` still has two
extra travel lines (42 bytes), while a single-volume height-0.25 control
matches completely. Disabling arcs does not remove the discrepancy. Thus
this is not an odd voxel-kernel implementation change. Keep the original
odd-kernel witness unchanged to verify the original report as well.

Failure modes to guard: retaining an avoidable tangent waypoint; erasing a
real crossing; losing collinear overlap; accidentally changing intersection
collection; masking the difference with writer/normalizer changes.

Confirmed seams remain Native CLI twice, core byte-in/options API and CLI.
Freeze the minimized beam0 target, original odd-kernel target and original
single-volume PASS control as input-only 3MFs, preserving SHA assertions.
Both full Native/API and Native/CLI ordered-byte comparisons use the existing
generator-only normalizer, and API/CLI must be raw-byte equal. Record launch,
input, Native replay and comparison artifacts; no committed Native output,
probe, ignores, unsupported guards or fallback.

## Sequence and gates

Commit spec, then tests with genuine fresh runtime RED before production.
GREEN must be production-only and make the entire admitted target streams
match, not merely reduce the difference. Re-run original report and controls.
Every changed file remains below 400 LOC; fmt and workspace/all-target clippy
with `-D warnings` clean. Review only after both complete FULL modes are
0 FAIL/0 skipped on the exact final hash based on then-current main.
The original WIP branch and other unresolved tests are not replayed here.
