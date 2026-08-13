# Task 22O.57 — current-layer bridge expansion context

## Status

Implemented and verified. Independent source/specification review approved the boundary before RED.

## Goal and source boundary

Port pinned `PrintObject.cpp:3181-3205`, immediately after O56, as one private geometry operation. Included behavior is deep-area expansion, ordered current-layer Top/Internal/InternalSolid/all-fill/Lightning collection, exact scaled-epsilon closing, deep intersection, lower-layer infill-line anchor clipping, and internal-unsupported shrink. Lines 3206-3210 are debug-only; the candidate block at 3211-3308 (loop 3213, expansion 3215) is deferred.

Direct sources are pinned `Flow.hpp:69`, `libslic3r.h:46,52,60-61,93,96`, `SurfaceCollection.cpp:45-59`, `Surface.hpp:126-155`, `ExPolygon.hpp:299-318`, `ClipperUtils.hpp:19,27,375-403,498,525`, and `ClipperUtils.cpp:207-222,267-414,593-597,671-673,702-703,838-845,926-927`. Inherited accepted closures are Task 22F (`deps_src/clipper/clipper.hpp:75-81,88-100,121-123,137,141-223,225-535`; `clipper.cpp:67-72,78-161,167-426,429-1614,1630-3340`), Task 22G (`clipper.hpp:138-139,144-167,538-575`; `clipper.cpp:63-65,73-106,128-134,150-161,1000-1036,3345-3777`), and Task 22O.6 (`clipper.cpp:756-949`, every reached `IsOpen` execution/output branch through PolyTree construction/extraction, matching `clipper.hpp`, and `ClipperUtils.cpp:835-934`).

Destination: ordinary private `prepare_infill/bridge_over_infill/current_layer_context.rs` and ordinary test children.

## Interface

```rust
pub(in crate::project_slice) struct CurrentLayerBridgeRegion<'a> {
    pub fill_surfaces: &'a [RegionSurface],
    pub fill_expolygons: &'a [ExPolygon],
    pub sparse_infill_pattern: ProcessInfillPattern,
}

pub(in crate::project_slice) struct CurrentLayerBridgeContext {
    pub deep_infill_area: Vec<Polygon>,
    pub lightning_area: Vec<Polygon>,
    pub expansion_area: Vec<Polygon>,
    pub total_fill_area: Vec<Polygon>,
    pub total_top_area: Vec<Polygon>,
    pub anchors: Vec<Polyline>,
    pub internal_unsupported_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_current_layer_bridge_context(
    deep_infill_area: &[Polygon],
    regions: &[CurrentLayerBridgeRegion<'_>],
    lower_layer_infill_lines: &[Polyline],
    scaled_spacing: Coord,
    scale: CoordinateScale,
) -> Result<CurrentLayerBridgeContext, ClipperError>;
```

The future composer must pass the post-O56 area, source-ordered current-layer regions, O46 lines for exactly `lidx - 1`, and O48 front-candidate-region scaled spacing. O57 does not search or infer provenance.

## Behavior

1. Compute the deep expansion delta as `(scaled_spacing as f64 * 1.5_f64) as f32` in that order and expand with Miter/3.
2. Walk regions and their stored geometry in order. Flatten contours before holes. Gather Top, Internal plus InternalSolid, every fill ExPolygon, and Lightning-region Internal into separate flat lists exactly as source.
3. Derive `float(SCALED_EPSILON)` exactly as `(1.0e-4_f64 / scale.factor()) as f32`, with no `checked_scale`, integer conversion, rounding, or saturation. Close total fill and expansion independently as `offset_paths(+delta)` then `offset_paths(-delta)`, Miter/3; do not use `closing_ex` or canonicalize output.
4. Intersect expansion with expanded deep area through one `intersection_polygons_paths` call.
5. Compute `scaled_spacing as f64 as f32`, shrink expansion, and call `intersection_open_polylines` once over all lower lines and the shrunken polygons; do not call per line or reorder output.
6. Compute `(scaled_spacing as f64 * 4.5_f64) as f32` and shrink expanded deep area for unsupported-area classification.
7. Preserve first-error precedence across deep expansion; total-fill expand/shrink; expansion expand/shrink; closed intersection; anchor shrink; open intersection; final deep shrink. Inputs and allocations remain unchanged; outputs are owned and ordered.

## Trusted domain and deferrals

Spacing is strictly positive source `coord_t`, exactly representable as f64, with finite positive products/f32 casts. Zero is out of domain and is not mapped to `ClipperError`. Geometry is valid source closed/open input and the direct f64 scale quotient is finite as f32. No public validation is added. Range-error tests deliberately cross the external engine boundary.

Deferred: O47/O48/O46 provenance composition; debug-only 3206-3210; candidate block 3211-3308 (loop 3213, expansion 3215); unsupported filtering; boundary polylines; O49/O51/O53; collision rerun; opening/closing postprocessing; candidate commit; region surface rewrite; successor/lifecycle; extrusion, motion, G-code, CLI parity.

## Acceptance

Begin with compiling RED. Build a removed temporary actual-source driver linked to the audited fixed-MSVC-order Clipper archive where ordered Paths/Polylines can vary; freeze exact literal outputs and source arithmetic bits. Tests must discriminate:

- exact `1.5`, `1.0`, and `4.5` promotion/multiplication/cast order, including source-safe values that differ under early f32 conversion or reassociation;
- both Normal and LargeBed exact scaled epsilon closing thresholds, plus a structural/static ban on `checked_scale` or integer intermediates because the two supported scales cannot behaviorally distinguish that mutation;
- region, surface, ExPolygon, contour-before-hole, and output list order;
- kind membership: Top only, Internal plus InternalSolid, all fill ExPolygons, Lightning Internal only;
- closing before intersection, expansion/deep intersection, shrink before anchor intersection, and one call over all lower lines;
- empty regions/deep/lines and unconditional normalization behavior;
- ordered anchors, hole/component topology, first closed/open range error, complete input/allocation nonmutation, and repeatability.

Run reversible mutations for each multiplier/cast, kind selection, Lightning ownership, flatten order, omitted/reordered closing/intersection/shrink, scale hardcoding, line reorder, early return, and role reversal; restore byte-exact. Private registration, ordinary test children, at most 399 lines per file, and no `include!`, `include_bytes!`, or `include_str!` splitting are required.

Final gates: focused O57, O43-O57/Clipper/Flow dependency, workspace Nextest, rustfmt, strict Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged checks, and independent six-axis repair/re-review until unconditional approval.

## Verification record

The removed actual-source driver matches all Rust ordered literals, including lower-line source order and the role-sensitive closed intersection pair. Driver/object/binary/output SHA-256 values are `3419fb71...`, `f129cd21...`, `93c22541...`, and `b4fb3285...`; fixed archive SHA-256 is `b643964e...`.

Focused 15/15, dependency 698/698, workspace 6,358/6,358 with two existing skips, strict Clippy, wasm32, four Windows/macOS checks, and formatting/static/repository gates pass. Nineteen distinct behavioral mutations fail, including the two operation-order mutations through production's private internal seams. Final production restores SHA-256 `80ec5b9315f79ea4daea43c0711006f4613c326cf28df4d873185879a3753f8e`.
