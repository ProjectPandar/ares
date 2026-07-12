# M17: Brim path emission

## Goal
Generate deterministic first-layer brim path artifacts around current contours and route them through print path, move, extrusion, speed, and G-code output stages.

## Exit checklist
- `ares-core` exposes `generate_brims`, `LayerBrims`, `BrimPath`, `BrimOptions`, and `BrimType`.
- `SliceOptions` parses `brim_width`, `brim_object_gap`, and `brim_type` with Orca defaults and validation.
- Brim generation preserves represented layers, emits only for layer `0`, and defaults to no brims when `brim_width == 0`.
- Pipeline diagnostics include a `Brims` stage and `total_brim_path_count`.
- `slice` and `ares slice` output include brim metadata, artifact lines, print paths, moves, extrusion values, and feedrates.
- Existing movement/extrusion/speed command adjacency remains intact.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No exact Orca offset-polygon parity, inner brim geometry, mouse ears, painted brim geometry, automatic brim-width analysis, support brim, bridge detection, or support generation.
- No new workspace crates.
