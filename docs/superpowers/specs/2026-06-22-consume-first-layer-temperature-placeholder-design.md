# Consume First Layer Temperature Placeholder Design

## Scope

Port OrcaSlicer's `machine_start_gcode` placeholder `[first_layer_temperature]` into Ares G-code output by consuming the existing `nozzle_temperature_initial_layer` option. This is a concrete start-G-code placeholder behavior slice, not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:3008` sets `first_layer_temperature` from `m_config.nozzle_temperature_initial_layer` as `ConfigOptionInts`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3082` processes `machine_start_gcode` through the placeholder parser after those variables are installed.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:906-943` expands vector placeholders by using `current_extruder_id`, falling back to index `0` when the index is out of range.
- `OrcaSlicer/src/libslic3r/Config.hpp:1023-1040` serializes `ConfigOptionInts` elements as integer strings.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1533` declares `nozzle_temperature_initial_layer` as `ConfigOptionInts`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316-3323` defines `nozzle_temperature_initial_layer` with minimum `0` and default `{200}`.

## Ares Destination Boundary

- Implement placeholder rendering in `crates/ares-core/src/gcode_placeholders.rs::machine_start_gcode`.
- Add focused behavior tests in `crates/ares-core/src/tests/nozzle_temperature_gcode.rs`.
- Reuse the existing `temperature_vector::parse_integer_vector` semantics by adding a `SliceOptions` helper for the full first-layer nozzle-temperature vector.
- Do not change option metadata, CLI behavior, WASM bindings, file I/O, UI, OpenGL, or unrelated placeholder scopes.

## Behavior

- When `machine_start_gcode` contains `[first_layer_temperature]`, Ares renders it before the first generated layer G-code.
- The rendered value comes from `nozzle_temperature_initial_layer`.
- Missing `nozzle_temperature_initial_layer` uses the existing Orca/Ares default `200`.
- Numeric JSON values, numeric string values, comma/semicolon-separated strings, and integer arrays keep the same validation semantics as the existing first-layer nozzle temperature runtime option.
- Since this start-G-code path currently has no Ares toolchange or filament-map context, unindexed `[first_layer_temperature]` renders index `0`, matching Orca's initial-extruder `0` case.
- If multiple first-layer nozzle temperatures are supplied, `[first_layer_temperature]` renders the first value; the existing automatic nozzle startup command behavior remains unchanged.
- Invalid `nozzle_temperature_initial_layer` values produce `SliceError::InvalidInput`.
- `[first_layer_temperature]` is not expanded in layer-change or other unrelated custom G-code scopes.

## Deferred Behavior

- Do not implement indexed placeholder forms such as `[first_layer_temperature_1]` or expression-indexed placeholder forms.
- Do not implement Orca toolchange, filament-map, `initial_extruder_id`, or multi-tool start placeholder routing in this slice.
- Do not implement `[first_layer_bed_temperature]`, `[bed_temperature_initial_layer]`, `[bed_temperature]`, `[chamber_temperature]`, or other temperature placeholders in the same `GCode.cpp` block.
- Do not change automatic first-layer nozzle startup command suppression, Reprap/Klipper flavor behavior, or second-layer temperature transition behavior.

## Acceptance Criteria

- A nextest RED run proves `[first_layer_temperature]` is currently not rendered in `machine_start_gcode`.
- Focused nextest GREEN tests prove configured value rendering, default rendering, first-value selection from a multi-value input, composition with existing start placeholders, invalid input rejection, and layer-change non-expansion behavior.
- Existing focused nozzle-temperature startup tests remain green.
- Full verification uses `cargo nextest run --workspace`, not `cargo test`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and Rust LOC guard pass before commit.
