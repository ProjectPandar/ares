# Task 22O.65 architecture decision record

## Status

Accepted and implemented; final independent implementation review pending.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3318-3319,3322-3336`, as one private,
lifecycle-neutral bridge rewrite-area collector. The Rust destination is
ordinary module
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/bridge_rewrite_areas.rs`
with ordinary test children.

This slice preserves the layer-presence gate, flattens current-layer committed
bridge polygons into `cut_from_infill`, and computes upper-layer one-spacing
perimeter rings into `additional_ensuring_areas`. It stops before per-region
surface rewriting.

## Source boundary and dependencies

Included source behavior is exactly:

1. `PrintObject.cpp:3318-3319`: skip a layer only when both current and next
   candidate-map keys are absent.
2. `PrintObject.cpp:3322-3327`: flatten current committed candidate polygons in
   candidate then polygon order.
3. `PrintObject.cpp:3329-3336`: for each upper committed candidate in order,
   compute `diff(new_polys, shrink(new_polys,
   region->flow(frSolidInfill).scaled_spacing()))` and append each result.

Direct dependencies are normal solid-infill Flow resolution at
`LayerRegion.cpp:21-28`, `PrintRegion.cpp:8-53`, and
`Flow.cpp:129-145,200-205`; `Flow.hpp:62-69`; `libslic3r.h:38-43,60-94`; flat
`ClipperUtils.hpp:19-27,331-383,430-432`; and
`ClipperUtils.cpp:264-408,671-679`. Rust dependencies are Task 22N
`project_slice/perimeters/flow.rs` and `perimeters/types.rs` as the existing
normal solid-infill Flow provider, O43/O64 `CandidateSurface`, O53 scaled-Flow
arithmetic, and Ares `offset_paths`/`difference_polygons_paths`. The future
composer projects candidate source/layer/region identity to the already-resolved
Task 22N record; O65 performs no Flow resolution.

## Exact private seam

```rust
pub(in crate::project_slice) struct UpperBridgeEnsuringInput<'a> {
    pub(in crate::project_slice) surface: &'a CandidateSurface,
    pub(in crate::project_slice) solid_infill_flow: Flow,
}

pub(in crate::project_slice) struct BridgeRewriteAreas {
    pub(in crate::project_slice) cut_from_infill: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring_areas: Vec<Polygon>,
}

pub(in crate::project_slice) fn collect_bridge_rewrite_areas(
    current: Option<&[CandidateSurface]>,
    upper: Option<&[UpperBridgeEnsuringInput<'_>]>,
    scale: CoordinateScale,
) -> Result<Option<BridgeRewriteAreas>, ClipperError>;
```

`Option` represents map-key presence exactly; `Some(&[])` is present and must
not be conflated with absence.

## Required semantics

- Return `Ok(None)` before Flow or geometry work only when both inputs are
  `None`. Every other presence combination returns `Some`, including present
  empty slices.
- Clone current `new_polygons` into `cut_from_infill` in candidate then polygon
  order, matching C++ vector insertion and producing independent ownership.
- For each upper candidate in input order, resolve truncating integer scaled
  solid-infill spacing through O53's helper, cast that integer to f32, shrink
  its complete polygon set once with negative Miter/3 offset, then run one
  default NonZero/no-safety difference with original polygons as subject and
  shrunk polygons as clip. Append each difference output in engine order.
- Process upper candidates sequentially. First offset/difference failure returns
  immediately and no partial owned result escapes. Task 22N Flow-resolution
  errors occur upstream. Borrowed candidates, polygons, and allocations remain
  unchanged.
- Add no union, safety offset, sorting, deduplication, validation, fallback,
  density/option inference, map traversal, or cross-layer lookup.

Production trusts same-object O64 committed current/upper histories, exact
per-upper normal solid-infill Flow already resolved by Task 22N from the
candidate's region at that upper layer, and retained object coordinate scale.
`Flow.spacing` is finite positive; its scaled f64 quotient is i64-representable;
the truncating scaled i64 and its f32 cast are strictly positive; upper
candidate polygons are candidate-local normalized/non-overlapping O63 output;
and all reached coordinates are Clipper-safe. The future composer owns
source-index/layer/region projection to the Task 22N record; O65 does not infer
or resolve Flow.

## Included and deferred behavior

Included only: pinned layer gate, current cut flattening, and upper ensuring
ring collection at `3318-3319,3322-3336` plus direct dependencies.

Deferred: parallel/range/map/layer traversal and timeout at `3315-3317`;
retrieving `Layer` at `3320`; Task 22N error handling and source-to-record
projection;
per-region near-perimeter computation and all surface rewrites at `3338-3387`;
second bridge pass at `3391+`; O46-O64 composer; successor/lifecycle; extrusion,
motion, G-code, CLI, and full golden parity.

## Architecture and verification constraints

The seam remains `pub(in crate::project_slice)`, filesystem-free,
platform-neutral, and production-unwired. Every Rust source is at most 399 LOC
and uses ordinary modules; include macros are forbidden for source splitting.

Behavioral RED must freeze absent versus present-empty gates, current clone and
flat order, exact per-upper Flow/scale/truncation, one shrink then one
difference per upper candidate, offset-before-difference and candidate error
order, empty/full erosion,
engine output append order, repeatability, and complete input nonmutation.
Reversible mutations must kill wrong gate/presence, stale/missing/reordered cut,
wrong Flow/scale/cast/sign/join/miter, one-shot or repeated shrink/difference,
safety/reversed/skipped difference, cross-candidate batching, ignored errors,
and output sorting, then restore production byte-exact.

The result owns `Vec<Polygon>` while every input is borrowed, so omitting the
current-polygon clone or aliasing borrowed storage cannot compile in safe Rust.
That ownership invariant is therefore a type/structural audit rather than a
behavioral mutation. Tests additionally prove distinct point allocations and
complete borrowed-allocation preservation; behavioral mutations cover wrong
source and clone/order semantics.

Implementation evidence: behavioral RED is preserved in
`/tmp/pi-unified-exec-887-f539e8c1.log`; focused O65 passes 9/9 in
`/tmp/pi-unified-exec-914-a0bffc02.log`; the exact dependency band passes
2,393/2,393 in `/tmp/pi-unified-exec-915-ccb2d87c.log`; and the Linux workspace
passes 6,424/6,424 with two skipped in `/tmp/pi-unified-exec-916-c8c3404d.log`.
Strict Clippy, rustfmt, wasm32, and all four desktop cross-target checks pass in
`/tmp/pi-unified-exec-917-4a11103e.log`; diff/LOC/static, clean pinned Orca, and
no-staged checks pass in `/tmp/pi-unified-exec-918-195e2056.log`.

The 24-mutation audit script has SHA-256
`17fac7291eeb3145b4d91ad4063a9e76a9ee08611c3a7b56924dd87fe1053580`;
it rejects compiler failures as invalid evidence before classifying behavioral
failures, and its output has SHA-256
`a7d3ba6054ee3d1e38c08db096b2aecff24387d51664a8f56158bd74a988c531`.
All 24 compiling behavioral mutations are killed and production is restored
byte-exact at SHA-256
`52701f7fca5b4bdfcb79624ab8cfa08cc808e39cc94095edeefbb0a19c6fe51e`.
Final independent six-axis review/fix/re-review remains required for approval.
