# Task 22O.50 — nearest anchor-line AABB tree

## Status

Implemented after independent source/specification approval; final independent implementation review approved unconditionally.

## Goal and source boundary

Port the indexed-line `LinesDistancer` reached by Orca commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject.cpp:2849-2930::determine_bridging_angle`:

- `AABBTreeLines.hpp:20-35` indexed-line primitive distance;
- `AABBTreeLines.hpp:165-220,300-365` tree construction/query surface;
- `AABBTreeIndirect.hpp:41-235` implicit tree and QuickSelect build;
- `AABBTreeIndirect.hpp:556-640` nearest recursive traversal;
- `Line.hpp:43-70` segment projection and squared distance;
- `Utils.hpp:307-359` power-of-two allocation;
- Eigen 5.0.1 `AlignedBox.h:415-429::squaredExteriorDistance`, pinned by
  `deps/Eigen/Eigen.cmake` SHA-256
  `0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`.

The Rust destination is `geometry/line_distance_tree.rs`, split into ordinary
submodules before any source file approaches 400 LOC, with tests under
`geometry/tests/line_distance_tree/`.

## Interface

```rust
pub(crate) struct LineDistanceTree<'a> { /* borrowed lines, owned nodes */ }
pub(crate) struct NearestLine {
    pub(crate) line_index: usize,
    pub(crate) squared_distance: f64,
    pub(crate) nearest_point: [f64; 2],
}
impl<'a> LineDistanceTree<'a> {
    pub(crate) fn new(lines: &'a [Line]) -> Self;
    pub(crate) fn nearest(&self, point: Point) -> Option<NearestLine>;
}
```

`geometry.rs` owns the private module and crate re-export. Under `cfg(test)`, a
read-only node snapshot exposes vector length, unused/inner/leaf state, source
index, and bbox; production does not expose tree internals.

## Required behavior

1. Empty lines produce an empty node vector and `None` for every query.
2. For each line, preserve its input index, inclusive endpoint AABB, and
   centroid calculated as source integer endpoint sum multiplied by `0.5` then
   truncated toward zero per coordinate.
3. Allocate `next_power_of_two(line_count) * 2 - 1` default nodes and build at
   implicit child indices `2n+1`, `2n+2`.
4. At every inner node, union inclusive input bounds, select X when X/Y
   diagonal lengths tie, and partition the active input interval at
   `(left+right)/2` with exact median-of-three QuickSelect comparisons and
   swaps. Preserve strict comparisons and equal-centroid permutation behavior.
5. Convert the input `Point` once from i64 coordinates to an f64 origin. For
   primitive distance, truncate that f64 origin back to i64 first, then
   calculate widened endpoint/roundtripped-query differences and promote each
   component to f64. Calculate `l2 = vx*vx + vy*vy` and
   `dot = vax*vx + vay*vy` in X-then-Y order without `mul_add`. Degenerate,
   `t<=0`, `t>=1`, and interior branches preserve source order. Interior
   nearest point truncates `a + t*v` toward zero. Interior squared distance is
   exactly `(t*vx-vax)^2 + (t*vy-vay)^2`, X then Y; endpoint distances likewise
   multiply X then Y. Do not substitute projected-coordinate subtraction,
   reuse `Line::distance_to`, or fuse operations.
6. Recursive containment truncates the shared f64 origin back to i64 and tests
   inclusive bounds left then right. Pinned Eigen bbox distance instead
   compares each i64 bound against the f64 origin, performs the selected
   subtraction in f64, truncates that result into fixed i64 `aux`, then squares
   and accumulates X followed by Y in the fixed scalar before promoting the
   completed sum to f64. If left is strictly smaller, visit left then
   right; otherwise visit right then left. Enter an unvisited child only when
   bbox distance is strictly below the current winner.
7. A primitive replaces the winner only for strictly smaller squared distance.
   Equal distances retain the first primitive reached by the exact tree
   traversal, never the lowest input index by fallback.
8. Return original line index, squared distance, and the nearest integer
   projection promoted to `[f64; 2]`, matching `LinesDistancer`'s public result.
   Borrowed lines/query remain unchanged; repeated finite-domain builds and
   queries are bitwise identical.
9. The trusted slicing caller supplies coordinates already accepted by the
   Clipper closed-path `HI_RANGE` boundary. The i64-to-f64-to-i64 query
   roundtrip and mixed-scalar Eigen bbox delta are required defined source
   behavior, including above 2^53 and at `HI_RANGE`; never substitute the
   original query integer. Rust uses i128 only after those source conversions
   for centroid sums, bbox diagonals, segment differences, and fixed bbox
   delta-square X-then-Y accumulation. Where C++ i64 intermediates are defined,
   this matches before the same final cast. Beyond that subset, widened i128 is
   intentional deterministic Rust behavior over C++ signed-overflow UB, not an
   upstream parity claim. `None` remains reserved for empty input.

## Included and deferred

Included: only balanced indexed-line build and unsigned nearest query.

Deferred: signed distance/outside tests, radius queries, line intersections,
triangle trees, automatic bridge direction sampling/map windows/pattern
adjustments, O49 composition, transaction activation, anchored polygon
construction, extrusion, motion, G-code, and CLI parity.

## Acceptance

Use a compiling behavioral RED. Focused literal tests cover empty/single,
degenerate and every projection branch, negative centroid truncation, each
median-of-three swap branch, equal centroids, longest-axis tie,
non-power-of-two layouts, containment visits, bbox-distance right-first ties,
strict pruning, equal primitive-distance ownership, nearest-point truncation,
input nonmutation, and repeatability. Boundary tests cross native i64 add/sub
hazards and exercise values above 2^53 plus the declared `HI_RANGE` maximum,
distinguishing original-point arithmetic, the roundtripped primitive query,
mixed-scalar Eigen bbox deltas, fixed accumulation, and per-axis f64
accumulation.

Freeze literals from a standalone C++ driver that instantiates the actual
pinned Orca templates and Eigen implementation rather than recreating them.
Freeze node-vector length; unused/inner/leaf slots; source indices/bboxes;
centroids above f64's exact-integer range; raw fixed bbox-distance values and
final f64 bits; containment overlap; bbox ties; strict pruning equality; and
primitive ties. Include a source-safe fixed accumulation case differing from
per-axis f64 accumulation. Separately freeze Rust i128-extension literals
beyond C++ defined arithmetic. Driver output may derive committed literals but
is never read, compiled, or run by Rust tests. Remove temporary artifacts and
leave the Orca checkout byte-clean.

All changed Rust files stay below 400 LOC and use no `include!`,
`include_bytes!`, or `include_str!` splitting. Final gates are focused and
geometry/O43-O50 dependency Nextest, workspace Nextest, rustfmt,
warning-denying Clippy,
`cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`,
diff/LOC/static audits with no new wasm-bindgen API, and independent six-axis
repair/re-review until approval.

## Implementation evidence

The compiling behavioral RED failed 0/5, and the source-shaped implementation
then passes focused 8/8, dependency 613/613, and workspace 6,273/6,273
Nextest. Warning-denying workspace Clippy, rustfmt, core/browser wasm32,
diff/LOC/static audits also pass. Literal expectations came from a temporary
standalone driver that instantiated the actual pinned Orca and Eigen templates;
the Rust test suite has no runtime dependency on that driver or checkout.
