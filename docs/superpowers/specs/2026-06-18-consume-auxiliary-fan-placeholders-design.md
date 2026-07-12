# Consume Auxiliary Fan Placeholders Design

## Goal

Port the OrcaSlicer startup placeholder injection for auxiliary fan controls into Ares so existing `machine_start_gcode` templates can consume already-registered auxiliary fan options during generated G-code output. This is a concrete G-code behavior slice, not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2694-2735`: computes `max_additional_fan` from `ToolOrdering::cal_max_additional_fan(print.config())`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2836-2839`: injects `max_additional_fan`, `first_x_layer_fan_speed`, `close_additional_fan_first_x_layers`, and `additional_fan_full_speed_layer` into the placeholder parser before startup G-code processing.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082-3101`: processes `machine_start_gcode` through the placeholder parser and writes the processed custom start G-code before printing layers.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:957-968`: defines `cal_max_additional_fan` as the maximum `additional_cooling_fan_speed` for extruders used by layers.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4660-4695` and `PrintConfig.hpp:1475-1478`: define the existing auxiliary fan option names and defaults.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5789-5796` and `PrintConfig.hpp:1386`: define `machine_start_gcode` as a string option with Orca's built-in `G28` / `G1 Z5` default.

## Rust Destination Boundary

- Add a focused `crates/ares-core/src/gcode_placeholders.rs` module for a small Orca-compatible placeholder replacer used by generated G-code.
- Extend `crates/ares-core/src/options/auxiliary_fan.rs` only as needed to parse the first values of `close_additional_fan_first_x_layers`, `additional_fan_full_speed_layer`, and `first_x_layer_fan_speed`, plus expose the existing `additional_cooling_fan_speed` as the single-extruder `max_additional_fan` value.
- Wire the processed `machine_start_gcode` output into `crates/ares-core/src/gcode.rs` after the current generated startup temperature/fan commands and before the first layer.
- Keep `gcode.rs` wiring to one helper call. Placeholder parsing, option extraction, newline normalization, and validation belong in focused modules so the `crates/ares-core/src/*.rs` 400 LOC gate remains satisfied.
- Add focused tests under `crates/ares-core/src/tests/auxiliary_fan_gcode.rs` and `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs`.

## Behavior

- If `machine_start_gcode` is absent or empty, generated G-code stays unchanged except for no-op parser setup.
- Ares accepts `machine_start_gcode` only as a JSON string. Any non-string value returns `SliceError::InvalidInput` mentioning `machine_start_gcode`.
- Orca's non-empty default `G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n` is out of scope for this slice because Ares has historically emitted no machine start custom G-code unless the user provides it; changing that default would alter every generated file.
- If `machine_start_gcode` is present, Ares emits the processed template before `;LAYER_CHANGE`.
- Replace bracket placeholders for these exact keys:
  - `[max_additional_fan]`
  - `[first_x_layer_fan_speed]`
  - `[close_additional_fan_first_x_layers]`
  - `[additional_fan_full_speed_layer]`
- `max_additional_fan` uses Ares' current single-extruder boundary: the first `additional_cooling_fan_speed` value, defaulting to `0`.
- The three dedicated auxiliary fan placeholders use the first value of their option vectors, matching Orca's scalar use of the initial extruder in startup placeholder context.
- Values are formatted without unnecessary trailing `.0` when integral, so `35.0` becomes `35` and `12.5` stays `12.5`.
- Unknown placeholders remain unchanged.
- Invalid values for the three newly consumed placeholder inputs return `SliceError::InvalidInput` naming the offending option.

## Deferred Behavior

- Do not implement auxiliary fan runtime ramp behavior from these options in this slice. Current upstream search found no `CoolingBuffer` consumption of `close_additional_fan_first_x_layers`, `additional_fan_full_speed_layer`, or `first_x_layer_fan_speed`; they are injected as placeholders.
- Do not add a full expression language, conditionals, array indexing, or custom G-code parser parity.
- Do not implement multi-extruder `ToolOrdering` layer traversal. Ares currently has a single generated toolpath stream, so `max_additional_fan` is the first configured value.
- Do not add or modify option registry metadata.

## Acceptance Criteria

- A `machine_start_gcode` template containing the four auxiliary fan placeholders emits concrete values before the first layer marker.
- Default auxiliary placeholder values are `0`, `0`, `1`, and `0` for `max_additional_fan`, `first_x_layer_fan_speed`, `close_additional_fan_first_x_layers`, and `additional_fan_full_speed_layer`.
- `additional_cooling_fan_speed = 70` makes `[max_additional_fan]` emit `70`.
- `first_x_layer_fan_speed = 12.5`, `close_additional_fan_first_x_layers = 3`, and `additional_fan_full_speed_layer = 8` emit `12.5`, `3`, and `8`.
- Unknown placeholders in `machine_start_gcode` remain unchanged.
- Existing auxiliary fan layer commands still behave as before.
- Verification passes `cargo fmt --check`, targeted ares-core tests for this slice, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the ares-core 400 LOC gate.
