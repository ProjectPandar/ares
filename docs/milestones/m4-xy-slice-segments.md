# M4: XY slice segments

## Goal
Intersect imported STL triangles with planned layer Z planes and expose deterministic XY line segments as the first geometric slicing output.

## Exit checklist
- `ares-core` exposes `slice_layers(model, layers) -> Result<Vec<LayerSlice>, SliceError>`.
- Public `Point2`, `Segment2`, and `LayerSlice` types describe per-layer slice geometry.
- Triangle-plane intersections produce one segment for exactly two distinct intersection points.
- Vertex-on-plane duplicate points are deduplicated.
- Coplanar triangles are ignored explicitly for this milestone.
- Segment coordinates and ordering are deterministic.
- Empty layer slices are retained to preserve layer-index alignment.
- `slice` emits per-layer segment counts, `;SEGMENT` metadata, and non-extruding `G0 X/Y` moves.
- `ares-cli` continues owning filesystem behavior and observes segment-aware core output.
- `crates/ares-core/src/lib.rs` remains under 400 LOC.
- No new crates or dependencies are introduced.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No polygon loop stitching, perimeters, infill, supports, extrusion E values, or Orca G-code parity.
- No 3MF geometry extraction.
- No new option families.
