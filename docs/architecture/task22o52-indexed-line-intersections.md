# Task 22O.52 architecture decision record

## Status

Accepted and implemented. Independent source/specification review approved the boundary before RED; implementation review is recorded after the verification evidence below.

## Decision

Port the two remaining indexed-line queries directly reached by pinned Orca
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject.cpp:2975-3001::construct_anchored_polygon`:

- `AABBTreeLines.hpp:119-156,269-296,359-362::intersections_with_line<true>`;
- `AABBTreeLines.hpp:35-117,239-266,325::outside`;
- reached `Line.hpp:123-148::line_alg::intersection`,
  `Point.hpp:104-114::cross2`, and pinned Eigen 5.0.1 inclusive AABB overlap
  semantics, archive SHA-256
  `0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`
  from `deps/Eigen/Eigen.cmake:6-9`;
- `AABBTreeIndirect.hpp:39-211`, already implemented by O50.

The Rust destination extends the existing borrowed `LineDistanceTree` with
crate-private `intersections_sorted(Line) -> Vec<(Point, usize)>` and
`outside(Point) -> i32` operations, split into ordinary `intersections.rs` and
`outside.rs` submodules. No second tree or generic spatial abstraction is
created.

## Required semantics

Intersection traversal rejects an empty tree, builds the inclusive query-line
AABB, visits intersecting left then right children, and appends one leaf hit in
that traversal order. Segment intersection preserves source i64 subtraction,
f64 cross products, `abs(denominator) < 1e-4`, closed `t` bounds, and truncating
f64-to-i64 output. Collinear overlap is not an intersection. Sorted output
computes squared distance from each truncated hit to the query's first endpoint
in X-then-Y f64 order, then applies the already-audited fixed-MSVC STL 14.44 `std::sort` control flow
from ARD-0024 with strict distance comparison. Equal-distance ownership follows
that comparator/sort, not source index or a stable fallback. A Linux
`std::sort` oracle is diagnostic only; the normative tie permutation comes from
the accepted fixed-MSVC sort oracle applied to a separately pinned pre-sort hit
vector.

Outside classification first casts no coordinates and traces the source
coordinate-aligned X ray through the same implicit tree. Internal child bbox
checks are inclusive and left-before-right. A leaf rejects the other coordinate
outside `[min,max)`, classifies points beyond the tested-coordinate extent, or
computes the source f64 intersection value. Exact boundary returns negative hit
counts immediately. Odd/odd means inside `-1`, even/even means outside `1`, and
mixed parity retries the Y ray; another mismatch or boundary means `0`. Empty
contours return `1`.

The trusted domain requires borrowed tree-line endpoints, intersection-query
endpoints, and the `outside` query point to keep every reached signed integer
operation representable, including coordinate-ray differences and sort-key
subtraction. Every f64 intersection coordinate must be finite and representable
as source `coord_t`; parity counts fit source `int`, and closed contour line
sets are valid. Clipper `HI_RANGE` proves only the borrowed closed-path portion.
Future scanlines subtract/add spacing and their outside midpoint adds two
intersection points before division; proofs for both generated operations are
deferred to the construction milestone. Near-boundary O52 oracles stay inside
defined C++ behavior. O52 adds no validation.
Rust introduces no repair, epsilon beyond source `EPSILON`, deduplication,
stable tie, or malformed contour fallback.

## Verification

Eight focused tests freeze actual pinned-C++ traversal/intersection/outside literals, the accepted fixed-MSVC equal-key permutation, and exact X-then-Y sort-key bits above 2^53. Reversible mutations killed left/right traversal reversal, equal-key sort removal, shared-vertex max ownership, X/Y parity fallback, and cast-before-subtraction sort arithmetic, then restored production byte-exact. Final gates pass focused 8/8, dependency 630/630, workspace 6,290/6,290, warning-denying Clippy, core/browser wasm32, rustfmt, diff, LOC, and static audits. Temporary oracle and mutation artifacts were removed and the pinned Orca checkout remains clean.

## Consequences

This milestone is a complete dependency for anchor scanline section extraction,
not `construct_anchored_polygon` itself. Rotation, vertical-line generation,
section extension/merge/sort, traced polygon reconstruction, safety union,
O49/O51 composition, candidate commit, lifecycle activation, G-code, and CLI
parity remain deferred. All production/test files stay below 400 LOC and use
ordinary modules without include macros.
