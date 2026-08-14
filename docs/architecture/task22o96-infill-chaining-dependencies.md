# Task 22O.96 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the pure chaining dependencies from pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/ShortestPath.cpp:15-40,92-393,1026-1069`;
- `src/libslic3r/ExtrusionEntityCollection.cpp:65-72,87-96`;
- `src/libslic3r/ExtrusionEntityCollection.hpp:78-123`;
- `src/libslic3r/Fill/FillBase.cpp:161-185`.

Generalize the reached project-slice classic-perimeter shortest-path seam rather
than the independent `geometry::chain_points` seam. Add explicit-cursor,
reversal-constrained entity chaining, source fallback nearest-neighbor ordering,
collection endpoint/reversal operations, and `chained_path_from`. Retained gap
paths reverse; retained gap loops are orientation-eligible during chaining but
ignore the selected reversal, preserving source winding.

`FillExtrusionCollection::no_sort` is owned source state: Monotonic and
MonotonicLine set it, while CrossHatch does not. A no-sort collection preserves
its internal path order and cannot reverse as a top-level entity.

The reached KSR patterns are CrossHatch, Monotonic, and MonotonicLine. Adjacent
`FillBase.cpp:164-185` flow-calibration and Grid path reversal locks are deferred
because KSR's `calib_flowrate_topinfill_special_order` is false and Grid is not
reached; their state is not represented as a false compatibility default.

This milestone is pure and is not invoked by `slice_project`; no cursor is
invented. O95 runtime integration, its real current-position cursor, multi-region
role filtering, motion, and G-code remain deferred. No fixture branch or legacy
pipeline fallback.

Four O96 entity tests and all ten reached shortest-path regressions pass. The KSR
inventory freezes 782 no-sort and 876 sortable generated collections, all with
valid endpoints. Strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass;
the largest changed Rust shard is 383 LOC.
