# M14: Extrusion value emission

## Goal
Attach deterministic absolute filament `E` values to existing print moves and emit them on path-following `G1` XY commands.

## Exit checklist
- `ares-core` exposes `generate_extrusion_moves`, `LayerExtrusionMoves`, `ExtrusionMove`, and `ExtrusionOptions`.
- Extrusion values are derived from move segment length, layer height, resolved line width, and first filament diameter.
- Width resolution supports `line_width`, `outer_wall_line_width`, and `sparse_infill_line_width` as millimeters or nozzle-relative percentages.
- Role-specific zero widths fall back to `line_width`; zero `line_width` falls back to Orca's current automatic width for external perimeter and sparse infill roles.
- Travel moves emit no `E`; print moves emit absolute cumulative `E` values.
- `SlicingPipeline` includes extrusion artifacts and diagnostics after move artifacts.
- `slice` and `ares slice` output include total and per-layer extrusion metadata plus `G1 ... E...` commands for print moves.
- Existing path and move metadata remains unchanged except for appending extrusion stage/metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No speeds/feedrates, retraction, Z-hop, seam placement, pressure advance, multi-extruder/toolchange behavior, relative extrusion mode, support/bridge/skirt/brim, or Orca G-code parity.
- No change to current infill path spacing.
- No new workspace crates.
