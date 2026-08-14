# Task 22O.91 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the layer/project ownership part of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/Fill.cpp:1213-1384`, into the
`project_slice::fill_entities` stage.

After bridge-over-infill mutation and infill-combination gating, visit every
object and planned layer in aligned source order and materialize O76/O90 layer
collections transactionally. Own the complete entity graph beside
`PreparedPostInfillCombination`; on any error dispose the predecessor and expose
no partial stage. Advance the public lifecycle to this stage, then explicitly
dispose it at the still-incomplete downstream boundary.

Deferred: thin fills, perimeter/fill ordering, motion planning, and G-code. No
fallback or fixture branch.

The first full KSR traversal exposed source-prerequisite mismatches: O77 now
applies the pinned endpoint-overlap classification, O79 tests vertical
connectivity on either contour side, and O80 materializes the zigzag reachability
invariant assumed by the source cost/emission assertions before extending a
region. These are general topology rules, not fixture branches.

Three O91 tests pass for all-layer KSR materialization, exact repeatability,
metadata validity, ownership hooks, and public lifecycle disposal. O79/O80/O90
regressions and strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass.
The stage and focused test shards are 139 and 59 LOC.
