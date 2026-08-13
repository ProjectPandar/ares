# Task 22O.44 — Connect infill to its boundary

## Status

Implemented and approved locally. The source-owned dependency remains
crate-private and intentionally unwired; public slicing still disposes O43 and
returns `ProjectSlicingIncomplete`. Final source/specification and standards
reviews approve the repaired implementation with no remaining finding.

## Goal and upstream boundary

Port the reusable boundary-connection helper from OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/FillBase.hpp:48-52,56,58-61,100`, for the relevant
  `FillParams` fields and caller-dispatch `dont_connect` predicate;
- `FillBase.hpp:219-224`, for the public `Fill::connect_infill` and
  `chain_or_connect_infill` declarations;
- `FillBase.cpp:323-398,420-842,995-1241,1243-1252,1263-1269,1432-1566,
  1580-1588,1594-1614,1690-1818`, for the active ExPolygon connector; and
- `FillBase.cpp:1820-1829`, only to prove that the KSR CrossHatch path
  dispatches into that connector.

The Rust destination is a new crate-private, source-owned `fill::connect`
module. It consumes ordered scaled-coordinate polylines, borrows an ExPolygon
boundary, and returns owned polylines or the first checked `ClipperError`. Its
direct input contract is unscaled `f64` spacing, `f32` `anchor_length` and
`anchor_length_max`, source-width integer `multiline`, `bool dont_sort`, and an
explicit `CoordinateScale` replacing Orca's process-global `SCALING_FACTOR`.
It does not create a `PreparedPost...` lifecycle type and is not wired through
the legacy `infills.rs` scaffold in this slice.

```rust
pub(crate) struct FillConnectionParams {
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) multiline: i32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn connect_infill(
    infill_ordered: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError>;
```

This dependency-first cut is intentional. The immediately preceding Orca
Lightning transaction at `PrintObject.cpp:2593-2723` is skipped by the KSR
fixture because O43 proves `has_lightning_infill == false`. The adaptive call
at `PrintObject.cpp:2728-2735` also returns null octrees for CrossHatch through
`PrintObject.cpp:972-979` and `FillAdaptive.cpp:276-358`. The next retained
caller result at `PrintObject.cpp:2725-2761` additionally requires complete
`group_fills`, `FillCrossHatch`, and this connector. A private CrossHatch
lattice checkpoint would expose an upstream-local intermediate and is not an
acceptable substitute.

KSR dispatch evidence is `FillCrossHatch.cpp:178-232`, especially its
`chain_or_connect_infill` call at line 225, followed by
`FillBase.cpp:1820-1829`.

## Direct dependencies

- `EdgeGrid.hpp:15-64,91-98,108-111,139-149,159-162,172-323,344-357`
  and `EdgeGrid.cpp:40-75,142-334,1047-1176` for stable contour/segment
  cells, line visitation, and closest endpoint association;
- `Line.hpp:43-76,251-266` for f64 `Linef` point-to-segment distance;
- `Geometry.hpp:208-258` and `Geometry/Circle.hpp:153-172` for
  Liang-Barsky and ray/circle collision intervals;
- `Point.hpp:117-123,187-203,420-424`, `BoundingBox.hpp`,
  `BoundingBox.cpp:203-211`, `Polygon.cpp:422-425`, `Polyline.hpp`, and
  `libslic3r.h:38-46,48-61,70,92-96,230-250` for perpendicular vectors,
  distinct rounding and truncating interpolation paths, path mutation,
  bounding boxes, scaling, `EPSILON`, `SCALED_EPSILON`, and predicate lower
  bounds.
- `PrintConfig.cpp:2995-3002` only for the trusted caller guarantee that
  `multiline` is in `1..=10`.

Existing Ares fixed-coordinate `Point`, `Polygon`, `ExPolygon`, `Polyline`,
`BoundingBox`, and `EdgeGrid` types are the destination primitives. Any narrow
EdgeGrid operation added for this slice is limited to a closest-point result
and signed-distance query plus ordered line-cell visitation. It must preserve
the existing checked coordinate/raster failure contract; O44 propagates that
first error before publishing output. A missing closest hit is an unconnected
endpoint, not an error. Trusted fill-graph invariants remain internal
assertions, not new validation or fallback branches.

```rust
fn closest_point_signed_distance(
    &self,
    point: Point,
    search_radius: Coord,
) -> Result<Option<ClosestPointResult>, ClipperError>;

fn visit_cells_intersecting_line(
    &self,
    p1: Point,
    p2: Point,
    visitor: impl FnMut(usize, usize, &[GridEdge]) -> bool,
) -> Result<(), ClipperError>;
```

`ClosestPointResult` retains contour and segment indices, signed `f64`
distance, and normalized `f64 t`. The inclusive closest-query cell rectangle
is not replaced by Ares' existing half-open box visitor, and no SDF is added.

The two active upstream `std::sort` calls compare only their source keys and
add no tie-break. ARD-0024 already fixes this repository's hidden C++ ordering
dependency to MSVC STL 14.44 through `fixed_msvc_sort_by`. O44 widens that
audited helper only to crate-private visibility and reuses its exact control
flow and comparators; it must not use a host Rust sort or add a tie-break.

## Required behavior

The crate-private seam trusts the source preconditions: nonempty ordered input,
at least two points per path, a nonempty contour, finite positive spacing,
`anchor_length >= 0`, `anchor_length_max >= 0.01f`,
`anchor_length_max >= anchor_length`, and caller-bounded `multiline >= 1`.
Preserve the connector's explicit source assertions; do not turn these trusted
conditions, including the config-owned multiline bound, into validation
branches. For such an input, Ares must:

1. keep the borrowed contour and holes unchanged, copy them in contour-then-
   hole order, and consume only its owned working polylines;
2. build the endpoint grid from the outer-contour bbox inflated by rounded
   `EPSILON / factor`, using resolution `Coord(10.0 / factor)`; query with
   radius `Coord(EPSILON / factor)`, with both `Coord` conversions truncating
   the completed expression;
3. associate each first and last point, in input order and front before back,
   with the closest boundary segment strictly closer than `SCALED_EPSILON`,
   preserving equal-distance first-win behavior; sort hits by `(contour index,
   segment index, t)`, but insert the original infill endpoint rather than the
   projected foot into copied contours, without consecutive duplicates; then
   parameterize each split contour by cast-before-subtract `f64` Euclidean
   length; closest replacement is strict, so an edge exactly at the initial
   search radius is not selected; preserve the source's signed
   `distance <= 3.0` debug assertion for every recorded hit;
4. preserve EdgeGrid's integer-space vertex ownership: calculate dot numerator
   `t_pt`, cross numerator `d_seg`, and squared segment length `l2_seg` before
   division; for `t_pt < 0`, accept the segment start only when the previous-
   edge dot is positive and derive sign from the corner determinant; for
   `0 <= t_pt <= l2_seg`, including exact segment-end equality, retain the
   current segment, compute signed distance as
   `d_seg / sqrt(f64(l2_seg))`, and only then normalize `t_pt / l2_seg`; for
   `t_pt > l2_seg`, skip that candidate so the successor segment's wedge rule
   may own the shared endpoint;
5. link each contour's T-junctions circularly by stable indices and retain
   unconnected endpoints as unconnected rather than fabricating a boundary;
6. build the touching grid from the original outer-contour bbox inflated by
   rounded `distance_colliding * 1.43`, with resolution
   `Coord(max(clip_distance, distance_colliding) + 10.0 / factor)`; both grids'
   creation merges every supplied contour point into the preset bbox and then
   adds the source's fixed 16 raw coordinate units on every bbox side; the
   touching grid therefore includes split original endpoints that protrude
   beyond the original contour bbox;
7. mark boundary intervals already occupied by the interiors of infill lines
   in source traversal order: trim `1.7 * (spacing / factor)` from each end,
   use collision radius `0.8 * (spacing / factor)`, visit the negative-
   perpendicular then positive-perpendicular raster trace, and apply the
   rounded-thick-segment interval updates with their strict comparisons,
   including the source's short/degenerate thickened infill-segment branch at
   length less than or equal to `SCALED_EPSILON`, which passes `offset` rather
   than `offset * offset` to its squared-radius helper; for the collision
   prefilter, merge the clipped infill subsegment endpoints, inflate that f64
   bbox by `distance_colliding + SCALED_EPSILON`, and use inclusive overlap
   against the uninflated boundary-segment f64 bbox;
8. preserve Orca's distinct conversion paths: anchors, half-width, clipping,
   and collision distances remain fractional `f64` scaled coordinates; grid
   resolutions and search radii truncate the completed expression to `Coord`;
   integer EdgeGrid bbox inflation rounds the delta first (halves away from
   zero) and then checked-subtracts/adds it from/to min/max, rather than rounding
   final endpoints; the collision visitor's `BoundingBoxf` prefilter stays
   fractional; and thick-line envelope points and limited-hook interpolation
   truncate toward zero, including for negatives;
9. when `dont_sort` is false, collect still-connectable next arcs and order
   them by ascending arc length through the fixed MSVC-sort control flow. The
   endpoint-hit sort uses the same fixed control flow with its lexicographic
   source comparator. Comparator-equivalent records receive no tie-break;
10. in that sorted-arc pass only, when `multiline > 1`, skip an arc whose length
   is strictly less than `(spacing / factor) * multiline`; then merge distinct
   roots only for an arc strictly shorter than scaled `anchor_length_max`, or
   take limited hooks for equality and greater lengths;
11. merge by reversing endpoints and taking the counter-clockwise boundary path
   in source order while keeping the lower working-vector index as the root;
12. only when scaled `anchor_length > SCALED_EPSILON`, take limited clockwise/
   counter-clockwise hooks of at most that length, subtracting half the nominal
   line width and all previously consumed or trimmed intervals without overlap;
13. process remaining endpoints in working-vector order independently of
    `multiline`: try the shorter then longer complete arc and accept lengths
    less than or equal to scaled `anchor_length_max`. Equal previous/next
    lengths try the previous side; if no complete merge is possible, choose
    the longer limited-hook side, with equality choosing the next side;
14. append nonempty roots in original working-vector order; and
15. propagate the first checked EdgeGrid coordinate/raster error with no
    partial result, fallback, sorting, path canonicalization, or mutation of
    the borrowed boundary.

Point order, path order, contour winding, hole order, strict threshold
comparisons, `f64` arithmetic, float-to-integer conversion order, and the fixed
MSVC comparator-equivalence permutations are compatibility behavior.
Geometrically equivalent reordered output fails this slice.

The checked work order is projection-grid construction, each path's front then
back closest query, touching-grid construction, then each clipped path segment's
negative-perpendicular and positive-perpendicular traces. The first checked
range/raster failure terminates that order and returns no output. O44 adds no
Clipper Boolean operation or new error variant; the direct error is the
existing `ClipperError::CoordinateOutOfRange`.

## KSR path and deferred behavior

The KSR CrossHatch branch supplies nominal sparse-flow spacing promoted from
f32 bits `0x3ed06cbe` to `0.40707963705062866` mm, a 400% f32 anchor with bits
`0x3fd06cbe` (`1.6283185482025146` mm), f32 anchor maximum 20 mm, multiline 1,
default-initialized `dont_sort=false`, and Normal scale. Therefore
`FillParams::dont_connect()` is false and
`FillBase.cpp:1828` calls this connector. The current Ares compatibility
scaffold instead extends two-point scanlines using a density pitch and cannot
substitute for this source helper.

Deferred from O44 are:

- `FillBase.cpp:401-417`, used only by the compile-disabled connector block;
- compile-disabled helper/visitor/debug blocks at `821-839`, `1175-1193`,
  `1197-1208`, and `1545-1551`;
- debug/assertion-only `FillBase.cpp:844-993` and all SVG/logging code;
- the alternate `Polygons` wrapper at `1254-1261`;
- support-connector-only `BoundaryInfillGraph` methods at `1271-1346`;
- support-only `mark_boundary_segments_overlapping_infill` at `1352-1430`;
- the compile-disabled connector branches at `1589-1592` and `1615-1688`;
- the entire `chain_or_connect_infill` wrapper and `dont_connect` predicate as
  caller behavior; O44 cites them only for reachability, and its no-anchor
  branch at `1823-1826` additionally depends on deferred `chain_polylines`;
- `connect_base_support` at `1832-2709` and multiline construction at
  `2712-2782`;
- complete `FillCrossHatch.cpp:28-232`, `Fill.cpp::group_fills`, the retained
  lower-layer anchor map at `PrintObject.cpp:2725-2761`, and every later
  bridge-depth, direction, commit, fill/toolpath/motion/G-code/CLI slice.

The old public Ares infill pipeline remains a temporary compatibility shell.
It is neither called by O44 nor treated as an implementation of the cited
`libslic3r::Fill` behavior. A later source-cited slice will compose the exact
CrossHatch filler with O44 before activating the anchor-map transaction. That
integration must retain both source anchor values; the legacy `InfillOptions`
accessor currently collapses them with `min(anchor, anchor_max)` and cannot
supply this connector's KSR tuple. Density 15% belongs to the deferred
CrossHatch lattice and is not an O44 connector input.

## Acceptance

The first compiling connector stub must produce a genuine exact-output RED.
Focused tests must then freeze outer-contour hooks, whole-arc merging, holes,
multi-vertex interior trimming, `dont_sort` crossed with multiline, sorted-pass
and remaining-pass anchor equality, equal-side quirks, both coordinate scales,
closest-edge first-win behavior, exact-search-radius rejection, output root
order, and boundary nonmutation.
They must also distinguish an endpoint slightly off the boundary from its
projected foot; convex, reflex, and shared-vertex ownership; checked integer-
product overflow from an incorrect early-f64 projection; f64 scaling from early
integer truncation; LargeBed grid resolution; negative fractional hook
truncation; negative-before-positive raster visitation; a positive half-unit
bbox inflation delta over both positive and negative-coordinate bounds;
fractional collision-prefilter bounds, checked min/max inflation overflow, the
nonzero-short-segment collision-radius quirk, and both early and late checked
EdgeGrid errors with no partial publication.
More than 32 comparator-equivalent records must exercise
the fixed MSVC sort beyond its insertion-sort threshold with no tie-break at
both the endpoint-hit and arc sort sites. Endpoint identities `0..=32` use key
`(7,11,0.5)`, identity 33 uses `(7,11,0.25)`, and identity 34 uses
`(7,11,0.75)`; arc identities use lengths `20.0`, `10.0`, and `30.0` in the
same groups. Each adapter must produce
`[33,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,0,34]`.

A disposable pinned-Orca harness must call public `Fill::connect_infill`
through the ExPolygon overload at `FillBase.cpp:1243-1252` and serialize exact
ordered point vectors for synthetic geometry with the KSR connector parameter
tuple. The harness record must prove every endpoint tuple and arc-length key in
those full-behavior vectors is comparator-distinct; Debug and `NDEBUG` outputs
must agree. Comparator-equivalent permutations use the independently
audited MSVC STL 14.44 oracle already frozen by ARD-0024, not the Linux host
STL. Rust tests must use frozen literals rather than reading upstream source,
a helper file, the committed golden G-code, or the filesystem. Reversible
mutations must distinguish the important ordering, strict-threshold,
scale/cast, and collision branches and restore production byte-for-byte. A
lifecycle regression must also prove O44 remains unwired and public slicing
still disposes O43 at the incomplete terminal.

Every Rust source file must stay below 400 physical lines. Final verification
requires focused Nextest, relevant geometry and O43 regressions, workspace
Nextest, rustfmt, workspace all-target/all-feature warning-denying Clippy,
ares-core/ares-wasm `wasm32-unknown-unknown` checks, diff/whitespace/LOC/include
audits, the unchanged normalized golden progress probe, and independent
standards and specification/upstream reviews.

## Completion evidence

The initial compiling empty-output stub failed the first exact nonempty Orca
hook vector. Final focused O44 coverage passes 41/41, the geometry/fixed-sort
dependency band passes 76/76, and the O24-O26/O40-O44 regression band passes
194/194. Workspace Nextest passes 6,201/6,201 with 27 slow and two skipped.
Workspace all-target/all-feature warning-denying Clippy, rustfmt, wasm32 core
and adapter checks, diff/whitespace, LOC, source-splitting, fixture-read, and
restored-source audits all pass.

The original comparator-distinct Orca Debug/Release harness output is frozen
by the matching plan. A second restored disposable harness freezes the
review-added remaining-pass, equal-side, `dont_sort`/multiline, and six-point
interior-trimming vectors in both Debug and Release. Reversible mutations make
both fixed-sort adapters, sorted and remaining threshold comparisons, both
equal-side decisions, continuous scaling, the short-segment collision quirk,
and negative-before-positive trace order observably RED; every production file
was restored to its recorded SHA-256 before the final gates.

The ignored normalized KSR golden remains the expected progress RED at the
unchanged pre-core CLI contract because `ares slice` still requires
`--options`. This is not an O44 failure: the connector is deliberately an
unwired dependency, so it changes no public G-code byte.
