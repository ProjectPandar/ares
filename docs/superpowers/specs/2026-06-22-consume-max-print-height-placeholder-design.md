# Consume Max Print Height Placeholder Design

## Scope

Port OrcaSlicer's `machine_start_gcode` placeholder `max_print_height` into Ares G-code output by consuming the existing `printable_height` option. This is a concrete G-code behavior slice, not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:3009` sets `max_print_height` from `m_config.printable_height` as a `ConfigOptionInt`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3082` processes `machine_start_gcode` through the placeholder parser after those variables are installed.
- `OrcaSlicer/src/libslic3r/Config.hpp:957-959` defines `ConfigOptionInt(double)` as `int(floor(value + 0.5))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1539` declares `printable_height` as `ConfigOptionFloat` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:757-764` defines `printable_height` with minimum `0`, maximum `214700`, and default `100.0`.

## Ares Destination Boundary

- Implement placeholder rendering in `crates/ares-core/src/gcode_placeholders.rs::machine_start_gcode`.
- Add focused behavior tests under `crates/ares-core/src/tests/`.
- Reuse existing `SliceOptions` values and the existing option registry default for `printable_height`.
- Do not change option metadata, CLI behavior, WASM bindings, file I/O, UI, OpenGL, or unrelated placeholder scopes.

## Behavior

- When `machine_start_gcode` contains `[max_print_height]`, Ares renders it before the first generated layer G-code.
- The rendered value comes from `printable_height`.
- Missing `printable_height` uses the existing registry default, matching Orca's default `100`.
- Numeric JSON values and numeric string values are accepted.
- Because Orca installs `max_print_height` as `ConfigOptionInt`, Ares rounds numeric values with Orca's `floor(value + 0.5)` rule and renders the placeholder as an integer string.
- `printable_height` values must be finite and non-negative. Invalid values produce `SliceError::InvalidInput`.
- `[max_print_height]` is not expanded in layer-change or other unrelated custom G-code scopes.

## Deferred Behavior

- Do not implement printer-height collision checks, `extruder_printable_height`, or multi-extruder printable-height validation.
- Do not add support for `[max_print_z]`, bed mesh, model metadata, bed-temperature placeholders, or other Orca placeholders in the same `GCode.cpp` block.
- Do not change the existing `first_layer_height`, `z_offset`, `retract_length`, `num_extruders`, or `total_layer_count` behavior.

## Acceptance Criteria

- A nextest RED run proves `[max_print_height]` is currently not rendered in `machine_start_gcode`.
- Focused nextest GREEN tests prove configured integer, default, numeric string composition, decimal rounding, invalid input rejection, and layer-change non-expansion behavior.
- Full verification uses `cargo nextest run --workspace`, not `cargo test`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust LOC guard pass before commit.
