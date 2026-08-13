# Task 22O.62 — candidate collision reconstruction

## Status

Implemented, gate-verified, and unconditionally approved by final independent six-axis implementation review.

## Goal and source boundary

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:3274-3288`, into private ordinary module
`prepare_infill/bridge_over_infill/candidate_collision_reconstruction.rs`.
This slice expands the initial O61 bridge area, finds the first colliding prior
expanded surface in composer append order, adopts its angle, and conditionally reruns O53
once. It remains production-unwired.

## Operation contract

The private operation accepts:

- borrowed original `area_to_be_bridge`;
- owned O61 `CandidateAnchoredBridge`;
- exact O48 `Flow` and current angle;
- borrowed prior-completed `CandidateSurface`s in exact future-composer append
  order, whose `new_polygons` are postprocessed at source lines `3292-3297` and
  appended at `3304-3305`, never raw/pre-expansion O43 candidate geometry;
- retained `CoordinateScale`.

History production is a caller-provenance assumption and remains deferred. The
exact seam returns `CollisionResolvedCandidateBridge { boundary_polylines,
bridging_area, bridging_angle }`, where polygons are collision-resolved but
still pre-postprocessing, or the first `ClipperError`:

```rust
pub(in crate::project_slice) struct CollisionResolvedCandidateBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
    pub(in crate::project_slice) bridging_angle: f64,
}

pub(in crate::project_slice) fn reconstruct_candidate_bridge_collision(
    area_to_be_bridge: &[Polygon],
    initial: CandidateAnchoredBridge,
    bridging_flow: Flow,
    bridging_angle: f64,
    completed_surfaces: &[CandidateSurface],
    scale: CoordinateScale,
) -> Result<CollisionResolvedCandidateBridge, ClipperError>;
```

## Behavior

1. Derive `coord_t(scale_(flow.spacing))` with source truncation. Preserve
   integer-to-f64, `* 3.0_f64`, then f32 offset conversion order.
2. Expand the exact initial O61 bridge polygons once with flat Miter/3 offset,
   even when there are no completed surfaces.
3. Visit caller-provided completed surfaces in composer append order without
   sorting/reversing/grouping. For each, run default flat NonZero, no-safety
   intersection with `new_polygons` as subject and the expanded initial area as
   clip; use only emptiness and discard the intersection geometry.
4. On the first nonempty intersection, select that surface's `bridge_angle`,
   stop traversal, and rerun O53 exactly once with the original area, unchanged
   O61 boundary lines, exact Flow, selected angle, and scale.
5. Without collision, preserve the consumed initial polygon and boundary
   allocations unchanged plus input angle. With collision, replace only polygons
   and angle while retaining the exact boundary allocation. Never sort.
6. Preserve expansion → visited intersections → conditional O53 error order;
   errors consume the owned input and return no partial/recoverable result.
   Borrowed inputs remain unchanged.

Production trusts successful O61/O53 provenance: positive representable Flow
width/spacing; positive representable scaled spacing; finite positive f32
expansion delta; finite current/prior angles; valid original area, initial
polygons, boundary lines, and generated-coordinate arithmetic. Natural range
errors deliberately probe outside this production domain. No fallback or
validation is added. O53 scaling and O61 line conversion gain only narrow
`pub(super)` visibility for exact sibling reuse.

## Included and deferred

Included: only `PrintObject.cpp:3274-3288` plus directly reached Flow scaling,
flat offset/intersection, O43 surface, O53, and O61 line-conversion behavior.

Deferred: `3292-3298` postprocessing; expansion mutation; candidate/per-layer
commit; cluster composer; successor/lifecycle; second bridge pass; extrusion,
motion, G-code, CLI, and golden parity.

## Tests and acceptance

Behavioral RED precedes implementation. Focused tests must discriminate:

- exact no-collision polygon/boundary allocation retention and first-collision
  composer-order ownership, with boundary allocation retained on collision;
- break before later geometry/error and no sort/reverse/grouping; exact use of
  each supplied `completed_surfaces[i].new_polygons`; raw-versus-postprocessed
  provenance remains a static contract whose integration mutation is deferred
  to the future composer;
- exact Normal/LargeBed spacing truncation and f64/f32 promotion order;
- expansion even for empty completed history, receiving exact initial O61 area;
- intersection subject/clip operand roles, rejected reversed operands, and
  discarded intersection geometry;
- empty initial polygons and deterministic repeatability;
- actual production Miter/3 offset, default no-safety intersection, and O53
  reconstruction;
- exact Flow/angle/scale/original-area/boundary forwarding;
- injected expansion errors, natural intersection/O53 range errors, competing
  error precedence, atomicity, and complete input nonmutation.

A reversible mutation audit must kill skip/wrong expansion, pre-truncation
scaling, f32-before-multiply, wrong factor/join/miter, safety intersection,
reversed operands, reversed/sorted traversal, failure to use supplied
completed-surface polygons,
last-collision ownership, missing break, reuse of intersection geometry,
unconditional reconstruction, forwarding corruption, ignored errors,
allocation loss, and output sorting.
Production must restore byte-exact.

Final acceptance requires focused O62, exact O43-O62/Clipper/Flow/tree/options
dependency Nextest, workspace Nextest, strict Clippy, rustfmt, wasm32,
x86_64/aarch64 Windows and macOS compile checks, diff/LOC/static, clean pinned
Orca, no staged files, and independent six-axis review/fix/re-review until
unconditional approval. Every Rust source is at most 399 LOC and uses ordinary
modules; include macros may not split source.

## Implementation evidence

Focused O62 passes 8/8 after the initial 5/8 behavioral failure exposed invalid
fixtures. Exact dependency and workspace runs pass 2,371/2,371 and 6,402/6,402
with two skipped. Strict lint, all portability targets, format/static,
clean-Orca, and no-staged gates pass. Twenty-six mutations were killed; audit
and restored-production SHA-256 values are
`cb35772b4aab33c113ec145a22e50dbbcf9a898e15c014fa5768134410806b57` and
`ad7143d62fe6c5c4d17202a1b1c71b0932e3cb0fc666e105102289d456a29b9f`.
