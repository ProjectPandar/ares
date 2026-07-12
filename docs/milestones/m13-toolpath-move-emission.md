# M13: Toolpath move emission

## Goal
Convert ordered print path artifacts into deterministic travel/print moves and emit the first path-following `G0`/`G1` XY commands.

## Exit checklist
- `ares-core` exposes `generate_toolpath_moves`, `LayerToolpathMoves`, `ToolpathMove`, and `ToolpathMoveKind`.
- Each print path emits a travel move to its first point.
- Sparse infill paths emit open print moves through their endpoints.
- External perimeter paths emit closed print moves back to their first point.
- `SlicingPipeline` includes move artifacts and diagnostics.
- `slice` and `ares slice` output include total and per-layer move metadata plus `G0`/`G1` XY commands for moves.
- Existing path metadata remains unchanged except for appending move stage/metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No extrusion E values, speeds, acceleration, jerk, retraction, Z-hop, seam placement, travel optimization, support/bridge/skirt/brim, arc fitting, or Orca G-code parity.
- No new workspace crates.
