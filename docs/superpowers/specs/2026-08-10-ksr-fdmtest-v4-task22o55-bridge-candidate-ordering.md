# Task 22O.55 — bridge candidate ordering

## Status

Implemented and verified. Independent source/specification review approved the boundary before RED.

## Goal and boundary

Port pinned `PrintObject.cpp:3127-3153`, the complete candidate presort inside each O54 cluster layer. Source closure is `Polygon.cpp:422-448`, `MultiPoint.cpp:89-92`, `BoundingBox.hpp:21,27-35,95-108`, `BoundingBox.cpp:94-105`, pinned Eigen 5.0.1 `Core/Dot.h:21-25,64-68`, `Core/Redux.h:99-119,443-450,488-490`, `Core/functors/UnaryFunctors.h:93-100`, `Core/functors/BinaryFunctors.h:34-45` (archive SHA-256 `0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12` from `deps/Eigen/Eigen.cmake:6-8`), and ARD-0024's fixed MSVC STL 14.44 sort target.

Destination: ordinary private `prepare_infill/bridge_over_infill/candidate_ordering.rs` and ordinary test children.

## Interface

```rust
pub(in crate::project_slice) fn order_candidate_surfaces(
    candidates: Vec<CandidateSurface>,
) -> Vec<CandidateSurface>;
```

The operation consumes and reorders owned candidates without cloning payloads. Every `source`, `new_polygons`, and `bridge_angle` value remains bit-exact.

## Ordering

1. Compute task-local source-shaped `{ min, max, defined }` keys. Do not call generic Ares `BoundingBox::from_polygons`. Empty outer polygon vectors are zero/undefined and present polygons are trusted nonempty. A per-polygon box is defined iff both coordinate extents are positive; zero-width/zero-height boxes retain min/max but are undefined, while diagonal collinearity with positive X/Y extents is defined. Later defined boxes replace undefined state and later undefined boxes are ignored.
2. Fixed-MSVC-sort the whole vector by minimum X, then minimum Y only. Equal minima receive no invented tie breaker.
3. Stop when length is at most two.
4. Otherwise read the post-sort first candidate's maximum point as f64 origin.
5. Stable-sort only the remaining candidates by strict ascending f64 squared distance from origin to each bounding-box minimum. Preserve exact cast/subtract/square/X-plus-Y order. Equal distances keep the first-sort order; the first element stays fixed.

Implement the first sort over a Copy permutation accepted by the shared fixed sorter, then consume candidates through that permutation. Rust's stable `slice::sort_by` is permitted only for the second sort because stability plus the comparator's equivalence classes fully determines its observable result.

## Trusted domain and deferrals

Candidate geometry is source-valid and every present polygon is nonempty; i64 coordinates are Clipper-bounded and all f64 distances are finite. No boundary validation or error result is introduced because the upstream operation cannot fail within that domain. X-then-Y square/add order is a structural/static invariant; operand swapping is commutative here and is not a behavioral mutation claim.

Deferred: the rest of `PrintObject.cpp:3114+`, including TBB/time-limit/debug adapters, deep-area gathering, lower-layer subtraction, expansion/anchor assembly, O46-O53 composition, collision rerun, postprocessing, candidate commit, region-surface rewrite, prepared successor, public lifecycle, extrusion, motion, G-code, and CLI parity.

## Acceptance

Start with compiling RED. Build a removed temporary oracle from pinned candidate/BBox/Eigen dependencies and exact fixed-MSVC `std::sort` replay. Rust tests retain only literals and do not read or compile source artifacts.

Tests must discriminate:

- empty, singleton, and two-element minimum-X/minimum-Y order;
- empty, zero-width/zero-height, and diagonal-collinear bounding boxes plus later defined-box replacement/ignore semantics;
- length two skips the distance pass while length three enters it;
- post-first-sort front maximum owns the origin; only the tail is distance sorted;
- f64 cast before subtraction at high coordinates and squared-distance alternatives such as integer subtraction, `(dx + dy)^2`, reassociation, or `mul_add`;
- equal-distance stability and preservation of the non-stable first-sort order;
- at least 42 comparator-equivalent candidates freezing the fixed-MSVC permutation beyond the insertion threshold and through the ninther branch;
- complete candidate field preservation, outer polygon-vector and inner point-vector allocation identities, and equal output from independently identical owned inputs.

Run reversible mutations for generic Ares bbox reuse, extent-defined classification and replacement/ignore, host/nonstable sort substitution, min comparator/tie breaker, origin min/front ownership, tail range, integer-before-f64 subtraction, squared-distance alternatives, stable-to-unstable replacement, and candidate clone/reconstruction; restore byte-exact. Exact `> 2` versus output-equivalent `>= 2`, and X-then-Y operand order, are structural audits rather than impossible behavioral mutation claims.

All files stay below 400 LOC and use ordinary modules without source-splitting include macros. Final gates: focused O55, O43-O55/ordering/geometry dependency, workspace, rustfmt, warning-denying Clippy, wasm32, Windows/macOS checks, diff/LOC/static/clean-Orca/no-staged gates, and independent six-axis repair/re-review until unconditional approval.

## Verification record

The removed actual-dependency/fixed-MSVC-replay driver and output hashes are `3aa80f9d1ec85e3a79dfb741ac888d62f5b3fd6229baf32d18a6c8715f9db1e4` and `1b4339927e830e9440cf5594e09ec8c1f165d8ae4e41bdb1fbecedcc157342f9`. Rust literals match its six records exactly.

Focused 12/12, dependency 673/673, workspace 6,333/6,333, strict Clippy, wasm32, four Windows/macOS checks, formatting/static/repository gates pass. Mutations for extent definition/merge, minimum comparator, host first sort, origin, tail range, integer subtraction, sum-square, mul-add, unstable tail, and cloned payload all fail; structural threshold and operand-order mutations are rejected. Restoration SHA-256 is `144b254aa21982bc6e04b173127615f686fc9f8f0afd1fb54e16ea7dc0ff3bcf`.
