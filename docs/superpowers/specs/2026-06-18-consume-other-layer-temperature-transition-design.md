# Consume Other-Layer Temperature Transition Design

## Goal

Consume the existing OrcaSlicer `nozzle_temperature` and selected build-plate `*_temp` options by emitting the first-to-second-layer temperature transition in Ares G-code. This is a concrete single-tool slicing behavior slice, not a new option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1568` declares `ConfigOptionInts nozzle_temperature`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478-6485` defines `nozzle_temperature` as "Other layers" nozzle temperature.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3948-3952` selects first-layer or other-layer bed temperature using `get_bed_temp_1st_layer_key` / `get_bed_temp_key`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4669-4686` performs the transition after the first layer: it emits non-wait nozzle temperature when `nozzle_temperature` is positive and differs from `nozzle_temperature_initial_layer`, then emits non-wait bed temperature for the other-layer selected bed temperature.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:168-187` formats non-wait bed temperature as `M140 S... ; set bed temperature`.

## Current Ares State

- Ares already emits first-layer startup bed temperature through `SliceOptions::first_layer_bed_temperature()` and `GCodeWriter::set_bed_temperature(..., true)`.
- Ares already emits first-layer startup nozzle temperature through `SliceOptions::first_layer_nozzle_temperature()` and `GCodeWriter::set_nozzle_temperature(..., false, None)`.
- Ares has `temperature_vector::parse_integer_vector()` for integer-vector temperature options.
- Ares does not currently parse `nozzle_temperature` at runtime.
- Ares does not currently parse other-layer bed temperature keys selected by `curr_bed_type`.
- Ares does not currently emit the first-to-second-layer temperature transition in the per-layer G-code loop.

## Design

Add a small temperature-transition module and focused option accessors:

- Add `SliceOptions::other_layer_nozzle_temperature()` to `crates/ares-core/src/options/nozzle_temperature.rs`.
  - Reads `nozzle_temperature`.
  - Defaults to `200`, matching Orca's option definition default.
  - Uses the first parsed integer because Ares currently emits single-tool layer-ordered G-code.
  - Accepts the same integer-vector forms as `nozzle_temperature_initial_layer`.
- Add `SliceOptions::other_layer_bed_temperature()` to `crates/ares-core/src/options/bed_temperature.rs`.
  - Uses `curr_bed_type` to select the non-initial bed temperature key:
    - `Cool Plate` -> `cool_plate_temp`
    - `Textured Cool Plate` -> `textured_cool_plate_temp`
    - `Engineering Plate` -> `eng_plate_temp`
    - `High Temp Plate` -> `hot_plate_temp`
    - `Textured PEI Plate` -> `textured_plate_temp`
    - `Supertack Plate` and `SuperTack Plate` -> `supertack_plate_temp`
  - Defaults to the parsed current first-layer selected bed temperature when the non-initial key is absent. This includes explicit first-layer bed-temperature overrides and keeps existing output from adding a redundant `M140` at layer two because `GCodeWriter` in Ares does not yet track previous bed temperature state.
  - Uses the first parsed integer because Ares is single-tool for this slice.
- Add `crates/ares-core/src/gcode_temperature_transition.rs`.
  - Expose one function that receives `GCodeWriter`, `GCodeFlavor`, `SliceOptions`, and the current 1-based layer number.
  - Return no command unless `layer_num == 2`.
  - For Klipper, return no command, matching existing Ares startup-temperature behavior.
  - Emit non-wait nozzle temperature before non-wait bed temperature.
  - Emit nozzle temperature only when `other_layer_nozzle_temperature > 0` and differs from `first_layer_nozzle_temperature`.
  - Emit bed temperature only when `other_layer_bed_temperature` differs from `first_layer_bed_temperature`.
  - Keep `0` as a valid bed target and emit `M140 S0 ; set bed temperature` when transitioning from a nonzero first-layer bed temperature to zero.
- Call the transition function in `format_gcode()` after `;LAYER_CHANGE` / `;LAYER` / `;Z` comments for layer two and before custom before-layer-change G-code and Z travel.

## Deferred Behavior

- Multi-extruder / multi-filament temperature selection, highest bed temperature across used filaments, `bed_temperature_formula`, `single_extruder_multi_material`, ooze-prevention temperature handling, tool-specific `T`/`P` arguments, idle/standby temperature, preheat commands, custom start-G-code temperature detection, GCodeWriter bed/nozzle temperature state suppression, calibration-specific interpolation, and post-processing temperature rewrites.
- Any UI behavior, filesystem behavior, new dependencies, new crates, or independently designed Ares pipeline behavior.

## Docs Impact

No user-facing documentation update is required. The observable behavior is that existing `nozzle_temperature` and selected non-initial bed-temperature options now affect generated G-code at the layer-two transition.

## Acceptance Criteria

- `nozzle_temperature` differing from `nozzle_temperature_initial_layer` emits exactly one non-wait nozzle command at the second layer transition before layer-two Z travel.
- `nozzle_temperature = 0` suppresses the other-layer nozzle command.
- `nozzle_temperature` equal to `nozzle_temperature_initial_layer` suppresses the other-layer nozzle command.
- Selected non-initial bed temperature differing from the selected first-layer bed temperature emits exactly one `M140 S... ; set bed temperature` at the second layer transition.
- Selected non-initial bed temperature `0` emits `M140 S0 ; set bed temperature` when the first-layer bed temperature is nonzero.
- Missing non-initial bed temperature does not add a redundant layer-two bed command, including when the selected first-layer bed temperature was explicitly overridden.
- `gcode_flavor = "klipper"` emits no second-layer nozzle or bed temperature transition commands.
- Invalid `nozzle_temperature` and selected non-initial bed temperature values return `SliceError::InvalidInput` naming the offending key.
- Existing first-layer nozzle and bed temperature tests still pass.
- No touched Rust file exceeds 400 LOC.

## Verification

- `cargo test -p ares-core --lib other_layer_temperature_gcode`
- `cargo test -p ares-core --lib nozzle_temperature_gcode`
- `cargo test -p ares-core --lib bed_temperature_gcode`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
