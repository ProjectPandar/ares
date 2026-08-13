# Task 22O.57 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently reviewed before RED.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:3181-3205`, the complete current-layer bridge expansion-context block immediately after O56. The Rust destination is a private ordinary module, `prepare_infill/bridge_over_infill/current_layer_context.rs`:

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

The caller supplies O47/O56 deep area, regions in source layer order, O46 output for exactly `lidx - 1`, and the O48 front-candidate region Flow `scaled_spacing()`. Selecting those values remains a transaction-composer responsibility. The operation is borrowed-input/owned-output and remains unwired.

## Required semantics

Preserve source order and arithmetic:

1. Promote `scaled_spacing` to `f64`, multiply by `1.5_f64`, cast the result to `f32`, then expand the O56 deep area with default Miter join and miter limit 3.
2. Traverse regions in order. Flatten each selected `RegionSurface`/`ExPolygon` contour before holes. Append Top surfaces to `total_top_area`; Internal and InternalSolid surfaces to `expansion_area`; all `fill_expolygons` to `total_fill_area`; and Internal surfaces from Lightning regions to `lightning_area`.
3. Compute source `float(SCALED_EPSILON)` exactly as `(1.0e-4_f64 / scale.factor()) as f32`, with no `coord_t`/integer conversion, rounding, saturation, or `checked_scale`. Close `total_fill_area` and `expansion_area` independently as `offset_paths(+delta)` then `offset_paths(-delta)` using Miter join and miter limit 3; do not use hierarchical `closing_ex` or canonicalize output.
4. Intersect the closed expansion area with expanded deep area through one `intersection_polygons_paths` call.
5. Cast promoted spacing to f32, shrink expansion area, then call `intersection_open_polylines` exactly once over all borrowed lower-layer infill lines and the shrunken polygons. Do not clip per line or reorder output.
6. Multiply promoted spacing by `4.5_f64`, cast to f32, and shrink expanded deep area to produce `internal_unsupported_area`.
7. Preserve engine-call error precedence: deep expansion; total-fill expand; total-fill shrink; expansion expand; expansion shrink; closed intersection; anchor shrink; open intersection; final deep shrink. Return the first error atomically and leave every borrowed allocation unchanged.

Direct closure is pinned `Flow.hpp:69::scaled_spacing`, `libslic3r.h:46,52,60-61,93,96`, filtering in `SurfaceCollection.cpp:45-59`, surface/ExPolygon contour-before-hole conversion at `Surface.hpp:126-155` and `ExPolygon.hpp:299-318`, declarations/defaults in `ClipperUtils.hpp:19,27,375-403,498,525`, and implementation in `ClipperUtils.cpp:207-222,267-414,593-597,671-673,702-703,838-845,926-927`. It inherits the accepted Task 22F closed Boolean/PolyTree kernel (`deps_src/clipper/clipper.hpp:75-81,88-100,121-123,137,141-223,225-535`; `clipper.cpp:67-72,78-161,167-426,429-1614,1630-3340`), Task 22G closed offset kernel (`clipper.hpp:138-139,144-167,538-575`; `clipper.cpp:63-65,73-106,128-134,150-161,1000-1036,3345-3777`), and Task 22O.6 open-path kernel (`clipper.cpp:756-949` plus every reached `IsOpen` execution/output branch through PolyTree construction/extraction, matching `clipper.hpp`, and `ClipperUtils.cpp:835-934`). No host sort, map grouping, fallback, validation, saturation, filesystem access, or platform branch is introduced.

The trusted domain is a strictly positive source `coord_t` spacing exactly representable as f64 and whose `1.5`/`4.5` products and f32 casts are finite and positive, source-valid closed polygons and open infill lines, valid region ordering, and a finite f32 epsilon quotient. Zero spacing is out of domain and is not mapped to `ClipperError`. Deliberate range tests cross only the Clipper boundary.

## Consequences

O57 supplies the reusable context consumed after the debug-only `PrintObject.cpp:3206-3210`; it does not expand individual candidates. The candidate block at 3211-3308 (loop at 3213, actual expansion at 3215), unsupported-area filtering, boundary construction, O49/O51/O53 angle/anchoring composition, lightning collision handling, postprocessing, candidate replacement, surface rewrite 3322+, prepared successor/lifecycle activation, extrusion, motion, G-code, and CLI parity remain deferred.

Register private `mod current_layer_context;` with ordinary test children. Every production/test source must contain at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting.

## Completion evidence

A removed Catch actual-source driver used pinned SurfaceCollection/ClipperUtils and the fixed-MSVC-order Debug archive SHA-256 `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`. Driver/object/binary/output SHA-256 values are `3419fb712f073a04e69014c1378c3e4bc791368b8dd2b40cd91dfed1b2c217e8`, `f129cd218fe54a1ee9ee7ee7550f34655f2bb6cf3b37cc3fb996d89d4d613a50`, `93c225413e0f733871c757e9b59a5dfff7a0f3bdbe12e26b0886fc310fc2f434`, and `b4fb32856243e33c784cf20da1806c6a8982fe46e65b412d95c460ce32d3f0a7`. Its deterministic literals freeze ordered deep/top/expansion/fill/lower-line/anchor/unsupported output, a role-sensitive subject/clip pair, both scale thresholds, and arithmetic hex values.

Final gates pass focused 15/15, O43-O57/Clipper/Flow dependency 698/698, workspace 6,358/6,358 with two existing skips, warning-denying Clippy, core/browser wasm32, both Windows and both macOS targets, rustfmt, diff, LOC, static, clean-Orca, and no-staged checks. Nineteen distinct behavioral mutations are killed, including subject/clip reversal and lower-line reversal through the same private operation-order seams used by production. The final order audit restores current production SHA-256 `80ec5b9315f79ea4daea43c0711006f4613c326cf28df4d873185879a3753f8e`.
