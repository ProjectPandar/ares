# Task 22O.61 — candidate anchored bridge

## Status

Implemented, gate-verified, and unconditionally approved by final independent
six-axis implementation review.

## Goal and source boundary

Port pinned `PrintObject.cpp:3268-3272`: append anchors to prepared boundaries,
conditionally clip those open polylines for Lightning overlap, flatten them to
lines, and invoke O53 once. Destination:
`prepare_infill/bridge_over_infill/candidate_anchored_bridge.rs` with ordinary
test children. O57/O58/O59/O60/O48/O53 provide every input; the future composer
owns provenance and O61 remains unwired.

## Interface

```rust
pub(in crate::project_slice) struct CandidateAnchoredBridge {
    pub(in crate::project_slice) boundary_polylines: Vec<Polyline>,
    pub(in crate::project_slice) bridging_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn construct_candidate_anchored_bridge(
    area_to_be_bridge: &[Polygon],
    boundary_polylines: Vec<Polyline>,
    anchors: &[Polyline],
    lightning_area: &[Polygon],
    bridging_flow: Flow,
    bridging_angle: f64,
    scale: CoordinateScale,
) -> Result<CandidateAnchoredBridge, ClipperError>;
```

## Behavior

1. Append cloned anchor polyline values after all owned boundaries in exact
   source order. Preserve points and duplicates; do not mutate borrowed anchors.
2. Empty Lightning skips all three Lightning geometry operations. Nonempty
   Lightning always performs flat no-safety-offset NonZero
   `intersection_polygons_paths` with bridge area as closed subject and
   Lightning as closed clip. Empty overlap skips expansion/open clipping; open,
   hierarchical, and safety-offset intersection are forbidden at this gate.
3. Nonempty overlap computes `(10.0_f64 / scale.factor()) as f32`, expands the
   original bridge area (never the overlap output) once with Miter/3, then
   replaces boundaries with one `intersection_open_polylines` call using
   boundaries as open subject and expanded area as closed clip.
4. Flatten the surviving polylines with the source two-pass count/reserve and
   adjacent-window traversal. One-point polylines emit nothing; zero-point inner
   polylines are outside source parity and prohibited in parity fixtures.
5. Call O53 once with original bridge area, exact lines, Flow, angle, and scale.
   Return O53 polygons and the exact post-append/post-clip boundaries. Add no
   union, normalization, inferred option, fallback, or alternate construction.
6. Error precedence is closed intersection, expansion, open intersection, O53.
   Borrowed inputs and their allocations remain unchanged; successful repeated
   source-safe calls are deterministic.

## Deferrals

Deferred: collision reconstruction 3274-3288; opening/closing/limiting/top
postprocessing 3290-3297; expansion mutation; expanded-surface and candidate
commit; layer surface rewrite; prepared successor/lifecycle; second internal
bridge pass; extrusion, motion, G-code, CLI, and golden parity.

## Acceptance

Begin with compiling behavioral RED. A removed source-derived oracle plus
private injected operation seam must discriminate:

- anchor append order, copies, duplicates, one-point values, and complete input
  nonmutation;
- exact outer Lightning-empty gate and nonempty-overlap gate;
- exact `intersection_polygons_paths` flat NonZero/no-safety kernel, closed
  subject/clip roles, and first-error ownership, discriminating open,
  hierarchical, safety-offset, and reversed-role variants;
- exact Normal/LargeBed `scale_(10)` delta bits, f64 divide-before-f32 cast,
  Miter/3 expansion of original bridge area rather than overlap output, and
  exact `intersection_open_polylines` open-subject/closed-clip roles;
- replacement by exact open-engine output, not append/reorder/fallback;
- source two-pass line flattening, no closure, repeated segments, and order;
- exactly one O53 call after all conditional work with exact area/line/Flow/
  angle/scale forwarding, exact returned polygons, and first O53 error;
- empty outer-boundary vectors, one-point-only polylines, and empty areas only
  through injected operation seams that do not execute O53, plus production-
  valid repeatability and allocation preservation; zero-point inner polylines
  are prohibited everywhere;
- real KSR provenance for candidate region Flow/angle/scale and O57/O58/O59
  inputs where available, without fixture-identity production behavior.

Trusted production calls require nonempty O58 area and nonempty flattened lines
satisfying O53. Deliberate empty/error-order discriminators use injected failures
or controlled Clipper-boundary errors and define no additional malformed-input
behavior.

Kill reversible gate/role/arithmetic/call/order/append/replace/flatten/forwarding
mutations and restore byte-exact. Rust tests reuse existing KSR support but do
not embed a temporary oracle or add fixture-derived production branches.

Use `pub(in crate::project_slice) mod` only, ordinary modules, and at most 399
lines per source; no include macro splitting. Final gates: focused O61; exact
O43-O61/Clipper/Flow dependency Nextest; Linux workspace Nextest; rustfmt;
strict Clippy; wasm32; x86_64/aarch64 Windows/macOS compile checks;
diff/LOC/static/clean-Orca/no-staged checks; independent six-axis repair and
re-review until unconditional approval.

## Implementation evidence

RED failed 0/5 and focused/KSR GREEN passes 9/9. Removed source-derived
source/binary/output SHA-256 values are `e6710df4...`, `edfcb578...`, and
`1b5db0f3...`; exact output freezes three appended lines plus Normal/LargeBed
delta bits. Twenty-three mutations were killed (audit
`f7ee7709f53252691139e2650c66c9f4c1c21cc8c4c384a76e9f11ae9b1a51f0`) and
source restored at `405c23e06e3fff145c066dba4b1d467bc822e6b26ec7001328ef05f87847144f`.
Dependency 2,363/2,363, workspace 6,394/6,394 with two skipped, strict Clippy,
all portability builds, format/static/clean/no-staged gates pass.
