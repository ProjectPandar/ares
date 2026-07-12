# M5: Contour stitching

## Goal
Stitch deterministic per-layer XY slice segments into simple closed contours for later perimeter and infill milestones.

## Exit checklist
- `ares-core` exposes `stitch_layer_slices(slices) -> Result<Vec<LayerContours>, SliceError>`.
- Public `Contour` and `LayerContours` types describe per-layer closed loops.
- Closed segment loops stitch from unordered segments.
- Contour vertices are unique and omit a repeated closing point.
- Contour orientation is normalized counter-clockwise for positive-area loops.
- Empty layer contour outputs are preserved.
- Open chains, branching segment graphs, and duplicate canonical segment edges return typed `SliceError::InvalidInput`.
- `slice` emits per-layer `contour_count` and `;CONTOUR` metadata while retaining segment metadata.
- `ares-cli` observes contour-aware output for STL fixtures.
- `crates/ares-core/src/lib.rs` remains under 400 LOC.
- No new crates or dependencies are introduced.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No hole assignment, boolean union, polygon repair, perimeters, infill, supports, extrusion E values, or Orca G-code parity.
- No 3MF geometry extraction.
- No new option families.
