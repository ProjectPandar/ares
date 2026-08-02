# Task 22O.4: Classic loop materialization and hierarchy

## Fixed rewrite boundary

This milestone rewrites OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`: `PerimeterGenerator.cpp:34-55`,
the loop conversion implied by lines 1353-1369, and the destructive hierarchy
loops at lines 1388-1443. Exact containment comes from `Polygon.hpp:66`,
`Polygon.cpp:722-729`, and Clipper v6 `deps_src/clipper/clipper.hpp:92-96` and
`deps_src/clipper/clipper.cpp:216-253`, including the active
`CrossProductType = double` determinant. The Rust destination is
`ares-core::project_slice::perimeters::classic::hierarchy`.

Task 22O.3 remains the immutable raw-shell predecessor. O4 materializes its
stored normal then smaller-width ExPolygons, contour before holes, into typed
loops without rerunning offsets, gaps, collapse, top splitting, or depth zero.
It then performs the source hole-first and contour destructive first-parent
searches with boundary-inclusive, first-point-only containment. Roots preserve
`contours[0]`; unmatched internal contours and holes remain diagnostic buckets.

The successor nests O3, preserves aligned optional records and source surfaces,
and retains the boxed resolved configuration. Effective depth `-1` creates no
buckets; otherwise buckets cover every depth through the effective loop count.

## Deferred boundary

This stops before `traverse_loops` around line 1450. Traversal, extrusion
entities, thin walls, overhang processing, wall direction/sequence, gap medial
axes, fill remainder, seams, infill, motion, writer, post-processing, complete
Task 22O, and exact KSR G-code parity remain deferred.

## Verification

In-memory tests cover Clipper tri-state containment, exact materialization
flags/order/depth, destructive depth/index search order, first-parent and
first-point behavior, erase/retry, boundaries, roots and orphans. Real KSR tests
cover reachability, determinism, O3 preservation, and the public lifecycle,
which remains `ProjectSlicingIncomplete`.
