# Task 22O.90 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the monotonic-active part of pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/Fill.cpp:1213-1374` and
`src/libslic3r/Fill/FillBase.cpp:133-155`, into
`project_slice::fill_entities::monotonic`.

The graph-native layer entity pass now dispatches CrossHatch, Monotonic, and
MonotonicLine. It derives every O89 parameter from grouped 3MF/effective graph
state, including percent density, role flow, layer/thickness, bridge angle,
fixed angle, overlap, anchor policy, and pinned density-gated `3 × spacing` link
length. MonotonicLine applies the source zero maximum anchor before filling.
Each nonempty expolygon produces one ordered extrusion collection with exact
role/flow metadata.

Deferred: remaining filler classes, thin fills, lifecycle, motion, and G-code.
No fallback or fixture branch.

Compile RED proved the graph-native layer seam lacked Monotonic dispatch. Two
focused tests pass for exact internal-solid flow/point metadata, Top
MonotonicLine disconnection, repeatability, and graph immutability; all three
O76 and both O89 regressions pass. Strict core Clippy, rustfmt, diff, and
sub-400-LOC gates pass. Implementation and focused tests are 60 and 82 LOC.
