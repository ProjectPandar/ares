# Consume First-Layer Bed Temperature Design

## Goal

Consume OrcaSlicer first-layer bed temperature options as concrete Ares startup G-code behavior. This slice must turn already registered bed-temperature options into emitted bed-temperature commands and must not add more option metadata.

## Upstream Boundary

Line citations are pinned to the checked-out `OrcaSlicer` revision `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:489-509` maps `BedType` values to first-layer bed temperature keys such as `cool_plate_temp_initial_layer`, `eng_plate_temp_initial_layer`, `hot_plate_temp_initial_layer`, and `textured_plate_temp_initial_layer`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489-1501` declares `curr_bed_type` and the bed temperature option vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1038` registers first-layer bed temperature defaults: SuperTack Plate `35`, Cool Plate `35`, Textured Cool Plate `40`, Engineering Plate `45`, High Temp Plate `45`, and Textured PEI Plate `45`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7868-7872` normalizes legacy `curr_bed_type` value `SuperTack Plate` to `Supertack Plate`; Ares already mirrors this deserialization behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2504-2512` registers `bed_temperature_formula` with default `by_highest_temp`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3084-3088` writes first-layer bed temperature before first-layer extruder temperature for non-Klipper flavors.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3948-3952` resolves first-layer bed temperature through `get_bed_temp_1st_layer_key(curr_bed_type)` and returns the vector entry for the selected filament/extruder.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3969-3995` emits first-layer bed temperature through `GCodeWriter::set_bed_temperature`, with custom start-G-code detection adjacent but deferred in this slice.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:168-187` formats bed temperature commands as `M140 S...` for non-wait and `M190 S...` for wait. The cited startup call uses wait mode.

## Current Ares State

- Ares already registers `curr_bed_type`, `bed_temperature_formula`, and bed temperature option keys as metadata.
- Ares currently emits no bed-temperature startup command.
- Ares now emits first-layer nozzle temperature through `gcode_startup`, after the writer preamble and before the first layer.
- `crates/ares-core/src/options.rs` is near the 400 LOC limit, so this slice cannot grow that file directly without a split.

## Ares Destination Boundary

- Add a focused runtime accessor for first-layer bed temperature under `crates/ares-core/src/options/`.
- Reuse the same integer vector parsing rules established for `nozzle_temperature_initial_layer`, while moving shared integer-vector parsing into a small options helper instead of duplicating it.
- Extend `GCodeWriter` with source-cited bed-temperature formatting.
- Extend `gcode_startup` so `format_gcode` emits bed temperature before nozzle temperature, matching Orca's non-Klipper startup order.

## Included Behavior

1. Missing `curr_bed_type` defaults to Orca's registry default `Cool Plate`.
2. Missing `bed_temperature_formula` defaults to Orca's registry default `by_highest_temp`. In current single-tool Ares slicing this is equivalent to the first value of the selected bed-type first-layer temperature vector.
3. `curr_bed_type` selects the first-layer temperature key:
   - `Cool Plate` -> `cool_plate_temp_initial_layer`, default `35`
   - `Textured Cool Plate` -> `textured_cool_plate_temp_initial_layer`, default `40`
   - `Engineering Plate` -> `eng_plate_temp_initial_layer`, default `45`
   - `High Temp Plate` -> `hot_plate_temp_initial_layer`, default `45`
   - `Textured PEI Plate` -> `textured_plate_temp_initial_layer`, default `45`
   - `SuperTack Plate` and `Supertack Plate` -> `supertack_plate_temp_initial_layer`, default `35`
4. Any other `curr_bed_type` string is rejected with `SliceError::InvalidInput` naming `curr_bed_type`.
5. Scalar integer number, integer string, semicolon/comma integer string list, and non-empty integer arrays are accepted for the selected bed temperature key.
6. Fractional, negative, non-finite, non-numeric, and empty values are rejected with `SliceError::InvalidInput` naming the selected key.
7. Ares uses the first bed temperature value only in this slice because current output is single-tool and has no multi-filament highest-temperature scheduling.
8. `0` emits `M190 S0 ; set bed temperature and wait for it to be reached`, matching `GCodeWriter::set_bed_temperature`; unlike the first-layer nozzle slice, this slice does not suppress zero.
9. Default Marlin-like output emits `M190 S35 ; set bed temperature and wait for it to be reached` before `M104 S200 ; set nozzle temperature` and before the first `;LAYER_CHANGE`.
10. `gcode_flavor: "klipper"` skips this startup bed-temperature branch, matching `GCode.cpp:3084-3088`.
11. The bed command comment comes from Orca's command formatter and is not controlled by Ares `gcode_comments`.

## Deferred Behavior

- Custom start-G-code temperature detection, non-wait bed temperature use outside startup (`M140`), multi-filament highest-temperature scheduling, other-layer bed temperature changes, bed temperature formula beyond the single-tool equivalent, placeholder variables, plate-specific compatibility checks, chamber temperature, fan behavior, bed texture/model behavior, and non-active hidden G-code flavors are deferred.
- This slice does not add new registry entries, roadmap milestones, crates, dependencies, UI behavior, filesystem behavior, or independent Ares pipeline design.

## Docs Impact

This spec and its implementation plan document the slice. No roadmap update is required because this continues the current option-consumption milestone and does not change milestone ordering.

## Acceptance Criteria

- Option tests prove default Cool Plate temperature, each active bed type mapping, accepted integer forms, zero, invalid selected-key values, and invalid `curr_bed_type`.
- Writer tests prove non-wait bed temperature uses `M140 S...` and wait-mode bed temperature uses `M190 S...`.
- Integration tests prove default slicing emits `M190 S35 ; set bed temperature and wait for it to be reached` before `M104 S200 ; set nozzle temperature`, selected bed type changes the emitted value, explicit selected key values override the default, zero emits `M190 S0 ; set bed temperature and wait for it to be reached`, and Klipper emits no startup bed-temperature command.
- Existing first-layer nozzle temperature, `gcode_flavor`, relative-E, speed, acceleration, jerk, skirt, brim, and z-offset behavior remains intact.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the `crates/ares-core/src` 400 LOC gate pass.

## Safety

The runtime surface remains limited to existing active bed temperature options and active public G-code flavors. Invalid user-provided bed temperature values fail before G-code emission. This slice emits only Orca's startup wait command and does not attempt to infer or edit custom start G-code.
