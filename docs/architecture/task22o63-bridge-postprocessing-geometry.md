# Task 22O.63 architecture decision record

## Status

Accepted, implemented, and unconditionally approved by independent six-axis review.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3290-3298`, as one private,
lifecycle-neutral candidate bridge postprocessing operation. The Rust
destination is ordinary module
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_bridge_postprocessing.rs`
with ordinary test children.

The operation consumes owned O62 collision-resolved state and owned
`expansion_area`; borrows `limiting_area`, `total_fill_area`, and
`total_top_area`; receives exact O48 `Flow` and retained `CoordinateScale`; and
returns owned boundaries, postprocessed bridge polygons, unchanged angle, and
remaining expansion polygons. It does not activate or commit the bridge
transaction.

## Source boundary and direct dependencies

Included source behavior is exactly:

1. `PrintObject.cpp:3290-3292`: retain Orca's active `0.75` fine-detail opening
   radius, not the commented full-spacing alternative.
2. `PrintObject.cpp:3293`: close by one exact scaled spacing.
3. `PrintObject.cpp:3294-3296`: intersect bridge polygons with limiting area,
   then total-fill area, then subtract total-top area.
4. `PrintObject.cpp:3297`: subtract the final bridge polygons from
   `expansion_area`.

Direct dependencies are `Flow.hpp:62-69::scaled_spacing`,
`libslic3r.h:38-43,60-94` for `coord_t = int64_t` and `scale_`, and flat-Polygon
morphology/boolean behavior in `ClipperUtils.hpp:19-27,400-425,430-432,495-498`
and `ClipperUtils.cpp:264-403,593-632,671-679,702-703`. Rust dependencies are
existing O53 scaled-Flow arithmetic; O62 `CollisionResolvedCandidateBridge`;
`geometry/clipper/offset/opening.rs:4-12`;
`geometry/clipper/offset/execute.rs:54-64,162-184`; and
`geometry/clipper/boolean_paths.rs:18-30,59-66`.

## Exact private seam

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

## Required semantics

- Reuse O53's source-exact truncating `coord_t(scale_(flow.spacing))` helper.
  Preserve source promotions: opening delta is integer scaled spacing promoted
  to f64, multiplied by `0.75_f64`, then cast to f32; closing delta is the
  integer scaled spacing cast directly to f32.
- Opening is flat Miter/3 shrink then expand with the same positive delta.
  Closing is flat Miter/3 expand then shrink with the same positive delta.
  Execute both unconditionally, including empty geometry.
- Preserve exact operation and operand order: opening(bridge), closing(result),
  intersection(result, limiting), intersection(result, total-fill),
  difference(result, total-top), difference(expansion, final bridge). Use
  default NonZero/no-safety flat polygon booleans.
- Every intermediate bridge result replaces the prior result; the final
  expansion difference uses the final postprocessed bridge, never an earlier
  intermediate. Preserve returned Clipper path order without sorting.
- Preserve O62 boundary allocation and angle bits unchanged on success. Consume
  O62 polygons and expansion polygons. Errors return no partial/recoverable
  owned state; first-error precedence follows the exact operation order.
- Borrowed limiting/fill/top inputs and their allocations remain unchanged.
- Production inputs are trusted successful O48/O62 state: positive
  source-representable Flow spacing and scaled spacing, finite positive opening
  and closing f32 deltas, finite angle, and valid geometry within reached
  Clipper arithmetic domains. O62 bridge polygons are normalized/non-overlapping
  as required by `ClipperUtils.hpp:413`, inherited from O53's final safety union
  at `anchored_polygon.rs:39`; O63 adds no normalization. Boundary tests outside
  this domain may assert natural range errors; no validation or fallback is added.

## Included and deferred behavior

Included only: pinned lines `3290-3298` and direct reached dependency closure.
The commented full-spacing opening is deliberately not implemented.

Deferred: debug drawing; `expanded_surfaces.push_back` at `3304-3305`;
per-layer surface swap/clear; the history-producing candidate/cluster composer;
prepared successor/lifecycle activation; second bridge pass at `3317+`;
region-surface rewrite; extrusion, motion, G-code, CLI, and full golden parity.

## Architecture and verification constraints

The seam remains `pub(in crate::project_slice)`, filesystem-free,
platform-neutral, and production-unwired. No option lookup or Ares-owned
pipeline behavior is introduced. Every Rust source is at most 399 LOC and uses
ordinary modules; include macros are forbidden for source splitting.

Behavioral RED must freeze exact dual-scale cast order, literal fine-detail
opening effect, real opening/closing Miter/3 kernels, every boolean operand role
and operation order, empty inputs, final-bridge use in expansion subtraction,
path order, complete borrowed-input nonmutation, consumed-allocation behavior,
and injected/natural error precedence. Reversible mutations must kill the
commented full-spacing radius, f32-before-multiply, multiply-before-truncate,
wrong factor/join/miter, swapped/omitted/repeated morphology, skipped/reordered
booleans, safety booleans, reversed operands, stale bridge in expansion diff,
ignored errors, angle/boundary corruption, and output sorting.

Implementation evidence: behavioral RED is preserved in
`/tmp/pi-unified-exec-860-f869a7b3.log`; focused O63 passes 7/7 in
`/tmp/pi-unified-exec-869-2383c455.log`; the exact dependency band passes
2,378/2,378 in `/tmp/pi-unified-exec-870-79ef9908.log`; and the Linux workspace
passes 6,409/6,409 with two skipped in `/tmp/pi-unified-exec-871-c0d6ee0e.log`.
Strict Clippy, rustfmt, wasm32, and all four desktop cross-target checks pass in
`/tmp/pi-unified-exec-872-a8511bed.log`; diff/LOC/static, clean pinned Orca, and
no-staged checks pass in `/tmp/pi-unified-exec-874-d641501b.log`.

The primary 21-mutation audit has SHA-256
`43c9bb0cfb4e02fbb03d72fc56c7895e9aa87fc581aca1b20c5f915527eec7c1`;
the supplemental four-mutation safety/repeated/order audit has SHA-256
`2c59d4335e06418309840c212d03c88ad4d20ece2ef8a8a3b1076e01c535a62c`.
All 25 mutations are killed and production is restored byte-exact at SHA-256
`29f279cf391dbf2f09936164fd8184531303227d67c974e6a5a44fa6b7160b4b`.
Final independent read-only six-axis implementation review approved unconditionally with no repair item or residual risk.
