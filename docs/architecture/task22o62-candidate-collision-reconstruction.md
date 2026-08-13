# Task 22O.62 architecture decision record

## Status

Accepted, implemented, gate-verified, and unconditionally approved by final independent six-axis implementation review.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3274-3288`, as one private,
lifecycle-neutral candidate collision-reconstruction operation. The Rust
destination is ordinary module
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_collision_reconstruction.rs`
with ordinary test children.

The operation consumes the owned O61 initial bridge result, the original
`area_to_be_bridge`, exact O48 `Flow`, current bridge angle, prior-completed
`CandidateSurface`s in exact future-composer append order, and retained
`CoordinateScale`. These surfaces are an input-provenance assumption: each
`new_polygons` value is postprocessed at `PrintObject.cpp:3292-3297` and
appended by the source-equivalent composer at `3304-3305`, never
raw/pre-expansion O43 candidate geometry. Producing that history remains
deferred. The operation returns owned
boundary polylines, collision-resolved pre-postprocessing bridge polygons, and
the selected angle. It does not activate the bridge transaction.

The exact private seam is:

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

## Source boundary and dependencies

The accepted source boundary is exactly the collision block after initial O53
construction:

1. `PrintObject.cpp:3277` expands the initial `bridging_area` by
   `3.0 * flow.scaled_spacing()`.
2. `PrintObject.cpp:3278-3284` visits `expanded_surfaces` in composer append
   order, intersects each `s.new_polys` with the expanded initial area, selects
   the first nonempty collision, copies `s.bridge_angle`, and breaks.
3. `PrintObject.cpp:3285-3287` reruns `construct_anchored_polygon` exactly once
   only after a collision, using the original `area_to_be_bridge`, unchanged
   O61 boundary lines, exact Flow, selected angle, and retained scale.

Direct reached dependencies are `Flow.hpp:62-69::scaled_spacing`,
`libslic3r.h:60-94::scale_`, the flat polygon `expand` and `intersection`
overloads in `ClipperUtils.hpp/.cpp`, O43 `CandidateSurface`, O53
`construct_anchored_polygon`, and O61 Polyline-to-Line conversion.

## Required semantics

- Compute source `scaled_spacing` as truncating `coord_t(scale_(flow.spacing))`
  in the trusted finite/representable Flow domain. Preserve C++ promotions:
  convert that integer to `double`, multiply by `3.0`, then convert the offset
  delta to `float` for the existing Miter/3 flat-path offset.
- Perform the expansion even when the completed-surface slice is empty. Offset
  receives the exact initial O61 `bridging_area`.
- Traverse caller-provided prior-completed surfaces without sorting, reversing,
  or grouping. Exact use of every supplied `completed_surfaces[i].new_polygons`
  is behavioral; raw-versus-postprocessed provenance is a static caller contract
  whose integration test/mutation remains deferred to the future composer. Run one default flat
  NonZero/no-safety polygon intersection per visited surface, with
  `s.new_polygons` as subject and the temporary expanded initial area as clip.
  The first nonempty result owns the selected angle; later surfaces are not
  inspected. Discard intersection geometry after testing emptiness.
- Do not replace the initial polygons merely because intersection geometry was
  computed. With no collision, preserve the consumed O61 bridge-polygon and
  boundary allocations unchanged plus the original angle. With a collision,
  call O53 once and replace only bridge polygons and angle while retaining the
  exact O61 boundary allocation. Errors consume the O61 input and return no
  partial or recoverable owned result.
- Preserve first-error precedence: expansion, then each visited intersection,
  then the conditional O53 reconstruction. Return no partial result on error.
- Borrow original area and prior surfaces without mutation. Consume O61 owned
  output so the no-collision branch retains its allocations. On collision the
  boundary allocation remains exact; only the newly reconstructed bridge-polygon
  allocation identity is outside parity.
- Production inputs are trusted internal state inherited from successful O61
  and O53 provenance: Flow width/spacing are positive and source-representable;
  scaled spacing and the final f32 expansion delta are finite and positive;
  current and prior angles are finite; original area, initial polygons, and
  boundary lines satisfy existing geometry/generated-coordinate arithmetic
  domains. Natural range-error fixtures deliberately probe outside trusted
  production state. No new defensive validation or fallback is added.
- O62 makes the existing source-exact O53 scaled-Flow helper and O61
  Polyline-to-Line helper narrowly `pub(super)` for sibling reuse; no duplicate
  arithmetic or broader public API is introduced.

## Included and deferred behavior

Included only: pinned lines `3274-3288` and their direct dependency closure.

Deferred: opening/closing/limiting/total-fill/top-area postprocessing at
`3292-3298`; `expansion_area` mutation; candidate append and per-layer surface
replacement; surrounding cluster/candidate composer; prepared successor and
lifecycle activation; second bridge pass; extrusion, motion, G-code, CLI, and
full golden parity.

## Architecture constraints

The seam remains `pub(in crate::project_slice)`, filesystem-free,
platform-neutral, and production-unwired. No option lookup or invented Ares
pipeline behavior is allowed. Source files must be at most 399 LOC and use
ordinary modules; `include!`, `include_bytes!`, and `include_str!` are forbidden
for source splitting.

## Verification decision

TDD must freeze exact no-collision allocation retention, boundary-allocation
retention on collision, first-collision ownership, composer-order break, spacing
cast/promotion for Normal and LargeBed scales, exact expansion input,
intersection subject/clip roles, discarded intersection geometry, actual
production Clipper and O53 behavior, empty completed history, empty initial
geometry, natural range errors, injected competing-error precedence,
repeatability, and complete input nonmutation. Reversible mutations must kill:
skipping expansion, scaling before integer truncation, f32-before-multiply,
wrong factor/join/miter, safety intersection, reversed intersection operands,
reversed/sorted traversal, failure to use supplied completed-surface polygons,
last-collision ownership, missing break, reuse of intersection output,
unconditional reconstruction, wrong angle/area/boundaries/Flow/scale
forwarding, ignored errors, allocation loss,
and output sorting.

Final gates require focused and O43-O62 dependency Nextest, Linux workspace
Nextest, strict Clippy, rustfmt, wasm32, x86_64/aarch64 Windows, x86_64/aarch64
macOS, diff/LOC/static/clean-Orca/no-staged checks, then independent six-axis
review and repair/re-review until unconditional approval.

## Implementation evidence

The first focused execution exposed three behavioral fixture failures (5/8) and
subsequent source-faithful fixture repair reached focused 8/8. Exact dependency
Nextest passes 2,371/2,371 and workspace Nextest passes 6,402/6,402 with two
skipped. Strict Clippy, wasm32, x86_64/aarch64 Windows, x86_64/aarch64 macOS,
rustfmt, diff/LOC/static, clean pinned Orca, and no-staged checks pass.

Twenty-six reversible arithmetic/kernel/operand/traversal/ownership/
forwarding/error/output mutations were killed. Production restored byte-exact
at SHA-256
`ad7143d62fe6c5c4d17202a1b1c71b0932e3cb0fc666e105102289d456a29b9f`;
combined mutation evidence SHA-256 is
`cb35772b4aab33c113ec145a22e50dbbcf9a898e15c014fa5768134410806b57`.
