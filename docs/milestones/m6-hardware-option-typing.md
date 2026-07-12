# M6: Hardware option typing

## Goal
Type the first OrcaSlicer machine/filament vector options needed by later extrusion and flow milestones while preserving unknown option keys.

## Exit checklist
- `ares-core` exposes `HardwareOptions` and typed `SliceOptions` accessors for `nozzle_diameter`, `filament_diameter`, `min_layer_height`, and `max_layer_height`.
- Accessors accept Orca profile-style JSON numbers, numeric strings, semicolon/comma numeric strings, arrays of numbers, and arrays of numeric strings.
- Accessors use Orca-compatible defaults for this subset: nozzle `[0.4]`, filament `[1.75]`, min layer `[0.07]`, max layer `[0.0]`.
- Invalid numeric forms and invalid threshold values return `SliceError::InvalidInput`.
- Unknown options remain preserved in `SliceOptions::values()`.
- `slice` and `ares slice` emit hardware option metadata in generated G-code.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No profile inheritance, preset directory loading, or Orca resource discovery.
- No extrusion, flow, speed, acceleration, temperature, cost, or material calculations.
- No new workspace crates.
