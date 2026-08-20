# Spec: Task 22O.244 closing offset coordinate determinism

## Observable contract

The first KSR first-layer perimeter emits `G1 X141.072 Y101.525 E.02865` followed by `G1 X140.573 Y102.11 E.02866`, matching OrcaSlicer 2.4.2. The result is derived from project geometry and configured closing offsets; production code does not inspect fixture names, reference G-code, or known coordinates.

## Upstream boundary

Rewrite OrcaSlicer 2.4.2 `src/libslic3r/ClipperUtils.cpp:353-410`, specifically `raw_offset`, `shrink_paths`, and `offset_paths<ClipperLib::PolyTree>`, at `geometry::clipper::offset_paths_tree`. Ares applies the signed delta to each oriented path before the final union so its Clipper rewrite produces the same deterministic closing coordinates. This is an implementation-local seam; callers continue to receive a `PolyTree`.

Included: positive and negative closed-path offset trees used by two-stage closing. Deferred: rectilinear fill offsets, later G-code differences, timing/M73, and final statistics.

## Acceptance

A focused `slice_project` test observes the exact adjacent E words. The source-stage root-order test is removed because it pins an internal artifact rather than output behavior. Changed Rust files remain below 400 lines and pass focused nextest before the slice is committed and pushed independently.
