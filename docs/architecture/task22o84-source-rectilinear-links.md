# Task 22O.84 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Replace the O78 approximate link implementation with the pinned OrcaSlicer
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:994-1214`, in
`fill::rectilinear::links`.

Connection selection uses O83 directed contour-segment distance with source
wraparound and strict first-tie retention. It considers adjacent same-kind
intersections and same-line opposite-kind intersections, classifies horizontal,
up, or down links, invalidates vertical arcs that skip inner intersections or
remain trapped on one side, measures exact retained contour arc length for the
maximum-length gate, and mirrors invalid vertical quality.

The lines-only O78 API is removed; link construction requires the O82 retained
slice. O79-O81 consume the corrected topology without compatibility fallback.

Deferred: region path costs, ant chaining, polyline emission, filler entities,
lifecycle, and G-code. No fixture branch or public API.

Compile RED rejected the retained-slice seam while O78 still accepted bare
lines. Two focused tests now pass for directed wraparound selection and strict
perimeter-arc length gating; all 15 O77-O83 regressions pass. Strict core
Clippy, rustfmt, diff, approximation-removal, and sub-400-LOC gates pass. The
link and focused test shards are 281 and 84 LOC.
