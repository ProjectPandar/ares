# Task 22O.47 — deep sparse bridge area

## Status

Implemented. Focused source-shaped tests and the 18-layer real-KSR regression
pass with 115 flat Polygons, 5,641 points, 91,464 serialized bytes, and ordered
SHA-256 `f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a`.
Final verification passes 9/9 focused, 590/590 dependency-band, and
6,250/6,250 workspace tests, plus rustfmt, warning-denying workspace Clippy,
wasm32, diff, LOC, and structural audits. The first independent review rejected
the hierarchical-result mismatch and weak arithmetic/closing discriminators;
the main thread repaired them. Two read-only re-review rounds verified the
flat result, exact f32 threshold, both-scale epsilon thresholds, and corrected
documentation; the final six-axis verdict is unconditional approval.

## Goal and upstream boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s
`PrintObject::bridge_over_infill()` helper `gather_areas_w_depth` from
`OrcaSlicer/src/libslic3r/PrintObject.cpp:2819-2846`.

Supporting source boundaries are:

- `Surface.hpp:15-33` for `stInternal`, `stInternalVoid`, and the other surface
  kinds;
- `Layer.hpp:33-60` for layer `print_z` and region-owned fill surfaces;
- `ClipperUtils.hpp:404-409,548-553` and the corresponding implementations for
  union, closing, and difference;
- `libslic3r.h:43-70,93-94` for `EPSILON`, `SCALED_EPSILON`, and coordinate
  scaling; and
- the completed O42/O43 retained surface graph.

The Rust destination is
`project_slice::prepare_infill::bridge_over_infill::deep_sparse_area`:

```rust
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct DeepSparseLayer<'a> {
    pub(in crate::project_slice) planned: &'a PlannedLayer,
    pub(in crate::project_slice) fill_surfaces: &'a [RegionSurface],
    pub(in crate::project_slice) sparse_infill_density_percent: f64,
}

pub(in crate::project_slice) fn gather_deep_sparse_infill_area(
    layers: &[DeepSparseLayer<'_>],
    candidate_layer_index: usize,
    target_flow_height: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError>;
```

The operation remains unwired and does not create a prepared successor.

## Trusted contract

- Layer views are aligned with planned layer indices and the candidate index is
  nonzero and in range.
- `target_flow_height` is finite and positive and already includes the caller's
  `target_flow_height_factor` multiplication at `PrintObject.cpp:3155-3157`.
- Planned `print_z`, retained post-O42 surfaces, and effective per-region sparse
  density come from the same resolved 3MF object graph.
- The current KSR graph has one region. The per-layer view nevertheless carries
  its own density; no object-wide or candidate-layer density shortcut is
  permitted.
- Coordinates were validated by preceding project boundaries.

## Required behavior

1. Compute `bottom_z` with source f32/f64 evaluation order:
   `candidate.print_z - f64::from(target_flow_height * 0.9_f32) - 1.0e-4`.
2. Traverse indices in descending order starting at `candidate_layer_index - 1`.
3. Include the immediately lower layer unconditionally. Before processing each
   deeper layer, stop when its `print_z < bottom_z`.
4. Never inspect the candidate layer itself or any upper layer.
5. For each reached layer and region surface in stored order, classify as sparse
   exactly when the kind is `Internal` and that layer's density is below 100,
   or when the kind is `InternalVoid` regardless of density. Classify every
   other reached kind as non-sparse.
6. Preserve each ExPolygon's contour/hole topology and sibling insertion order.
7. Independently union sparse and non-sparse ExPolygons and close each union by
   exact scaled epsilon. Flatten contour before holes, then return the source
   one-pass flat-path `sparse - non_sparse` result without a PolyTree rebuild.
8. Empty sparse input and fully subtracted sparse input return `Ok(empty)`.
9. Return the first reachable `ClipperError`; do not catch, continue, publish a
   prefix, or mutate input.
10. Do not sort or canonicalize output.

## Deferred behavior

- multi-region aggregation with density carried per source region rather than
  the current trusted single-region layer view;
- thick `LayerRegion::bridging_flow(frSolidInfill, true)` construction;
- candidate-layer clustering and inflated AABB overlap;
- removal of already committed lower bridge areas;
- current-layer expansion, fill, top, and Lightning areas;
- the O46 lower-layer map and line-3203 anchor intersection;
- bridge direction and anchored polygon reconstruction;
- final surface commit and extra bridge layers;
- extrusion, fill/toolpath generation, motion, G-code, CLI activation, and
  final golden equality.

The deferred lower-layer map must remain transaction-local and may not become a
prepared-project stage.

## Acceptance

Follow TDD and keep tests in separate modules.

Focused tests must discriminate:

- unconditional inclusion of the immediate lower layer;
- threshold inclusion and first-below-threshold stopping for deeper layers;
- exclusion of the candidate layer;
- per-layer density, `InternalVoid`, and non-sparse classification;
- independent union/closing before difference, including holes and overlapping
  siblings;
- exact `0.9_f32` and epsilon arithmetic;
- empty and fully subtracted success;
- coordinate-range error atomicity and complete borrowed-input nonmutation.

The real-KSR regression must exercise every O43 candidate layer, prove
repeatability and complete input preservation, and freeze ordered per-layer
geometry literals produced by the Ares operation. Runtime tests may read only
the committed 3MF through `KsrArchive`; they must not read Orca source,
temporary oracle artifacts, or reference G-code, and must not invoke Orca.
Source parity is reviewed against the cited operation; final end-to-end parity
remains owned by the active KSR G-code golden.

Every changed Rust source file must remain below 400 physical lines. Rust source
must not use `include!` or `include_bytes!` for splitting. Final gates are
focused and dependency Nextest, workspace Nextest, rustfmt, warning-denying
workspace Clippy, wasm32 check, diff/LOC/static audits, and an independent
read-only six-axis review. Findings are fixed in the main thread and re-reviewed
until approval.
