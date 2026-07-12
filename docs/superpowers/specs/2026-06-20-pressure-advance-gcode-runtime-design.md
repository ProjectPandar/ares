# Pressure Advance G-code Runtime Design

## Goal

Consume the already-registered `enable_pressure_advance` and `pressure_advance` options as concrete startup G-code behavior. When the first filament enables pressure advance, Ares should emit the Orca-compatible pressure advance command for the active G-code flavor instead of leaving the options as registry/staged metadata only.

## Upstream Boundary

Line numbers are from the vendored `OrcaSlicer/` tree in this repository.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1302-1303` declares `GCodeConfig` tuple entries for `enable_pressure_advance` and `pressure_advance`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252-2262` registers `enable_pressure_advance` as `coBools` with default `false` and `pressure_advance` as `coFloats` with default `0.02`, max `2`, and advanced mode.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:370-389` implements `GCodeWriter::set_pressure_advance(double pa)`:
  - returns no command for negative values,
  - emits `SET_PRESSURE_ADVANCE ADVANCE=<pa>; Override pressure advance value` for Klipper,
  - emits `M572 D0 S<pa>; Override pressure advance value` for RepRapFirmware,
  - emits `M233 X<pa> Y<pa> ; Override pressure advance value` for Repetier,
  - emits `M900 K<pa>; Override pressure advance value` for other supported non-BBL flavors.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3118-3124` emits the initial filament pressure advance command after filament-start handling when `m_config.enable_pressure_advance.get_at(initial_non_support_extruder_id)` is true.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1369-1373` repeats the same command on tool change for a new extruder.

## Current Ares Gap

`enable_pressure_advance` and `pressure_advance` exist in registry definitions and source-cited `PrintConfig.hpp` metadata, and they are already normalized by the existing printer/extruder update helpers. `rg -n "set_pressure_advance|pressure_advance" crates/ares-core/src` shows no runtime writer method, no typed option accessor, and no G-code emission path outside metadata and update tests. Generated slices currently ignore both options.

## Ares Destination Boundary

- `crates/ares-core/src/options/pressure_advance.rs`: parse the first filament's `enable_pressure_advance` and `pressure_advance` values into a small runtime control object.
- `crates/ares-core/src/options.rs`: include the new options module.
- `crates/ares-core/src/gcode_writer.rs`: add a source-aligned `set_pressure_advance` formatter on `GCodeWriter`.
- `crates/ares-core/src/gcode_pressure_advance.rs`: emit the startup pressure advance command using the active `GCodeFlavor` and parsed pressure advance control.
- `crates/ares-core/src/gcode.rs`: call the startup helper after custom machine/filament start G-code and before object labels / first M73 progress.
- `crates/ares-core/src/lib.rs`: register the new G-code helper module.
- `crates/ares-core/src/tests/pressure_advance_gcode.rs`: add end-to-end slice tests for emitted commands, disabled defaults, flavor-specific formatting, and invalid inputs.
- `crates/ares-core/src/tests/mod.rs`: register the new test module.
- `docs/roadmap.md`: add a live roadmap entry describing the consumed runtime slice and deferred behavior.

## Included Behavior

- Missing `enable_pressure_advance` preserves current output and emits no pressure advance command.
- `enable_pressure_advance=false` emits no pressure advance command even when `pressure_advance` is present.
- `enable_pressure_advance=true` emits one startup pressure advance command using the first `pressure_advance` value.
- Missing `pressure_advance` under enabled pressure advance uses Orca's registered default `0.02`.
- Scalar booleans and one-or-more-element boolean arrays are accepted for `enable_pressure_advance`; arrays use the first element for the current single-initial-filament boundary.
- Scalar numbers, numeric strings, comma/semicolon numeric strings, and one-or-more-element numeric arrays are accepted for `pressure_advance`; arrays use the first element for the current single-initial-filament boundary.
- `pressure_advance` accepts finite values from `0.0` through `2.0` inclusive, matching Orca's configured max and allowing the writer's non-negative command path.
- For `gcode_flavor`:
  - default Marlin legacy emits `M900 K<pa>; Override pressure advance value`,
  - Klipper emits `SET_PRESSURE_ADVANCE ADVANCE=<pa>; Override pressure advance value`,
  - RepRapFirmware emits `M572 D0 S<pa>; Override pressure advance value`,
  - Repetier emits `M233 X<pa> Y<pa> ; Override pressure advance value`,
  - MarlinFirmware emits `M900 K<pa>; Override pressure advance value`.
- Pressure advance values use the existing Ares axis formatter behavior for concise decimal output, capped at four fractional digits for this command.
- The command is emitted after `machine_start_gcode` and `filament_start_gcode`, matching the initial-filament placement in `GCode.cpp:3118-3124` as closely as Ares' current startup structure allows.

## Deferred Behavior

- Tool-change pressure advance emission from `GCode.cpp:1369-1373`, because Ares currently has no multi-tool-change G-code path.
- Adaptive pressure advance, adaptive PA model parsing, bridge/overhang adaptive PA, and PA post-processing.
- BBL-specific `M900 K... L1000 M10` formatting, because Ares does not currently model BBL printer detection separately from active `GCodeFlavor`.
- Multiple filament/extruder runtime selection beyond the existing first-filament startup path.
- Pressure advance calibration modes, wipe tower integration, nozzle change handling, and PA reset processor state.
- UI, preset editing, profile serialization, new public APIs, new dependencies, new crates, filesystem behavior, or independent Ares pipeline design.

## Acceptance Criteria

- E2E tests prove default/missing pressure advance options preserve output with no pressure advance command.
- E2E tests prove `enable_pressure_advance=false` suppresses pressure advance even when `pressure_advance` is configured.
- E2E tests prove default Marlin legacy output emits exactly one startup `M900 K...` command when pressure advance is enabled.
- E2E tests prove Klipper, RepRapFirmware, Repetier, and MarlinFirmware flavor-specific pressure advance command formatting.
- E2E tests prove the pressure advance command appears after custom `machine_start_gcode` and `filament_start_gcode` markers.
- E2E tests prove invalid `enable_pressure_advance` and out-of-range or malformed `pressure_advance` values return `SliceError::InvalidInput` with the relevant option key.
- The implementation keeps touched `crates/**/src/**/*.rs` files at or below 400 LOC.
- Focused verification passes with `cargo nextest run -p ares-core pressure_advance`.
- Final verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC guard.

## Docs Impact

Update `docs/roadmap.md` with a short runtime slice entry stating that startup pressure advance G-code is now consumed for the first filament and that adaptive PA, BBL-specific formatting, and tool-change PA remain deferred.

## Safety And Simplicity

This is a narrow G-code runtime slice. It reuses `SliceOptions`, existing numeric-vector parsing, current `GCodeFlavor`, and `GCodeWriter`. It should not add dependencies, mutate stored JSON values, introduce a tool-change subsystem, or implement unrelated pressure-advance/adaptive-PA behavior.
