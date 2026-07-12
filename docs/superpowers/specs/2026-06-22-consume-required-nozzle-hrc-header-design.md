# Consume Required Nozzle HRC Header Export Design

## Goal

Consume the existing OrcaSlicer `required_nozzle_HRC` option scaffold into observable Ares G-code output by exporting it in the generated G-code config header.

This is a concrete behavior slice, not a new option-metadata milestone. The Rust change must use the already-known option key and make slicer output reflect user-provided values.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1334` declares `required_nozzle_HRC` as a `ConfigOptionInts` member of `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2393-2399` defines the option with label "Required nozzle HRC", tooltip text saying zero disables nozzle HRC checking, `min = 0`, `max = 500`, and default `ConfigOptionInts{0}`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1023-1031` serializes `ConfigOptionInts` as comma-separated integer values.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends every non-banned full config key as `; key = serialized_value` in G-code output.

## Rust Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` because it already owns filament-related full-config header exports and the existing ConfigOption vector serialization helpers.
- Extend `crates/ares-core/src/gcode_header.rs` so `format_header` appends `; required_nozzle_HRC = ...` when the option is present and valid.
- Add focused async slicing tests under `crates/ares-core/src/tests/required_nozzle_hrc_gcode.rs`.
- Register the test module in `crates/ares-core/src/tests/mod.rs`.

No new crate, dependency, public API, CLI behavior, filesystem behavior, UI behavior, OpenGL behavior, or independent Ares pipeline design is allowed.

## Included Behavior

- If `required_nozzle_HRC` is present as a JSON integer array, Ares emits one config header line:
  - `[0]` becomes `; required_nozzle_HRC = 0`
  - `[0, 60, 500]` becomes `; required_nozzle_HRC = 0,60,500`
- If the option is absent, Ares does not emit a `required_nozzle_HRC` config export line.
- Values must be integers in the upstream range `0..=500`.
- Invalid shapes or values return `SliceError::InvalidInput` mentioning `required_nozzle_HRC`.
- Validation must still happen before BTT thumbnail header skipping can hide invalid input, matching the existing filament config export validation pattern.

## Deferred Behavior

- Do not implement nozzle material or hardness compatibility checks.
- Do not implement preset compatibility diagnostics, UI warnings, or printer-nozzle HRC storage.
- Do not modify `filament_map`, `filament_map_mode`, `filament_extruder_variant`, or `support_object_skip_flush`.
- Do not broaden generic option export beyond this one source-cited key.

## Acceptance Criteria

- A focused RED run of `cargo nextest run -p ares-core required_nozzle_hrc_gcode` fails before implementation because the header line and invalid-value checks are missing.
- After implementation, `cargo nextest run -p ares-core required_nozzle_hrc_gcode` passes.
- Existing adjacent filament header export tests still pass with a targeted nextest command.
- Full verification before commit uses:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, with every touched Rust file at or below 400 LOC
- Independent spec, plan, and implementation reviewers must return `VERDICT: APPROVE`.

## Risks

- `required_nozzle_HRC` uses a mixed-case upstream key. Rust field names should remain snake_case, but emitted header text must preserve the exact upstream key.
- The existing integer vector helper accepts all `i32` values. This slice needs range-aware validation for this key without changing `filament_printable` behavior.
