# Task 22O.66 architecture decision record

## Status

Accepted and implemented; final independent implementation review pending.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3341-3343`, as one private,
lifecycle-neutral operation that derives a region's near-perimeter ring and
clips O65 upper-layer ensuring areas to it. The Rust destination is ordinary
module
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/region_bridge_ensuring_areas.rs`
with ordinary test children.

This is the smallest coherent continuation after O65: it consumes O65's
`additional_ensuring_areas` and the selected region's already-resolved normal
solid-infill Flow, but stops before rebuilding `stInternal` at source line 3345.

## Source boundary and direct dependencies

Included source behavior is exactly:

1. `PrintObject.cpp:3341`: flatten every `region->fill_surfaces.surfaces`
   `ExPolygon` in region-surface order and contour-before-holes order, run
   `union_safety_offset_ex`, then flatten its `ExPolygon` output in
   contour-before-holes order.
2. `PrintObject.cpp:3342`: shrink that complete flat polygon set once by the
   region's truncating integer `flow(frSolidInfill).scaled_spacing()`, then run
   one default NonZero/no-safety flat difference with the pre-shrink polygons
   as subject and shrunk polygons as clip.
3. `PrintObject.cpp:3343`: run one default NonZero/no-safety Polygon/Polygon
   intersection producing `ExPolygons`, with O65
   `additional_ensuring_areas` as subject and `near_perimeters` as clip.

Direct upstream dependencies are `Surface.hpp:119-157` and
`ExPolygon.hpp:300-363` (`to_polygons` ordering and lvalue/rvalue ownership),
`LayerRegion.cpp:21-28`, `PrintRegion.cpp:8-53`, `Flow.cpp:129-145,200-205`,
`Flow.hpp:62-69`, `libslic3r.h:38-43,60-94`, and
`ClipperUtils.hpp:17-27,331-383,509-520` plus
`ClipperUtils.cpp:264-410,642-679,738-739,788-810`. Rust dependencies are Task 22N's
already-resolved `Flow`, O53 `scaled_flow_value`, O65's flat ensuring polygons,
`RegionSurface`, `union_safety_offset_ex`, `offset_paths`,
`difference_polygons_paths`, and a new direct
`intersection_polygons_polygons_ex(subject: &[Polygon], clip: &[Polygon])`
overload in `geometry/clipper/boolean_ex.rs`, re-exported through
`geometry/clipper.rs` and `geometry.rs`. Its direct dependency tests freeze the
source overload's subject/clip insertion order, NonZero fill, topology, empty
behavior, and range errors.

## Exact private seam

```rust
pub(in crate::project_slice) struct RegionBridgeEnsuringAreas {
    pub(in crate::project_slice) near_perimeters: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring: Vec<ExPolygon>,
}

pub(in crate::project_slice) fn prepare_region_bridge_ensuring_areas(
    fill_surfaces: &[RegionSurface],
    additional_ensuring_areas: &[Polygon],
    solid_infill_flow: Flow,
    scale: CoordinateScale,
) -> Result<RegionBridgeEnsuringAreas, ClipperError>;
```

The operation returns `near_perimeters` because the pinned function retains it
through this block for debug consumption and because keeping the direct
intermediate makes the future region-rewrite composer explicit. It returns only
owned geometry and borrows every input.

## Required semantics

- Flatten every input surface, regardless of kind, in surface order; append each
  contour before its holes. Do not union per surface or filter by type.
- Run exactly one safety union over the complete flat list. Flatten its output
  contour-before-holes without sorting or reorientation.
- Compute scaled spacing with O53's exact `f64::from(f32) / scale.factor()` then
  truncating `i64` cast; cast that integer to `f32` and call one negative
  Miter/3 offset over the complete safety-union output.
- Run exactly one flat original-minus-shrunk default difference, preserving its
  engine output as `near_perimeters`.
- Run exactly one Polygon/Polygon-to-ExPolygon default intersection with O65
  ensuring polygons as subject and `near_perimeters` as clip. Preserve engine
  contour/hole/output order.
- Operation/error order is safety union, shrink, difference, intersection.
  Return the first `ClipperError`; no partial result escapes. Inputs and their
  allocations remain unchanged.
- Empty inputs still traverse the same operation sequence; add no early gate,
  validation, fallback, safety intersection, batching, sorting, deduplication,
  kind filter, option lookup, or surface mutation.

Production trusts the caller-provided current-layer region `fill_surfaces` at
this transaction point, O65 ensuring
polygons from the adjacent layer transaction, exact Task 22N normal
solid-infill Flow projected by the future composer, and retained object scale.
Spacing is finite positive; the scaled quotient is `i64`-representable; the
truncated integer and `f32` delta are strictly positive; all polygons are
normalized and Clipper-safe. O66 performs no Flow resolution or validation.

## Included and deferred behavior

Included only: pinned `3341-3343` geometry and the direct dependencies above.
The enclosing region loop at `3338`, `new_surfaces` allocation at `3339`, and
blank line `3340` are composer context, not behavior owned by O66.

Deferred: O65 map/layer traversal and source-to-record projection; internal
infill subtraction at `3345-3350`; bridge retagging at `3352-3367`; solid
recomposition at `3368-3374`; region replacement at `3385-3386`; second bridge
pass at `3391+`; O46-O65 composer and prepared successor/lifecycle; extrusion,
motion, G-code, CLI, and full golden parity.

## Architecture and verification constraints

The seam remains `pub(in crate::project_slice)`, filesystem-free,
platform-neutral, and production-unwired. Every Rust source is at most 399 LOC
and uses ordinary modules; `include!` and `include_bytes!` are forbidden for
source splitting.

Behavioral RED must freeze all-kind/source-order flattening, holes, one global
safety union, exact Flow/scale/cast/sign/join/miter, whole-set shrink and flat
difference, exact intersection operand roles and ExPolygon topology, empty
operation traversal, first-error precedence, repeatability, and complete input
nonmutation. Reversible compiling mutations must kill type filtering,
contour/hole or surface reordering, per-surface/repeated/skipped safety union,
wrong Flow/scale/cast/sign/join/miter, per-polygon/repeated/skipped/reversed or
safety difference, skipped/reversed/safety intersection, early empty return,
ignored errors, and output sorting; production must restore byte-exact.

Implementation adds the ordinary private seam and test children, the direct
flat Polygon/Polygon intersection overload and re-exports, exact ordered
geometry composition, injected operation-order/error tests, real topology,
empty/range/repeatability, and allocation-preservation tests. Behavioral RED was preserved against the deliberate `todo!` stub in
`/tmp/task22o66-behavioral-red.log`.

Focused O66 and the exact dependency band pass 12/12 and 776/776 in
`/tmp/pi-unified-exec-967-58fbf8ad.log`; workspace Nextest passes
6,436/6,436 with two skipped in `/tmp/pi-unified-exec-968-c6df80da.log`.
Strict Clippy, rustfmt, wasm32, x86_64/aarch64 Windows/macOS,
diff/LOC/static, pinned Orca, and no-staged checks pass in
`/tmp/pi-unified-exec-969-e230f5e9.log`.

The compile-validating 30-mutation audit script has SHA-256
`86cc8dc6e84b8864fa1a866412a69f92a342a8c2d36a1d9a22c862f88f14c749`;
its output has SHA-256
`577d9aef4ab2f8c26676f3cbb99e09383a75df3988f496783c6b9425748a5d1f`.
All 30 compiling behavioral mutations, including no-safety difference and
intersection discriminators, are killed and production restores
byte-exact at SHA-256
`ea5648423925ad89e5085409a0c3551c6315f5ac5e36a1a5a908e58cdae9a009`.
Final independent six-axis review remains required.
