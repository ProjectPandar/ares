# M16: Skirt path emission

## Goal
Generate deterministic first skirt path artifacts around current contours and route them through print path, move, extrusion, speed, and G-code output stages.

## Exit checklist
- `ares-core` exposes `generate_skirts`, `LayerSkirts`, `SkirtPath`, and `SkirtOptions`.
- `SliceOptions` parses `skirt_loops`, `skirt_distance`, `skirt_height`, and `skirt_speed` with Orca defaults.
- Skirt generation preserves represented layers and emits only for enabled layers.
- Pipeline diagnostics include a `Skirts` stage and `total_skirt_path_count`.
- `slice` and `ares slice` output include skirt metadata, artifact lines, print paths, moves, extrusion values, and feedrates.
- Existing movement/extrusion/speed command adjacency remains intact.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No brim, support, bridge, draft shield, multi-extruder skirt, or exact Orca offset-polygon parity.
- No new workspace crates.
