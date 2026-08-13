# Task 22O.61 architecture decision record

## Status

Accepted, implemented, gate-verified, and unconditionally approved by final
independent six-axis implementation review.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:3268-3272`, as one private candidate anchored-bridge operation.
The Rust destination is ordinary module
`prepare_infill/bridge_over_infill/candidate_anchored_bridge.rs`:

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

The future composer supplies O58 area, owned O59 boundaries, O57 anchors and
Lightning area, O48 Flow, O60 angle, and retained scale. O61 infers no options
and stays unwired.

## Required semantics

1. Append copies of all source-valid anchor polylines after every existing
   boundary polyline, preserving container order, point order, duplicates, and
   one-point values. Selected inner polylines contain at least one point;
   zero-point behavior is outside parity. Source iterator insertion copies
   `Polyline` values; final allocation identity is not observable parity, but
   borrowed anchors remain unchanged.
2. Test `lightning_area.is_empty()` before geometry. If empty, skip the closed
   intersection, expansion, and open clipping entirely. Otherwise call flat,
   no-safety-offset NonZero `intersection_polygons_paths` exactly once with
   `area_to_be_bridge` as closed subject and `lightning_area` as closed clip,
   matching source `intersection(Polygons, Polygons)`. Do not use open,
   hierarchical, or safety-offset intersection at this gate.
3. Only when that closed intersection is nonempty, compute source `scale_(10)`
   as f64 `10.0 / scale.factor()`, cast to f32, expand the original
   `area_to_be_bridge` (not the overlap output) once with default closed Miter/3,
   then replace the entire appended boundary vector with one
   `intersection_open_polylines` call using boundaries as open subject and the
   expanded area as closed clip. Preserve engine output order; do not append the unclipped
   input, recombine independently, sort, or fall back.
4. Convert the resulting boundary polylines to lines exactly as
   `Polyline.hpp:180-193`: count every source-valid `len - 1`, reserve the
   complete line capacity, then emit adjacent windows in polyline/point order.
   One-point polylines emit no line; selected source polylines have at least one
   point because empty inner polylines are undefined in the pinned overload.
5. Call O53 `construct_anchored_polygon` exactly once with the original
   `area_to_be_bridge`, exact flattened lines, exact O48 Flow, exact O60 angle,
   and exact scale. Return its ordered polygons together with the post-append or
   post-clip boundary polylines for the source collision-reconstruction caller.
6. Propagate the first error in source order: closed intersection, expansion,
   open intersection, then O53. Preserve every borrowed area, anchor, and
   Lightning input and every copied Flow, angle, and scale bit.

Direct closure is pinned `PrintObject.cpp:3268-3272`, `libslic3r.h:89-96`
scaling, `Polyline.hpp:16-32,180-193`, `ClipperUtils.cpp:702-703` flat closed
intersection, `ClipperUtils.hpp` default Miter/3 expand,
`ClipperUtils.cpp:926-927` open `intersection_pl`, Ares
`intersection_polygons_paths` and `intersection_open_polylines`, and O53.

The trusted production domain is source-valid O57-O60/O48/O53 state: nonempty
O58 bridge area, nonempty flattened lines, finite source Flow and angle
satisfying O53, Clipper-safe coordinates, every selected inner polyline having
at least one point, `10.0 / scale.factor()` representable as positive f32, and
all source vector counts/capacities representable by Rust `usize`. O61 adds no
validation. Empty-area, empty-outer-boundary, or one-point-only-line cases are
permitted only through injected operation-order seams that do not invoke O53;
they freeze dispatch, not malformed production behavior.

## Consequences

O61 closes anchor append, conditional Lightning clipping, and initial O53
construction only. Collision reconstruction at 3274-3288, postprocessing at
3290-3297, expanded-surface commit, layer/surface rewrite, prepared successor
and lifecycle activation, second internal bridge pass, extrusion, motion,
G-code, CLI, and golden parity remain deferred.

Register `pub(in crate::project_slice) mod candidate_anchored_bridge;` with
ordinary test children. Every source contains at most 399 lines. `include!`,
`include_bytes!`, and `include_str!` are prohibited for splitting. Linux
runtime, wasm32, x86_64/aarch64 Windows, and x86_64/aarch64 macOS compile gates
remain required.

## Implementation evidence

Behavioral RED failed 0/5; GREEN plus real-KSR provenance pass 9/9. A removed
repeatable source-derived driver freezes anchor append/line order and exact
Normal/LargeBed `scale_(10)` bits. Driver source/binary/output SHA-256 values are
`e6710df487466d3258790f6be55782990554fc5034920add69892a555dd85a78`,
`edfcb578e3b9ee43ed7490119734f36fdd051e15cc9a23174c00c590f51087ad`, and
`1b5db0f377afd362e0dff95e45a97f1190cc07990dfa97343059e6706e1fc6d8`;
O53 retains its independently approved exact geometry oracle.

Twenty-three gate/append/role/arithmetic/replace/line/forwarding/output and
production-wiring mutations were killed; audit SHA-256 is
`f7ee7709f53252691139e2650c66c9f4c1c21cc8c4c384a76e9f11ae9b1a51f0`, and
production restored byte-exact at
`405c23e06e3fff145c066dba4b1d467bc822e6b26ec7001328ef05f87847144f`.
Final gates pass dependency 2,363/2,363, workspace 6,394/6,394 with two skipped,
strict Clippy, wasm32, four desktop cross-target checks, rustfmt,
diff/LOC/static, clean Orca, and no staged files.
