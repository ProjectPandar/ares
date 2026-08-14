# Task 22O.83 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:38-116,459-685`, into
`fill::rectilinear::perimeter`.

Provide directed contour-segment distance, forward contour-arc length,
forward/reverse intermediate-vertex append, and adjacent-line/same-line
perimeter measure and emit primitives over the exact O82 retained contour
inventory. Source segment indices identify edge endpoints; emitters exclude the
first intersection and include the final intersection.

Split rectilinear tests into normal Rust modules before adding the focused O83
oracle. No `include!` or other source-splitting macro is used.

Deferred: replacing O78 approximate link selection/quality, region lengths,
ant chaining, final polylines/entities, lifecycle, and G-code. No legacy
fallback, fixture branch, or public API.

A RED same-segment reverse-append oracle exposed and removed an incorrect full
loop. Two focused tests now pass for directed/wrapped lengths and exact indexed
emission; all seven O77-O79 regressions pass. Strict core Clippy, rustfmt, diff,
and sub-400-LOC gates pass. The perimeter and focused test shards are 181 and
62 LOC.
