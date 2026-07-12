# Consume BBL Bed Temperature Placeholder Design

## Goal

Consume the existing OrcaSlicer `bbl_bed_temperature_gcode` option into a concrete Ares `machine_start_gcode` placeholder result. This is a narrow G-code output slice, not another option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1353-1355` declares `gcode_add_line_number`, `bbl_bed_temperature_gcode`, and `gcode_flavor` inside the `GCodeConfig` option tuple list.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2996` sets the custom-start placeholder value with `placeholder_parser().set("bbl_bed_temperature_gcode", new ConfigOptionBool(false));`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082-3101` renders `machine_start_gcode` through the placeholder parser before writing it and before normal layer G-code.

## Rust Destination Boundary

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` owns Ares machine-start placeholder rendering.
- `crates/ares-core/src/tests/bbl_bed_temperature_gcode.rs` will own focused integration coverage for the placeholder.
- `crates/ares-core/src/tests/mod.rs` will register the new test module.

## Included Behavior

1. `[bbl_bed_temperature_gcode]` in `machine_start_gcode` renders as `0`, matching Orca's current hard-coded `ConfigOptionBool(false)` placeholder value.
2. The placeholder composes with existing machine-start placeholders such as `[bed_temperature_initial_layer]` and `[first_layer_bed_temperature]`.
3. The placeholder remains literal in non-machine-start scopes, including `layer_change_gcode`.
4. No option input is read for this placeholder. Even if the user supplies `bbl_bed_temperature_gcode: true`, the rendered machine-start placeholder remains `0` because the cited Orca G-code boundary sets the parser variable to `false`.
5. The rendered custom start G-code continues to participate in existing startup temperature suppression exactly as before.

## Deferred Behavior

- Full `bbl_bed_temperature_gcode` option storage/export semantics beyond the cited placeholder assignment.
- Any future branch where Orca sets the placeholder to `true`.
- Full Orca placeholder parser parity, expression parsing, and conditional evaluation.
- Bed-temperature formula expansion beyond the already implemented Ares bed-temperature placeholders.
- UI behavior, preset migration behavior, and config class generation.
- Any movement, extrusion, fan, temperature command generation, or post-processor changes outside the rendered custom start string.

## Acceptance Criteria

- A focused RED nextest run fails before implementation because `[bbl_bed_temperature_gcode]` remains literal in `machine_start_gcode`.
- After implementation, the focused nextest run passes and proves the placeholder renders as `0`.
- Tests prove explicit `bbl_bed_temperature_gcode: true` does not change the machine-start placeholder value.
- Tests prove the placeholder composes with existing bed-temperature placeholders.
- Tests prove the placeholder remains literal in `layer_change_gcode`.
- Full verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust files remain at or below 400 LOC.

## Self-Review

- No placeholder or TODO text is left in this spec.
- Scope is intentionally limited to the source-cited Orca placeholder assignment and Ares machine-start rendering.
- The spec does not ask for full config class generation or a new Ares-owned pipeline.
- The behavior is externally visible in generated G-code and therefore directly consumes an existing option into concrete runtime output.
