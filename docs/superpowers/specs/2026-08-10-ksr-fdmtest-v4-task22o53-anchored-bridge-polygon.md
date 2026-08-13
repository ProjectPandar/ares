# Task 22O.53 — anchored bridge polygon construction

## Status

Implemented and verified. The temporary fixed-MSVC-replayed pinned-C++ oracle, 19 mutation kills, focused 20/20, dependency 650/650, workspace 6,310/6,310, Clippy, WASM, Windows/macOS cross-checks, and static gates pass; final independent re-review remains.

## Goal and source boundary

Port the complete geometry-only `construct_anchored_polygon` lambda at pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:2939-3111`. Direct closure is `Flow.hpp:62,69`, `libslic3r.h:60-94`, `Polygon.hpp:162-168,196-217`, `Polygon.cpp:422-448`, `Line.cpp:135-139`, `Line.hpp:152-197`, `Point.hpp:104-114,187-242`, `MultiPoint.cpp:21-34`, `BoundingBox.hpp:16-155`, O52's indexed queries, and the flat-Paths offset chain in `ClipperUtils.hpp:17-32,46-148,339,362` plus `ClipperUtils.cpp:267-293,333-365,392-408`. The Rust destination is ordinary `bridge_over_infill/anchored_polygon.rs`, `anchored_polygon/sections.rs`, and `anchored_polygon/tracing.rs` modules plus ordinary test children.

## Interface

```rust
pub(in crate::project_slice) fn construct_anchored_polygon(
    bridged_area: &[Polygon],
    anchors: &[Line],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError>;
```

Inputs remain unchanged. Returned polygon/path/point order is observable and must match the pinned source.

## Scaling, rotation, and scanlines

1. Compute scaled spacing/width exactly as `coord_t(scale_(m_spacing/m_width))`: promote the stored f32 to f64, divide by the runtime coordinate factor, then truncate to i64. Do not round or route through a different Flow width.
2. Compute `aligning_angle = -bridging_angle + PI * 0.5` in source order. Rotate every polygon point using original x/y f64 values and `round(cos*x - sin*y)`, `round(cos*y + sin*x)`, ties away from zero. Rotate anchor endpoints identically.
3. Area extents own X and anchor extents own Y. Compute `n_vlines = (max_x - min_x + spacing - 1) / spacing`, then each `x = min_x + (i + 0.5) * spacing` with the source size_t-to-double expression and truncating assignment. Y endpoints are `anchor_min_y - spacing` and `anchor_max_y + spacing`.
4. Build two O52 trees from rotated anchor edges and rotated closed area edges. For every scanline, retain every adjacent pair of area hits whose integer `(left + right) / 2` midpoint classifies strictly inside.

## Anchor extension and section preparation

1. For a section's low endpoint, reproduce reverse `upper_bound` with predicate `point.y > hit.y`; if found, replace it by that hit, evaluate `f64(hit.y) - f64(scaled_width) * (0.5 + 0.5)`, and truncate the compound-assignment result to i64. For the high endpoint, reproduce forward `upper_bound` with predicate `point.y < hit.y`; if found, replace it, evaluate the corresponding f64 addition, and truncate. The mixed-width expression is normative and differs from integer arithmetic above 2^53.
2. Run exactly one forward neighbor pass. Inclusive interval overlap merges into the later section using source endpoint ownership and turns the earlier section into a zero-length line.
3. Remove zero-length sections. Apply ARD-0024's fixed-MSVC STL 14.44 sort with the exact source comparator: identical lines compare false, otherwise `left.a.y < right.b.y`. Do not substitute stable, host, geometric, index, or normalized endpoint sorting.

## Trace reconstruction and output

1. Visit section slices in scanline order. Each active trace owns low/high point vectors. Per-slice used-segment identity is exact; use indices rather than pointer addresses but preserve source order and uniqueness.
2. Reproduce both source `upper_bound` candidate bounds against the sorted slice: begin is the first segment with `seg.b.y > low.y`, so equality is excluded; end is the first segment with `seg.a.y > high.y`, so `seg.a.y == high.y` remains included. Take the first unused candidate in that half-open range.
3. For low and high independently, compute integer endpoint differences before f64 promotion. Strictly below `36.0 * double(spacing) * spacing` appends the endpoint directly. Otherwise append prior plus `(spacing/2,0)`, candidate minus `(spacing/2,0)`, then candidate. Integer division occurs before addition/subtraction.
4. If no candidate is added, append half spacing to both trace ends, emit lows followed by reversed highs, and remove the trace. Seed each unused segment with `{endpoint - half_spacing, endpoint}` in source order.
5. After the final slice, emit every remaining trace without an extra half-spacing close, exactly as source. Apply flat `union_safety_offset(Polygons)` with Clipper safety offset 10, Miter, limit 3, preserving flat output order, then rotate output by `-aligning_angle`.
6. Return the first Clipper coordinate error atomically and never mutate borrowed input.

## Trusted domain and deferred behavior

Inputs satisfy the internal source domain recorded in `docs/architecture/task22o53-anchored-bridge-polygon.md`: bridged-area polygons are nonempty valid closed contours satisfying O52 `outside`; anchors are a nonempty ordered set of arbitrary open or closed line edges and need not form contours; scaled Flow values are positive and representable; all signed/unsigned extent, count, midpoint, threshold, mixed-width, and generated-coordinate arithmetic is defined; and rotation/intersection results are finite and representable. No runtime repair or public validation is added.

Deferred: the two `construct_anchored_polygon` call sites, boundary/anchor assembly, automatic/override angle composition, collision rerun, opening/closing/limiting intersections/difference, candidate clustering and commit, lifecycle successor, extrusion, motion, G-code, and CLI parity.

## Acceptance

Start with compiling behavioral RED tests whose literals come from the actual pinned lambda dependencies and exact Eigen/Clipper environment. Normative full output must run with audited MSVC STL 14.44; if the available driver runs on another standard library, freeze every output-affecting pre-sort tuple vector separately and replay each sort through ARD-0024's fixed-MSVC control flow before accepting the oracle. This applies to section ordering and Clipper-internal Paths ordering. Rust tests must not read, compile, or execute Orca or oracle artifacts. Freeze:

- both coordinate scales and nonbinary f32 Flow scaling; positive/negative half-tie rotation and round-trip drift;
- exact vertical-line count/coordinates, area-X versus anchor-Y extent ownership, integer midpoint classification, and adjacent-pair rather than pair-step traversal;
- lower/upper anchor `upper_bound` equality ownership, greater-than-2^53 mixed-width extension, inclusive overlap merge, zero removal, source comparator pre-sort tuples, and fixed-MSVC output including greater-than-32 ties when reachable from valid full inputs (otherwise a structural helper oracle is labeled as such);
- tracing begin equality exclusion and end equality inclusion, direct versus three-point trace connection at strict threshold equality, integer-before-f64 distance subtraction, half-spacing integer division, unmatched close, unused seeding, split/merge traces, final unclosed emission, and complete ordered polygon points;
- flat Paths safety-union topology/order with overlapping, hole, and multiple-component fixtures that differs from raw per-path offset, repeatability, empty output after valid geometry, natural Clipper failure atomicity, and complete borrowed-input nonmutation. The existing PolyTree/ExPolygon contour-before-hole flattening is observationally identical at this `Vec<Polygon>` boundary and therefore is not a separate mutation discriminator.

Run reversible mutations for scanline center shift, midpoint integer order, each anchor bound direction, integer-versus-f64 width extension, inclusive overlap, section comparator/sort, both tracing bound predicates, cast-before-subtraction distance arithmetic, strict distance threshold, half-spacing branch, used-segment ownership, final trace handling, flat safety union, and rotation back. Restore byte-exact and remove all temporary artifacts.

All production/test files remain below 400 LOC and use ordinary modules without source-splitting include macros. Final gates: focused O53, O43-O53/geometry/Flow dependency and workspace Nextest, rustfmt, warning-denying Clippy, native Linux plus core/browser wasm32, Windows, and macOS compile/test CI, diff/LOC/static audits, clean pinned Orca, and independent six-axis implementation repair/re-review until unconditional approval.
