# Task 22O.88 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:2584-2753`, into
`fill::rectilinear::emit`.

Traverse O87 region/orientation chains over O84 links. Emit outer endpoints,
inner vertical runs, O83 forward/reverse same-line and adjacent contour arcs,
split disconnected runs, remove duplicate/near-zero paths using the active
coordinate scale, and merge paths split only by O79 phony pinch points. Preserve
source path and point order.

Deferred: full `fill_surface_by_lines` orchestration/rotation, filler extrusion
entities, lifecycle, and G-code. No fallback, fixture branch, or public API.

Compile RED proved the missing emitter. Two focused tests pass for empty output,
exact six-point rectangular zigzag, repeatability, and immutable inputs; all
three O87 regressions remain green. Strict core Clippy, rustfmt, diff, and
sub-400-LOC gates pass. Emitter and focused test shards are 225 and 46 LOC.
