# Consume First-Layer Nozzle Temperature Design

## Goal

Consume OrcaSlicer `nozzle_temperature_initial_layer` as concrete Ares startup G-code behavior. This slice must turn an already registered option into emitted nozzle-temperature commands and must not add more option metadata.

## Upstream Boundary

Line citations are pinned to the checked-out `OrcaSlicer` revision `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1533` declares `ConfigOptionInts nozzle_temperature_initial_layer`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316-3323` registers `nozzle_temperature_initial_layer` with default `200`, minimum `0`, and maximum `max_temp`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3084-3091` writes first-layer bed and extruder temperatures before custom start G-code for non-Klipper flavors.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4000-4032` emits first-layer extruder temperature commands from `nozzle_temperature_initial_layer`, skipping non-positive temperatures.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:117-154` formats nozzle-temperature commands by flavor: `M104` for normal non-wait commands, `G10` for RepRapFirmware, and `M109`/`M116` wait handling where applicable.

## Current Ares State

- Ares registry already exposes `nozzle_temperature_initial_layer` as `Ints` with default `200`.
- Ares validation/lookup tests already cover the upstream metadata.
- Ares G-code output currently emits no nozzle-temperature command; the option is not consumed by slicing behavior.
- `crates/ares-core/src/gcode.rs` is near the 400 LOC limit, so this slice cannot keep adding logic to that file directly.

## Ares Destination Boundary

- Add a focused runtime option accessor for first-layer nozzle temperature under `crates/ares-core/src/options/`.
- Extend `GCodeWriter` with a source-cited nozzle-temperature formatter matching the relevant `GCodeWriter.cpp` flavor behavior.
- Move Ares header/preamble construction out of `crates/ares-core/src/gcode.rs` into a new focused module so the temperature command can be inserted without pushing `gcode.rs` over the 400 LOC rule.
- Wire `format_gcode` to emit the startup nozzle-temperature command after the existing writer preamble and before the first layer.

## Included Behavior

1. Missing `nozzle_temperature_initial_layer` defaults to `200`.
2. Scalar integer number, integer string, semicolon/comma integer string list, and non-empty integer arrays are accepted. Fractional values such as `200.5`, `"200.5"`, and `[200.5]` are rejected because upstream uses `ConfigOptionInts` and `GCodeWriter::set_temperature` takes an integer temperature.
3. Ares uses the first temperature value only in this slice because current slicing output is single-tool and has no tool ordering or multi-extruder temperature scheduling.
4. `0` disables first-layer nozzle-temperature emission.
5. Negative, fractional, non-finite, non-numeric, and empty values are rejected with `SliceError::InvalidInput`.
6. Default Marlin-like output emits `M104 S200 ; set nozzle temperature` before the first `;LAYER_CHANGE`.
7. `gcode_flavor: "reprapfirmware"` emits `G10 S200 ; set nozzle temperature`.
8. `gcode_flavor: "klipper"` skips this startup temperature branch, matching `GCode.cpp:3084-3091`.
9. The command composes with `gcode_comments`; temperature comments come from Orca's command formatter and are not controlled by Ares inline move comments.

## Deferred Behavior

- Bed temperature, chamber temperature, other-layer nozzle temperature changes, standby/idle temperature, ooze prevention, custom start G-code temperature detection, multi-extruder tool parameters, wait-mode startup temperature, toolchange temperature, temperature-range validation, and start-G-code placeholder variables are deferred.
- This slice does not enable hidden G-code flavors through public `SliceOptions`.
- This slice does not add new registry entries, roadmap milestones, crates, dependencies, UI behavior, filesystem behavior, or independent Ares pipeline design.

## Docs Impact

This spec and its implementation plan document the slice. No roadmap update is required because this continues the current option-consumption milestone and does not change milestone ordering.

## Acceptance Criteria

- Option tests prove default, scalar integer, integer string, list string, array, zero, negative, fractional number, fractional string, empty, and invalid values for `nozzle_temperature_initial_layer`.
- Writer tests prove Marlin-like non-wait temperature uses `M104 S...`, RepRapFirmware non-wait temperature uses `G10 S...`, and wait-mode flavor differences remain source-cited at the writer boundary.
- Integration tests prove default slicing emits `M104 S200 ; set nozzle temperature` before the first layer, `0` suppresses it, `reprapfirmware` emits `G10 S200 ; set nozzle temperature`, and `klipper` emits no startup nozzle-temperature command.
- Existing `gcode_flavor`, relative-E, speed, acceleration, jerk, skirt, brim, and z-offset behavior remains intact.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass.

## Safety

The runtime surface remains limited to one existing active temperature option and existing active public G-code flavors. Valid `0` temperatures suppress heater commands, matching Orca's first-layer extruder-temperature branch; negative temperatures are invalid input and do not reach emission.
