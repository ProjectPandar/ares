# Task 22O.63 — bridge postprocessing geometry

## Status

Implemented and unconditionally approved by independent six-axis implementation review.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3290-3298`, into private ordinary
module `prepare_infill/bridge_over_infill/candidate_bridge_postprocessing.rs`.
This slice applies fine-detail opening, closing, three bridge clips, and the
expansion-area subtraction to owned O62 state. It remains production-unwired.

## Exact operation contract

```rust
pub(in crate::project_slice) struct PostprocessedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
    pub(in crate::project_slice) expansion_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn postprocess_candidate_bridge(
    collision_resolved: CollisionResolvedCandidateBridge,
    expansion_area: Vec<Polygon>,
    limiting_area: &[Polygon],
    total_fill_area: &[Polygon],
    total_top_area: &[Polygon],
    bridging_flow: Flow,
    scale: CoordinateScale,
) -> Result<PostprocessedCandidateBridge, ClipperError>;
```

## Behavior

1. Derive truncating `coord_t(scale_(flow.spacing))` through the approved O53
   helper. Opening delta is `(scaled_spacing as f64 * 0.75_f64) as f32`;
   closing delta is `scaled_spacing as f32`.
2. Unconditionally run flat Miter/3 opening (shrink then expand) and flat
   Miter/3 closing (expand then shrink), each with equal positive stage deltas.
3. Run default NonZero/no-safety flat booleans exactly in this order and with
   these roles: bridge ∩ limiting; result ∩ total-fill; result − total-top;
   expansion − final bridge.
4. Replace the bridge at every step. The final expansion difference must use
   the final bridge. Preserve all Clipper output ordering without sorting.
5. Preserve O62 boundary allocation and angle bits. Consume bridge/expansion
   ownership. Errors return no partial owned result and preserve exact first
   failure order. Borrowed limiting/fill/top inputs remain unchanged.

Production trusts successful O48/O62 state: positive source-representable Flow
and scaled spacing, finite positive deltas/angle, and valid Clipper geometry.
Flat O62 bridge polygons are normalized/non-overlapping per
`ClipperUtils.hpp:413`, inherited from O53's final safety union at
`anchored_polygon.rs:39`; O63 performs no normalization, validation, or fallback.

## Included and deferred

Included: only `PrintObject.cpp:3290-3298` plus `Flow.hpp:62-69`,
`libslic3r.h:38-43,60-94`, and flat Clipper dependencies
`ClipperUtils.hpp:19-27,400-425,430-432,495-498` and
`ClipperUtils.cpp:264-403,593-632,671-679,702-703`. Rust dependencies are O53
scaling, O62 output, `geometry/clipper/offset/opening.rs:4-12`,
`offset/execute.rs:54-64,162-184`, and `boolean_paths.rs:18-30,59-66`. The
commented full-spacing opening is excluded; active `0.75` fine-detail behavior
is normative.

Deferred: debug drawing; candidate append `3304-3305`; layer swap/clear;
history-producing composer; successor/lifecycle; second bridge pass;
region-surface rewrite; extrusion, motion, G-code, CLI, and golden parity.

## Tests and acceptance

Behavioral RED precedes implementation. Tests must discriminate:

- Normal/LargeBed integer scaling, multiply-before-f32 order, and the active
  `0.75` radius against the commented full-spacing alternative;
- real flat opening and closing, Miter/3 join/limit, and empty geometry;
- exact morphology and four-boolean order, operand roles, and first errors;
- final rather than stale bridge in expansion subtraction;
- literal ordered output, repeatability, boundary/angle allocation/value
  retention, consumed ownership, and complete borrowed-input nonmutation;
- injected competing errors and natural Clipper range errors.

Reversible mutations must kill wrong arithmetic/factor/join/miter, swapped or
missing morphology, reordered/skipped/safety/reversed booleans, stale expansion
clip, ignored errors, allocation/angle/boundary corruption, and output sorting,
then restore production byte-exact.

Final acceptance requires focused O63, exact O43-O63/Clipper/Flow/tree/options
dependency and workspace Nextest, strict Clippy, rustfmt, wasm32,
x86_64/aarch64 Windows and macOS checks, diff/LOC/static, clean pinned Orca, no
staged files, and independent six-axis repair/re-review until unconditional
approval. Every Rust source is at most 399 LOC and uses ordinary modules; no
include macro may split source.
