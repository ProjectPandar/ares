# Task 22O.52 — indexed line intersections and contour outside

## Status

Implemented and verified. Independent source/specification review approved RED; independent implementation review follows the recorded runtime and mutation gates.

## Goal and source boundary

Port the indexed-line queries reached by pinned Orca commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` at
`PrintObject.cpp:2975-3001::construct_anchored_polygon`:

- `AABBTreeLines.hpp:119-156` recursive line intersection;
- `AABBTreeLines.hpp:269-296,359-362` sorted public intersection query;
- `AABBTreeLines.hpp:35-117,239-266,325` coordinate-ray/outside query;
- `Line.hpp:123-148::line_alg::intersection` and
  `Point.hpp:104-114::cross2`;
- `AABBTreeIndirect.hpp:39-211`, already implemented by O50;
- pinned Eigen 5.0.1 inclusive `AlignedBox::intersects` behavior, archive
  SHA-256 `0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`
  from `deps/Eigen/Eigen.cmake:6-9`;
- ARD-0024's accepted MSVC STL 14.44 sort compatibility target.

The Rust destination is ordinary `geometry/line_distance_tree/intersections.rs`
and `outside.rs` submodules on O50's existing borrowed tree. Tests are ordinary
children of `geometry/tests/line_distance_tree.rs`.

## Interface

```rust
impl LineDistanceTree<'_> {
    pub(crate) fn intersections_sorted(&self, line: Line) -> Vec<(Point, usize)>;
    pub(crate) fn outside(&self, point: Point) -> i32;
}
```

Returned indices are original borrowed-line indices. Inputs remain unchanged.

## Sorted intersection behavior

1. Empty trees return an empty vector.
2. Build the query segment's inclusive endpoint AABB. At each inner node, visit
   the left child first if its inclusive bbox intersects the query bbox, then
   the right child under the same condition. Append leaf hits in traversal
   order.
3. Leaf intersection calculates source i64 endpoint differences before f64
   promotion, then `cross(v1,v2)` in source order. If
   `abs(denominator) < 1e-4`, return no hit, including collinear overlap.
4. Calculate `v12 = query.a - source.a`, `nume_a = cross(v2,v12)`,
   `nume_b = cross(v1,v12)`, and `t1/t2` in source order. Accept both endpoints
   with `0 <= t <= 1.0f`. Return
   `query.a.cast<f64>() + t1*v1`, truncating each result coordinate to i64.
5. For every hit, calculate the source sort key from the truncated point minus
   `query.a` in i64, then X/Y promotion, `dx*dx + dy*dy` in order. Apply
   ARD-0024's audited MSVC STL 14.44 `std::sort` flow with comparator
   `left.key < right.key`. Do not stabilize, deduplicate, tie-break by
   index/point, sort geometrically by parameter, or use host `sort_by`. Linux
   `std::sort` order is diagnostic only.
6. Return sorted `(Point, original_line_index)` pairs; repeated source-safe
   calls are identical and leave lines/query unchanged.

## Outside behavior

1. Empty trees classify every point as outside `1`.
2. Run the coordinate-aligned ray counter first with tested coordinate X and
   other coordinate Y. Inner nodes recurse left then right only when
   `bbox.min[other] <= origin[other] <= bbox.max[other]`; a negative child count
   returns `(-1,-1)` immediately.
3. At a leaf, reject the segment when
   `origin[other] < min(other) || origin[other] >= max(other)`. The non-sharp
   second inequality is required for shared-vertex ownership.
4. If origin's tested coordinate is strictly above the segment maximum, return
   `(1,0)`; if strictly below its minimum, return `(0,1)`. Otherwise calculate
   source integer differences followed by f64 division and
   `value = a[test] + t * (b[test]-a[test])`. Greater/less returns positive or
   negative ray count; exact equality returns `(-1,-1)`.
5. Boundary/negative counts classify as `0`. Odd/odd classifies inside `-1`;
   even/even classifies outside `1`. Mixed parity repeats steps 2-4 with tested
   coordinate Y. Its boundary or another mixed result classifies `0`.
6. Preserve source line/tree/input order, exact parity, and i32 hit addition.
   Do not substitute polygon winding, Clipper point-in-polygon, epsilon tests,
   or a repaired malformed-contour answer.

## Trusted domain and deferred behavior

Borrowed tree-line endpoints, intersection-query endpoints, and the `outside`
query point must keep every reached source signed integer operation
representable, including coordinate-ray differences and sort-key subtraction.
Every computed f64 intersection coordinate must remain finite and representable
as source `coord_t`. Clipper closed-path `HI_RANGE` proves only the borrowed
contour/anchor-edge portion; it does not prove future query scanlines after
spacing subtraction/addition or the caller's outside midpoint addition before
division. Both generated-arithmetic proofs are deferred to the construction
milestone. Closed contour callers also provide valid edge sets and ray hit
totals fitting source `int`. Near-`HI_RANGE` literals stay in this defined C++
domain rather than freezing Rust saturation/overflow. These are internal
preconditions, not validation branches.

Deferred: unsorted/radius/signed-distance APIs, line rotation, anchor scanline
construction, section extension/merge/sort, traced polygon reconstruction,
safety union, O49/O51 composition, candidate transaction/commit, public
lifecycle, extrusion, motion, G-code, and CLI parity.

## Acceptance

Use a compiling behavioral RED. Freeze non-tie literals from a temporary standalone C++ driver instantiating
the actual pinned tree/query/Line/Eigen code. For equal keys, separately freeze
the pre-sort tuple vector, then obtain the normative output from the exact
accepted MSVC STL 14.44 toolset or ARD-0024 fixed-sort oracle; Linux
`std::sort` remains diagnostic only. Tests must
discriminate:

- empty/single and non-power-of-two trees; left-before-right traversal and bbox
  inclusive/exclusive boundaries;
- proper crossing, endpoint hits, exact-zero parallel/collinear rejection, the
  smallest nonzero determinant `±1`, a high-magnitude cast/cancellation case,
  truncating negative/fractional intersections, and original index. For the
  reached integer `Line`, determinant magnitudes are zero or at least one, so
  the `<1e-4` comparator itself is not separately mutation-discriminable and no
  floating-line abstraction is introduced;
- fixed-MSVC sorted distance ordering with a separately pinned pre-sort tuple
  vector, duplicate geometric hits retained, X-then-Y key bits, and a normative
  `>32` equal-key fixture that differs from stable/host sorting. A separate
  small equal-key fixture exposes left/right traversal through the sorted API;
- outside empty, ordinary inside/outside, contour boundary, shared vertices,
  horizontal/vertical edges, holes/multiple contours, X mixed-parity Y retry,
  and malformed second mismatch returning zero;
- values near `HI_RANGE`, repeatability, and complete borrowed-input
  nonmutation.

Add a reversible mutation audit for max-exclusive leaf ownership, parity
fallback, intersection traversal order, and equal-distance sort behavior. Rust
tests do not read/compile/run oracle artifacts; remove temporary artifacts and
leave Orca byte-clean.

All files remain below 400 LOC and prohibit `include!`, `include_bytes!`, and
`include_str!` splitting. Final gates pass: focused 8/8, O43-O52/geometry dependency 630/630, workspace 6,290/6,290, rustfmt, warning-denying Clippy, core/browser wasm32, diff/LOC/static audits, and five reversible mutations, including cast-before-subtraction sort-key arithmetic. Temporary artifacts were removed and Orca remains byte-clean. Independent six-axis implementation review and any required repair/re-review complete this milestone.
