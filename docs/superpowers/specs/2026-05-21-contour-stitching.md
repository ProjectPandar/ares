# Contour Stitching Spec

## Goal
Advance Ares from raw per-layer XY slice segments to deterministic closed contours, giving later perimeter/infill milestones a polygon-like boundary while still avoiding full polygon boolean repair and extrusion generation.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/TriangleMeshSlicer.cpp` converts mesh intersections into loops and then into `ExPolygons` through `make_expolygons` / `make_expolygons_simple` paths.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` receives `ExPolygons` from `slice_mesh_ex` and stores them on layers before region/perimeter processing.
- `OrcaSlicer/src/libslic3r/Layer.cpp::make_slices` merges region slices into layer-level `lslices`.
- Ares M5 ports only the simple closed-loop stitching step from already deterministic segments; polygon boolean union, hole assignment, simplification, offsetting, and repair remain later milestones.

## Scope
Milestone 5 adds contour stitching for simple, non-branching segment graphs. A contour is a closed sequence of `Point2` values. Open chains, branching/non-manifold graphs, and duplicate canonical segment edges are rejected with `SliceError::InvalidInput` so future repair milestones can handle them explicitly.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub fn stitch_layer_slices(slices: &[LayerSlice]) -> Result<Vec<LayerContours>, SliceError>
   ```
2. Public types:
   - `Contour`, storing ordered `Point2` vertices.
   - `LayerContours`, storing layer id, print Z, and contours for one layer.
3. Each `LayerSlice` produces one `LayerContours` output, even when it has zero contours.
4. Segments are stitched when endpoints match exactly after the six-decimal rounding already used by `Point2`/`Segment2`.
5. A simple closed square made from four unordered segments stitches into one contour with four unique vertices; the closing point is not repeated in storage.
6. Output ordering is deterministic by ascending layer id, then contour first point, then full vertex sequence.
7. Contour orientation is normalized counter-clockwise for positive-area loops.
8. Open segment chains, branching endpoint graphs, and duplicate canonical `Segment2` edges return `SliceError::InvalidInput`.
9. `slice` emits, per layer:
   - `; contour_count = N`
   - one `;CONTOUR:x1,y1 -> x2,y2 -> ...` line per contour.
10. Existing segment metadata remains in `slice` output for diagnostics.
11. No new crates or dependencies are introduced.
12. `crates/ares-core/src/lib.rs` remains under 400 LOC.

## Non-goals
- No hole assignment, boolean union, self-intersection repair, contour simplification, offsetting, perimeters, infill, extrusion E values, or Orca G-code parity.
- No support for branching/non-manifold segment graphs or duplicate edge repair beyond typed rejection.
- No 3MF geometry extraction.
- No new option families.

## Acceptance criteria
- Core tests cover unordered square stitching, multiple contours deterministic ordering, reversed layer input ordering, clockwise input normalization to counter-clockwise, empty layer preservation, open-chain rejection, branching rejection, duplicate-edge rejection, and `slice` contour output.
- CLI tests prove `ares slice --options option.json -o output.gcode input.stl` writes contour metadata for an STL fixture that yields a closed square contour.
- `docs/roadmap.md` and `docs/milestones/m5-contour-stitching.md` describe this milestone and defer polygon repair/path generation.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation review returns APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
