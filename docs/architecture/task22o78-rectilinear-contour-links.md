# Task 22O.78 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:994-1214`, into
`fill::rectilinear::links`.

The module enriches O77's ordered vertical intersections with source
previous/next contour links. Candidate links are selected on adjacent vertical
lines by contour identity, kind, and minimum source segment distance; same-line
opposite-kind candidates may replace them. The result records horizontal/up/down
link kind and valid/invalid/too-long quality, including source symmetry of
invalid same-line links.

Included: contour orientation/distance, candidate selection/order, same-line
replacement, inner-intersection invalidation, don't-connect and max-length
quality gates, deterministic owned output, and immutable geometry input.

Deferred: pinch insertion, monotonic-region generation/chaining, ordinary graph
traversal, complete line filler, entities, lifecycle, and G-code. No old
`infills` fallback, fixture branch, or public API is added.

Two focused tests pass for adjacent horizontal symmetry and don't-connect /
maximum-length quality. O77's three segmentation regressions remain green.
Strict core all-target/all-feature Clippy, rustfmt, diff, and sub-400-LOC checks
pass; the new link shard is 198 LOC.
