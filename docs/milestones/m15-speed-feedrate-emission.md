# M15: Speed / feedrate emission

## Goal
Attach deterministic speed artifacts to extrusion moves and emit first `F` feedrate values on path-following XY G-code commands.

## Exit checklist
- `ares-core` exposes `generate_speed_moves`, `LayerSpeedMoves`, `SpeedMove`, and `SpeedOptions`.
- `SliceOptions` parses `travel_speed`, `outer_wall_speed`, and `sparse_infill_speed` as positive finite mm/s values.
- Default speeds match Orca: travel `120`, outer wall `60`, sparse infill `100` mm/s.
- Speed artifacts preserve extrusion move metadata and attach mm/s plus mm/min feedrate values.
- `SlicingPipeline` includes speed artifacts and diagnostics after extrusion artifacts.
- `slice` and `ares slice` output include speed metadata plus `F` values on current path-following `G0`/`G1` commands.
- Existing path, move, and extrusion metadata remains unchanged except for appending speed stage/metadata and adding feedrates.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No acceleration, jerk, volumetric speed limiting, cooling, first-layer speed overrides, retraction/wipe speeds, support/bridge/skirt/brim speeds, travel optimization, or Orca G-code parity.
- No new workspace crates.
