# Task 22O.85 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:1989-2077,2179-2188`, into
`fill::rectilinear::costs`.

Traverse each O80 region from both left boundary orientations over corrected
O84 links. Preserve source f32 accumulation, half-weighted valid perimeter
connections, straight gap distance, vertical-run endpoint selection, coordinate
scale unscaling, and subtraction of the common minimum so exactly one stored
orientation cost is zero.

Deferred: inter-region path matrix, ant chaining, polyline emission, filler
entities, lifecycle, and G-code. No fallback, fixture branch, or public API.

Compile RED proved the missing cost module. Two focused tests pass for symmetric
normalization, asymmetric exact f32 bits, scale-specific rounding,
repeatability, and input immutability; both O84 regressions remain green. Strict
core Clippy, rustfmt, diff, and sub-400-LOC gates pass. The cost and focused test
shards are 147 and 56 LOC.
