# Use Firmware Retraction For Layer Changes Design

## Goal

Consume the existing OrcaSlicer `use_firmware_retraction` option in concrete Ares G-code output by making layer-change retractions emit firmware retract/unretract commands instead of E-axis retract/unretract moves when the option is enabled.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp`: `use_firmware_retraction` is a `coBool` option with default `false` and tooltip text describing `G10`/`G11` firmware retraction.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp`: `GCodeWriter::_retract` forces a fake retract length when firmware retraction is enabled, then emits `G10 ; retract` or `G22 ; retract` for Machinekit. `GCodeWriter::unretract` emits `G11 ; unretract` or `G23 ; unretract` for Machinekit.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp`: `process_G10` and `process_G11` emulate the commands as `retraction_length` and `retraction_length + retract_restart_extra`, confirming that the firmware commands stand in for the ordinary configured retract/unretract pair.

## Ares Destination Boundary

- Extend `crates/ares-core/src/options/layer_change_retraction.rs` so `LayerChangeRetraction` also records whether layer-change retraction should use firmware commands.
- Extend `crates/ares-core/src/gcode_writer/retraction.rs` with flavor-aware firmware retract/unretract command helpers.
- Update `crates/ares-core/src/gcode.rs` only at the existing layer-change retract/unretract emission points.
- Add `crates/ares-core/src/gcode_writer/tests/retraction.rs` and register `mod retraction;` from `crates/ares-core/src/gcode_writer/tests/mod.rs` so Machinekit helper mapping tests do not grow the already-large writer test root module.
- Add `crates/ares-core/src/tests/layer_change_retraction_gcode/firmware.rs` and register `mod firmware;` from `crates/ares-core/src/tests/layer_change_retraction_gcode.rs` so firmware runtime tests do not push the existing file over the 400 LOC limit.

## Included Behavior

- `use_firmware_retraction` defaults to `false`, preserving current E-axis retraction output.
- When `use_firmware_retraction` is `true` and layer-change retraction is otherwise enabled, the second and later layer changes emit firmware commands around the Z travel:
  - active non-Machinekit runtime flavors emit `G10 ; retract` before the layer Z move and `G11 ; unretract` before the first print move;
  - the writer helper maps Machinekit to `G22 ; retract` and `G23 ; unretract`, covered by a unit test because `SliceOptions::gcode_flavor()` does not yet expose Machinekit as an active runtime flavor.
- Firmware retraction suppresses the layer-change `G1 E... ; retract` and `G1 E... ; unretract` commands because the firmware handles the configured retract distance.
- The existing `retract_when_changing_layer` and `retraction_length == 0` gates remain in force: firmware retraction does not create a retraction if layer-change retraction is disabled.
- `use_firmware_retraction` accepts only a JSON boolean in this slice. Invalid types return `SliceError::InvalidInput` naming `use_firmware_retraction`.
- Absolute and relative E print move state remains coherent after firmware unretract. Firmware commands do not mutate Ares writer E state because there is no emitted E-axis movement to account for in the output stream.

## Deferred Behavior

- Ordinary travel retraction, wipe, z-hop, lift, toolchange retraction, minimum travel filtering, and multi-extruder-specific firmware retraction are deferred.
- Activating `machinekit`, `smoothie`, or other currently inactive `GCodeFlavor` values through `SliceOptions::gcode_flavor()` is deferred to a flavor-activation slice.
- Orca's post-firmware-unretract `reset_e()` emission is deferred because Ares currently models `reset_e` as internal writer state plus explicit preamble reset only; adding runtime `G92 E0` outside the existing writer contract would be a separate G-code state slice.
- Full firmware-specific validation remains in the existing `validate_firmware_retraction_options` path; this slice only parses the boolean for layer-change emission.

## Docs Impact

No roadmap or architecture document update is required for this narrow runtime slice. The source-cited design file, implementation plan, and behavior tests are the durable documentation for the option consumption.

## Acceptance Criteria

- A focused RED run of `cargo nextest run -p ares-core layer_change_retraction_gcode` fails before implementation because firmware retraction still emits E-axis layer-change retract/unretract commands or rejects the new runtime expectation.
- After implementation, the focused command passes.
- Related validation tests pass with `cargo nextest run -p ares-core layer_change_retraction_gcode validation::firmware_retraction relative_e_gcode`.
- Writer unit tests cover the Machinekit `G22`/`G23` mapping without requiring `SliceOptions::gcode_flavor()` to accept `machinekit`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC check.
- No new crates or dependencies are added.
- Touched Rust files remain at or below 400 LOC.
