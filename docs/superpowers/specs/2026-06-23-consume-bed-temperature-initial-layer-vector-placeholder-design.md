# Consume Bed Temperature Initial Layer Vector Placeholder Design

## Goal

Consume OrcaSlicer's `bed_temperature_initial_layer_vector` machine-start placeholder into concrete Ares generated G-code output. This is a narrow runtime placeholder slice, not another option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2996-3000` sets the bed-temperature custom-start placeholders, including `placeholder_parser().set("bed_temperature_initial_layer_vector", new ConfigOptionString());`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082-3101` renders `machine_start_gcode` through the placeholder parser before startup temperature suppression and final custom-start emission.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10882-10899` documents the adjacent custom-placeholder config definitions used by this `GCode.cpp` placeholder setup area; `bed_temperature_initial_layer_vector` itself is a `GCode.cpp` runtime parser variable in the cited boundary.

## Rust Destination Boundary

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` owns Ares machine-start placeholder rendering.
- `crates/ares-core/src/tests/bed_temperature_initial_layer_vector_placeholder_gcode.rs` will own focused integration coverage.
- `crates/ares-core/src/tests/mod.rs` will register the new test module.

## Included Behavior

1. `[bed_temperature_initial_layer_vector]` in `machine_start_gcode` renders as an empty string, matching Orca's current `ConfigOptionString()` placeholder value at the cited boundary.
2. The placeholder composes with existing machine-start bed-temperature placeholders such as `[bed_temperature_initial_layer]`, `[first_layer_bed_temperature]`, and `[bbl_bed_temperature_gcode]`.
3. The rendered custom start G-code continues to participate in existing startup temperature suppression exactly as before.
4. The placeholder remains literal in non-machine-start scopes, including `layer_change_gcode`.
5. No user option input is read for this placeholder because the cited Orca boundary provides a runtime parser variable, not a persisted user-facing option value.

## Deferred Behavior

- Any future Orca behavior that populates `bed_temperature_initial_layer_vector` with non-empty content.
- Full Orca placeholder parser parity, brace expressions, conditionals, vector indexing, and typed placeholder metadata.
- Public option storage/export semantics for `bed_temperature_initial_layer_vector`.
- Bed-temperature formula changes beyond existing Ares bed-temperature placeholder behavior.
- UI/preset behavior, model/plate metadata behavior, movement/extrusion behavior, and temperature command generation outside the rendered custom start string.

## Acceptance Criteria

- A focused RED nextest run fails before implementation because `[bed_temperature_initial_layer_vector]` remains literal in `machine_start_gcode`.
- After implementation, the focused nextest run passes and proves the placeholder renders as an empty string.
- Tests prove the placeholder composes with existing machine-start bed-temperature placeholders without changing those outputs.
- Tests prove the placeholder remains literal in `layer_change_gcode`.
- Full verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust files remain at or below 400 LOC.

## Self-Review

- No placeholder or TODO text is left in this spec.
- Scope is intentionally limited to the source-cited Orca `GCode.cpp` runtime placeholder assignment and Ares machine-start rendering.
- The spec does not add a new Ares-owned pipeline or a broad placeholder parser rewrite.
- The behavior is externally visible in generated G-code and directly consumes an existing Orca machine-start placeholder into runtime output.
