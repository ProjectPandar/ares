# Task 22O.82 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the retained-data boundary of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:357-457,759-993`, into
`fill::rectilinear::segments`.

Return one owned `RectilinearSlice` containing the rotated source expolygon,
ordered outer/inner offset contours, and O77 vertical lines. Intersection
`contour_index` and `segment_index` address this retained inventory, enabling
source contour-distance and perimeter-emission ports without reconstructing or
re-offsetting geometry.

The existing private lines-only function remains only as a temporary test shell
around this source object until O77-O81 callers migrate in the next slice; it
does not select behavior or provide fallback geometry.

Deferred: contour path measurement/emission, region lengths/chaining,
polylines/entities, lifecycle, and G-code. No fixture branch or public API.

Two focused tests pass for source/inventory retention, index addressability,
repeatability, nonmutation, and atomic range error. O77-O81 regressions passed
before this ownership-only change. Strict core Clippy, rustfmt, diff, and
sub-400-LOC gates pass; the segment and test shards are 279 and 381 LOC.
