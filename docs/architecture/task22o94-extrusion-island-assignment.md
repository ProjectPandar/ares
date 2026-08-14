# Task 22O.94 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/GCode.cpp:4970-5048`, limited to single-region/single-extruder
extrusion-to-layer-island assignment.

For each retained ordered `lslice`, compute its contour bounding box and test
order by increasing box area. Assign generated fills, then appended thin fills,
then perimeter collections by their first point using source half-open box and
contour containment rules. Keep the source fallback island after all slices.
Preserve ownership and within-kind source order.

KSR currently has one region and one extrusion tool. Multi-region/tool override,
wiping, role splitting, island traversal/chaining, motion, and G-code are
deferred. No fallback to the legacy Ares pipeline and no fixture branch.

The KSR oracle freezes 3,350 total islands, 2,881 nonempty islands, zero
nonempty fallback islands, 1,658 generated fill collections, 2,285 thin fills,
and 2,881 perimeter collections. Of nonempty islands, 1,835 are perimeter-only
and 1,046 contain both; none are infill-only. Three assignment/repeatability/
lifecycle tests and strict core Clippy, rustfmt, diff, and sub-400-LOC gates
pass. Implementation/test shards are 188/84 LOC.
