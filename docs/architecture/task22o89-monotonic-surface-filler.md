# Task 22O.89 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillBase.cpp:255-324` and
`src/libslic3r/Fill/FillRectilinear.cpp:2755-2908,3404-3421`, into
`fill::rectilinear::surface`.

Accept explicit source-derived fill parameters, derive infill direction,
rotated retained contours, density spacing, full-solid spacing adjustment,
source offsets, line count/x origin, exact link gate, O79-O88 monotonic graph,
and inverse rotation. Refactor O82 to populate vertical lines into already
retained contours without recomputing offsets.

This remains a core filler API; project graph/entity integration is deferred to
the next slice. No hidden defaults, fallback, or fixture branch.

Deferred: grouped-fill extrusion entities, public lifecycle, motion, and G-code.

Compile RED proved the missing surface module. Two focused tests pass for exact
eight-point adjusted-solid output, fixed versus alternating direction,
repeatability, and input immutability; all five O77/O88 boundary regressions
pass. Strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass. Surface and
focused test shards are 139 and 71 LOC; refactored segments are 301 LOC.
