# M203: PrintConfig validate spiral vase CLI constraints

## Goal
Port OrcaSlicer's CLI-only spiral-vase validation slice into Ares as an explicit validation API for UI/CLI config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10207-10235`, with option-definition/default context from `PrintConfig.cpp:2881-2889`, `4918-4924`, `5678-5684`, `5903-5908`, `6013-6025`, `6564-6573`, and `PrintConfig.hpp:948`, `958`, `1101`, `1158`, `1167`, `1560`. It covers only the `cfg.spiral_mode && under_cli` checks for `wall_loops`, `sparse_infill_density`, `top_shell_layers`, `enable_support`, and `enforce_support_layers`. No extrusion-width, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI popup correction behavior, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_spiral_vase_cli_options()` returns a key-to-message map like Orca validation for the `under_cli == true` branch.
- Missing keys use source-cited registry defaults and pass because default `spiral_mode` is false.
- When `spiral_mode` is true and constrained keys are missing, source-cited registry defaults are validated and report the default `wall_loops`, `sparse_infill_density`, and `top_shell_layers` conflicts.
- When `spiral_mode` is false, conflicting values are not reported.
- When `spiral_mode` is true, `wall_loops != 1` reports key `wall_loops` with `Invalid value when spiral vase mode is enabled: {value}`.
- When `spiral_mode` is true, `sparse_infill_density > 0` reports key `sparse_infill_density` with `Invalid value when spiral vase mode is enabled: {value:.6}`.
- When `spiral_mode` is true, `top_shell_layers > 0` reports key `top_shell_layers` with `Invalid value when spiral vase mode is enabled: {value}`.
- When `spiral_mode` is true, `enable_support == true` reports key `enable_support` with `Invalid value when spiral vase mode is enabled: 1`.
- When `spiral_mode` is true, `enforce_support_layers > 0` reports key `enforce_support_layers` with `Invalid value when spiral vase mode is enabled: {value}`.
- JSON boundary type errors for malformed bool/int/float values return `SliceError::InvalidInput`; numeric strings remain accepted for numeric options to match existing Ares option-boundary behavior.
- Existing M196-M202 validation behavior remains intact.
- `PrintConfig.cpp:10237+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
