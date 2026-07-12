# Consume Temperature Placeholder Design

## Goal

Port OrcaSlicer's machine-start `[temperature]` placeholder into Ares G-code rendering so the existing `nozzle_temperature` option reaches concrete custom start G-code output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2828` registers machine-start `temperature` as `new ConfigOptionInts(print.config().nozzle_temperature)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478-6485` defines `nozzle_temperature` as `ConfigOptionInts` with min `0`, max `max_temp`, and default `ConfigOptionInts { 200 }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1568` stores `nozzle_temperature` in `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:907-943` expands legacy bracket placeholders by using the current extruder id for vector variables and falling back to index `0` when the current id is out of range.

## Current Ares Context

- `crates/ares-core/src/options/nozzle_temperature.rs` already parses `nozzle_temperature` for other-layer nozzle commands, defaulting to `200` and accepting scalar, array, and separated-string integer forms through `temperature_vector::parse_integer_vector`.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` already renders adjacent machine-start temperature placeholders, including `[first_layer_temperature]`, `[bed_temperature]`, `[first_layer_bed_temperature]`, and `[bed_temperature_initial_layer_single]`.
- `crates/ares-core/src/gcode_startup.rs` suppresses automatic first-layer nozzle startup commands when the rendered custom machine-start G-code contains an explicit nozzle temperature command.
- `crates/ares-core/src/tests/nozzle_temperature_gcode.rs` already covers first-layer temperature placeholder behavior and automatic nozzle startup suppression.

## Included Behavior

1. Add a `SliceOptions` accessor for the machine-start `temperature` placeholder that returns the initial extruder value from `nozzle_temperature`.
2. Missing `nozzle_temperature` renders Orca's default `200`.
3. Supported input forms reuse the existing `nozzle_temperature` parser:
   - scalar integer renders that integer;
   - integer array renders the first element for unindexed `[temperature]`;
   - comma-separated or semicolon-separated string renders the first parsed element.
4. Replace `[temperature]` in `machine_start_gcode` with the selected integer value.
5. Keep `[temperature]` literal in `layer_change_gcode`; this slice only ports the machine-start registration at `GCode.cpp:2828`.
6. If `machine_start_gcode` contains a nozzle command using `[temperature]`, the rendered custom start string still participates in the existing automatic first-layer nozzle startup suppression.
7. Invalid `nozzle_temperature` values continue to return `SliceError::InvalidInput` mentioning `nozzle_temperature`.

## Deferred Behavior

- Do not implement brace-expression parser support such as `{temperature[0]}`.
- Do not implement legacy indexed bracket aliases such as `[temperature_1]`.
- Do not change `[first_layer_temperature]`, other-layer automatic nozzle temperature scheduling, tool-change temperature behavior, filament start/end G-code placeholder behavior, or multi-extruder current-tool selection beyond Ares' initial extruder scope.
- Do not change `nozzle_temperature` option metadata, add dependencies, add file I/O, add UI behavior, or introduce independent Ares pipeline concepts.

## Rust Destination Boundary

- Modify `crates/ares-core/src/options/nozzle_temperature.rs` to expose the machine-start `temperature` accessor.
- Modify `crates/ares-core/src/gcode_machine_start_placeholders.rs` to replace `[temperature]` in machine-start G-code.
- Extend focused coverage in `crates/ares-core/src/tests/nozzle_temperature_gcode.rs`.

## Acceptance Criteria

- `cargo nextest run -p ares-core temperature_placeholder` initially fails before implementation and passes after implementation.
- Tests prove configured scalar, default, first array value, serialized string first value, composition with existing temperature placeholders, layer-change literal scope, automatic nozzle startup suppression after rendering, and invalid `nozzle_temperature` input behavior.
- Existing adjacent temperature tests still pass with `cargo nextest run -p ares-core nozzle_temperature_gcode other_layer_temperature_gcode`.
- Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC <= 400.

## Safety And Rollback

The change is limited to in-memory option parsing and machine-start string substitution in `ares-core`. It has no external I/O, no dependency changes, and no persistent state. Rollback is reverting the accessor, replacement, tests, spec, and plan files from this slice.

## Spec Self-Review

- Placeholder scan: no unresolved placeholder markers.
- Scope check: one source-cited machine-start placeholder only.
- Ambiguity check: unindexed `[temperature]` explicitly maps to Ares' initial extruder value from `nozzle_temperature`; indexed and brace-expression forms are deferred.
- Consistency check: automatic nozzle startup suppression uses the already-rendered machine-start string, so `M104 S[temperature]` and `M109 S[temperature]` should suppress duplicate startup commands after replacement.
