# XY Slice Segments Spec

## Goal
Advance Ares from layer planning to the first geometric slicing operation: intersect imported STL triangles with planned layer Z positions and expose deterministic XY slice segments through the core API and `ares slice` output.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` calls `slice_volume` / `slice_mesh_ex` after layers are created, using each layer's slice Z positions.
- `OrcaSlicer/src/libslic3r/TriangleMeshSlicer.*` owns triangle mesh to plane slicing in libslic3r, producing 2D contours from mesh intersections.
- `OrcaSlicer/src/libslic3r/Layer.*` stores per-layer slices before later region/perimeter/infill processing.
- Ares M4 ports only the triangle-plane segment stage; contour stitching, polygon repair, perimeters, infill, and extrusion remain separate later milestones.

## Scope
Milestone 4 adds deterministic line-segment slice geometry. It does not yet stitch segments into polygons or generate extrusion E values. `slice(input, options)` still returns G-code-like bytes, now with per-layer segment metadata and non-extruding XY travel moves so CLI output visibly depends on mesh XY geometry.

## Functional requirements
1. `ares-core` exposes a public slicing geometry API:
   ```rust
   pub fn slice_layers(model: &Model, layers: &[Layer]) -> Result<Vec<LayerSlice>, SliceError>
   ```
2. `Point2`, `Segment2`, and `LayerSlice` are public core types. `LayerSlice` records the layer id, print Z, and deterministic ordered segments.
3. For each layer, `slice_layers` intersects every triangle against the horizontal plane at `Layer::print_z()`.
4. A triangle contributes one segment when the plane intersects it in exactly two distinct XY points. Duplicate points from a vertex-on-plane case are deduplicated. Whole coplanar triangles are ignored for M4 to avoid ambiguous filled-area semantics.
5. Segment endpoints are rounded to six decimals and segment ordering is deterministic by layer id, then endpoint coordinates.
6. Layers with no intersecting segments are retained with an empty segment list; this keeps layer planning and slice geometry indices aligned.
7. `slice` calls `slice_layers` after `plan_layers` and emits, per layer:
   - `; segment_count = N`
   - one `;SEGMENT:x1,y1 -> x2,y2` line per segment
   - non-extruding `G0 X... Y...` moves to both endpoints for each segment.
8. Existing `ares slice --options option.json -o output.gcode input.stl` continues to work for STL files and now includes segment metadata for positive-Z STL fixtures.
9. No new crates or dependencies are introduced.
10. `crates/ares-core/src/lib.rs` remains under the 400 LOC split threshold by moving existing lib tests into a sibling `tests.rs` module before adding new public API smoke coverage.

## Non-goals
- No polygon loop stitching, polygon repair, contour orientation, perimeters, infill, support, extrusion E values, or Orca G-code parity.
- No handling for coplanar triangle filled areas beyond explicitly ignoring them.
- No 3MF geometry extraction.
- No additional Orca option typing.

## Acceptance criteria
- Core tests cover interpolation through a triangle, vertex-on-plane deduplication, coplanar triangle ignore behavior, empty layer slice preservation, deterministic segment ordering, and `slice` output segment metadata/moves.
- CLI tests prove `ares slice --options option.json -o output.gcode input.stl` writes segment metadata for an STL with positive Z height.
- `docs/roadmap.md` and `docs/milestones/m4-xy-slice-segments.md` describe this milestone and defer polygon/path generation.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation review returns APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
