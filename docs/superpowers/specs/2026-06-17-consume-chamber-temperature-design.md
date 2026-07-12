# Consume Chamber Temperature Options Design

## Goal

Implement the smallest source-cited chamber-temperature runtime slice by consuming the existing `activate_chamber_temp_control` and `chamber_temperature` options in generated G-code.

## Source Boundary

This slice ports the startup and shutdown chamber-temperature behavior from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1636-1637`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448-6476`
- `OrcaSlicer/src/libslic3r/GCode.cpp:2973-2977`
- `OrcaSlicer/src/libslic3r/GCode.cpp:3093-3098`
- `OrcaSlicer/src/libslic3r/GCode.cpp:3451-3452`
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:192-214`

Existing Ares registry metadata and legacy alias handling for `activate_chamber_temp_control`, `chamber_temperature`, and `chamber_temperatures` are reused. No new option metadata is added.

## Ares Destination Boundary

The Rust destination boundary is limited to:

- `crates/ares-core/src/options/chamber_temperature.rs`: new runtime option accessor and bool-vector parsing for chamber control.
- `crates/ares-core/src/options.rs`: module registration and crate-private export only.
- `crates/ares-core/src/gcode_writer.rs`: chamber command formatter matching Orca's `M191`/`M141` spelling and spacing.
- `crates/ares-core/src/gcode_startup.rs`: startup command helper gated by flavor and effective chamber control.
- `crates/ares-core/src/gcode.rs`: read chamber options, emit startup command before first layer, and emit shutdown command before `M2`.
- `crates/ares-core/src/options/tests/chamber_temperature_runtime.rs`: runtime option parsing tests.
- `crates/ares-core/src/gcode_writer/tests.rs`: chamber command formatting tests.
- `crates/ares-core/src/tests/chamber_temperature_gcode.rs`: slice-level G-code behavior tests.
- `crates/ares-core/src/tests/mod.rs` and `crates/ares-core/src/options/tests.rs`: test module registration.

No other crates, dependencies, registry definitions, profile metadata, UI code, or pipeline architecture are in scope. If a touched `crates/ares-core/src/*.rs` file would exceed 400 LOC, the implementation must split or compress declarations without changing behavior.

## Runtime Behavior

Ares must parse:

- `activate_chamber_temp_control`: bool, lowercase bool string (`"true"`/`"false"`), bool array, or semicolon/comma separated lowercase bool string. Leading/trailing whitespace around string tokens is ignored. Empty strings, empty separators such as `"true;"`, empty arrays, mixed arrays, non-bool array items, numeric values, objects, null, and differently cased bool strings such as `"True"` are invalid. Missing defaults to `[false]`.
- `chamber_temperature`: non-negative integer, integer string, integer array, or semicolon/comma separated integer string. Leading/trailing whitespace around string tokens is ignored. Empty strings, empty separators such as `"45;"`, empty arrays, floats, negative values, mixed arrays, non-numeric array items, objects, booleans, and null are invalid. Missing defaults to `[0]`.

The legacy alias `chamber_temperatures` is already normalized to `chamber_temperature` by Ares legacy handling. This slice must include at least one runtime test proving that a profile using `chamber_temperatures` can drive the new chamber-temperature behavior after normal option deserialization.

For Ares' current single-tool output, the effective chamber control is enabled when any `activate_chamber_temp_control` entry is true. The effective chamber temperature is the maximum value in `chamber_temperature`, matching Orca's max-over-extruders loop. If control is disabled or the maximum chamber temperature is zero, no chamber-temperature G-code is emitted.

When effective control is enabled and the maximum chamber temperature is positive:

- Emit startup chamber command before `;LAYER_CHANGE`:
  `M191 S{temperature} ;set chamber_temperature and wait for it to be reached`
- Emit shutdown chamber command before final `M2`:
  `M141 S0;set chamber_temperature`

The startup command must appear after the current first-layer bed/nozzle startup commands in Ares because Ares does not yet emit `machine_start_gcode`; this keeps the slice local to the existing startup section while preserving the Orca command semantics. Future `machine_start_gcode` work may move the chamber startup command to the Orca-exact position before user start G-code.

`gcode_flavor = "klipper"` skips the chamber startup and shutdown commands for this slice, matching the existing Ares temperature-startup convention until user start macros exist.

## Deferred Behavior

This slice does not implement:

- `machine_start_gcode` placeholder parsing or custom start-G-code temperature detection.
- Auxiliary fan `M106 P2` wrapping from `GCodeWriter::set_chamber_temperature`.
- Chamber placeholder variables such as `overall_chamber_temperature`.
- Multi-extruder tool iteration beyond vector any/max semantics.
- Flat-print chamber hold heuristics.
- UI, printer capability gating, or device chamber controls.

## Docs Impact

No user-facing documentation, architecture decision record, roadmap entry, or example update is required for this runtime slice. The new SDD spec and plan are the traceable design artifacts, and the public API surface remains unchanged.

## Acceptance Criteria

- Default slicing output remains unchanged for chamber control because `activate_chamber_temp_control` defaults to false.
- Enabling chamber control with `chamber_temperature = 45` emits `M191 S45 ;set chamber_temperature and wait for it to be reached` before the first layer and `M141 S0;set chamber_temperature` before `M2`.
- Multiple chamber temperatures use the maximum positive value.
- Multiple activation values enable output if any value is true.
- Zero chamber temperature emits no chamber commands even when activation is true.
- Klipper flavor emits no chamber commands.
- Invalid activation or chamber temperature values return `SliceError::InvalidInput` mentioning the offending option key.
- All changed `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification Criteria

The implementation is not complete until all of the following pass with fresh output:

- Targeted option parser tests for chamber defaults, accepted forms, max/any semantics, legacy alias normalization, and invalid inputs.
- Targeted writer tests for `M191` wait formatting and `M141` shutdown formatting.
- Targeted slice-level tests for default unchanged output, enabled startup/shutdown output, max temperature selection, any activation selection, zero suppression, Klipper suppression, and invalid input propagation.
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- A `crates/ares-core/src/*.rs` LOC check proving every core source file is at or below 400 lines.
