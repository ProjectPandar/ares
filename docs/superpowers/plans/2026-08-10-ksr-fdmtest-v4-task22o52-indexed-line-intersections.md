# Task 22O.52 indexed line query implementation plan

## Status

Implemented and verified. The pre-RED source/specification review approved unconditionally; focused, dependency, workspace, portability, static, and mutation gates pass.

## Objective

Extend O50 with the exact sorted-intersection and closed-contour outside queries
reached by pinned `PrintObject.cpp::construct_anchored_polygon`, without yet
porting anchor scanline reconstruction or activating the slicing lifecycle.

## Plan

1. **Freeze source semantics**
   - Independently review ADR/spec against pinned `AABBTreeLines.hpp`,
     `AABBTreeIndirect.hpp`, `Line.hpp`, `Point.hpp::cross2`, hash-pinned Eigen
     `AlignedBox`, ARD-0024's fixed-MSVC STL 14.44 sort compatibility seam, and
     the construct caller.
   - Compile a temporary standalone C++ driver using the actual pinned sources
     to freeze tree traversal, intersection/index/pre-sort-key literals,
     contour classifications, mixed-axis fallback, and source-safe boundary
     behavior. Freeze equal-key output with the accepted fixed-MSVC oracle, not
     Linux `std::sort`, including a `>32` fixture.

2. **Behavioral RED**
   - Add ordinary `intersections.rs`/`outside.rs` production submodules and
     ordinary geometry test children.
   - Register compiling `LineDistanceTree` methods with `todo!()` bodies.
   - Add literal tests for all acceptance branches and retain the focused RED.

3. **Minimal implementation**
   - Reuse O50 nodes/bounds and existing source-shaped `Line::intersection` only
     after its arithmetic is proven by the new literals.
   - Implement left/right bbox-pruned collection, source-key calculation and
     fixed-MSVC sorting; implement coordinate-ray recursion and exact X/Y
     parity classification.
   - Add no generic tree abstraction, polygon conversion, deduplication,
     validation, fallback, or lifecycle wiring.

4. **Verify, mutate, and review**
   - Run focused, O43-O52/geometry dependency, workspace, rustfmt,
     warning-denying Clippy, core/browser wasm32, diff/LOC/static audits.
   - Run reversible mutations for the acceptance-critical traversal,
     ownership, parity, and sort branches and restore byte-exact, while recording that the reached integer
     determinant makes the `<1e-4` threshold itself non-discriminable.
   - Record evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent six-axis implementation review; repair and re-review until
     unconditional approval.

## Exit criteria

- Intersections, indices, ordering, and outside classifications match pinned C++
  literals exactly.
- Borrowed lines/query remain unchanged and all tie/vertex ownership is frozen.
- Production stays crate-private, portable, and lifecycle-neutral.
- Every file is below 400 LOC with ordinary modules and no include macros.
- All runtime/static/mutation gates and independent review approve.
