# Consume Change Filament G-code Runtime Design

## Goal

Consume the already-registered Orca `change_filament_gcode` option as typed Ares runtime state before G-code formatting returns output, without adding filament-change insertion, tool-change behavior, placeholder expansion, or manual filament-change semantics.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1392`: `GCodeConfig` declares `change_filament_gcode` as `ConfigOptionString`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6516-6523`: `change_filament_gcode` option definition, multiline UI metadata, and empty-string default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7880-7881`: legacy `tool_change_gcode` key is renamed to `change_filament_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11118-11130`: placeholder-key list for `change_filament_gcode`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7882-7894`: downstream custom filament-change G-code read and placeholder processing during tool changes.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7890`: downstream `manual_filament_change` first-tool-change omission gate.

## Current Ares Boundary

- Registry metadata for `change_filament_gcode` already exists from the M130 registry slice with kind `String` and default `""`.
- The former M2224 source-line-only slice was removed by the Option pinning cleanup; runtime behavior is owned by this source-cited slice.
- Legacy normalization already renames `tool_change_gcode` to `change_filament_gcode`.
- `crates/ares-core/src/options/custom_gcode.rs` owns adjacent custom G-code accessors, but it does not expose `change_filament_gcode()`.
- `crates/ares-core/src/gcode_runtime_options.rs` currently consumes preheat and timelapse options before G-code bytes are returned; it does not consume `change_filament_gcode`.
- Ares does not currently model tool changes that would trigger Orca's `change_filament_gcode` insertion path.

## Design

Add `SliceOptions::change_filament_gcode() -> Result<&str, SliceError>` in `crates/ares-core/src/options/custom_gcode.rs`, following the existing `change_extrusion_role_gcode()` string-accessor style:

- omitted `change_filament_gcode` returns the Orca-compatible empty default.
- string values return the configured string.
- non-string values return `SliceError::InvalidInput` with the option key in the message.

Consume the accessor from `crates/ares-core/src/gcode_runtime_options.rs`:

- call `options.change_filament_gcode()?;` after the existing preheat and timelapse runtime consumption.
- do not store the returned value because this slice only validates and marks the registered option as consumed by the current runtime boundary.
- do not alter `crates/ares-core/src/gcode.rs`; it already delegates runtime option consumption through `gcode_runtime_options`.

Add focused option tests in a new `crates/ares-core/src/options/tests/change_filament_gcode.rs` module and register it through the existing compact `option_test_modules!(...)` line so `crates/ares-core/src/options/tests.rs` stays at or below 400 LOC.

Add focused G-code tests in a new `crates/ares-core/src/tests/change_filament_gcode.rs` module and register it through the existing test-module list. The tests must prove invalid values fail through the formatting path and valid values preserve current output because insertion is deferred.

## Behavior Included

- `change_filament_gcode` is now a typed crate-private runtime read.
- Invalid non-string `change_filament_gcode` values are rejected before G-code bytes are returned.
- Legacy `tool_change_gcode` remains renamed to `change_filament_gcode`; after deserialization it is validated by the same accessor.
- Omitted, empty, and non-empty string values preserve current generated G-code output because Ares has no tool-change insertion path in this slice.

## Behavior Deferred

- Filament-change G-code insertion.
- Tool-change state, tool-change count, next/previous extruder state, and travel-point placeholders.
- `manual_filament_change` first-tool-change omission behavior.
- `single_extruder_multi_material` behavior.
- Full Orca placeholder expression and conditional evaluation.
- Runtime behavior for `single_extruder_multi_material_priming`.
- UI, CLI, WASM binding changes.
- Orca binary E2E filament-change parity.

## Acceptance Criteria

- Option tests prove absent, empty string, non-empty string, legacy-renamed string, and invalid non-string values.
- G-code tests prove invalid values reject before output and valid string values preserve command output.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, and `crates/ares-core/src/gcode.rs` remain at or below 400 LOC after `cargo fmt`.
- Verification passes with:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run -p ares-core change_filament_gcode`
  - `cargo nextest run --workspace`
