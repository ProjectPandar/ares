# Consume printable_height Validation Design

## Source Boundary

This slice ports the FDM `printable_height` height-limit behavior from OrcaSlicer into Ares:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:757-764` defines `printable_height` as a non-negative float with default `100.0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1539` owns the FDM `PrintConfig` tuple entry.
- `OrcaSlicer/src/libslic3r/Print.cpp:1376-1393` rejects an object when generated object layers exceed `config().printable_height`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2168-2169` passes `printable_height` into G-code validity checks.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1797-1889` tracks extruded path `max_print_z` and flags paths over `plate_printable_height`.

## Current Ares State

Ares already has registry metadata for `printable_height` with default `100` and uses it only for the machine-start `[max_print_height]` placeholder in `crates/ares-core/src/gcode_machine_start_placeholders.rs`. The option does not currently stop slicing when planned print Z exceeds the configured machine height.

## Goal

Consume `printable_height` into concrete core slicing behavior by rejecting a slicing pipeline whose planned layer `print_z` exceeds the configured printable height. The rejection must be observable through both `run_slicing_pipeline` and public `slice`, before any G-code is returned.

## Design

Add a small core module, `crates/ares-core/src/printable_height.rs`, that owns this behavior:

- Parse `printable_height` from `SliceOptions::values()` as a JSON number or numeric string.
- If absent, read the default from `crate::options::registry::option_definition("printable_height")`.
- Reject non-numeric, non-finite, or negative values with `SliceError::InvalidInput` mentioning `printable_height`.
- Validate planned layers by taking the maximum `Layer::print_z()` and rejecting when it is greater than `printable_height + 1e-6`.

Call the validator in `run_slicing_pipeline` immediately after `plan_layers(&model, &options)?` and before segment slicing or G-code generation. This uses the same effective height surface as Orca's generated-layer check while remaining inside Ares' current layer-planning boundary.

The existing `[max_print_height]` placeholder behavior remains unchanged in this slice, including Orca-style integer rounding. This slice does not edit `gcode_machine_start_placeholders.rs`.

## Included Behavior

- Default `printable_height = 100` accepts existing small models.
- Numeric JSON and numeric-string values are accepted.
- A model whose planned maximum `print_z` equals `printable_height` is accepted.
- A model whose planned maximum `print_z` exceeds `printable_height` is rejected.
- Invalid `printable_height` values are rejected even when no machine-start placeholder references the option.
- The implementation remains platform-neutral and WASM-safe: no filesystem, terminal, UI, OpenGL, or native runtime behavior in `ares-core`.

## Deferred Behavior

- `extruder_printable_height` and per-extruder height maps from `PrintObject.cpp:307-315` and `GCodeProcessor.cpp:1923`.
- Exact Orca UI/localized error text, object labels, `print_height_error_infos`, and `limit_filament_maps`.
- Bed-area, wrapping-exclude-area, and extruder unprintable-area validation from `check_multi_extruder_gcode_valid`.
- Shrinkage-compensation-specific error handling from `Print.cpp:1379-1383`.
- SLA `SLAPrinterConfig` `printable_height` behavior from `PrintConfig.hpp:1829`.
- Any change to `[max_print_height]`, `[max_print_z]`, or other placeholder rendering.

## Tests

Add focused tests under `crates/ares-core/src/pipeline/tests/printable_height.rs`:

- `run_slicing_pipeline` rejects `square_pyramid_ascii_stl()` with `printable_height = 0.3` because its planned top `print_z` is `0.4`.
- `run_slicing_pipeline` accepts the same model with `printable_height = "0.4"`.
- public `slice` rejects before returning G-code when `printable_height = 0.3`.
- invalid values such as `-0.1`, `"abc"`, and arrays reject with `SliceError::InvalidInput`.

RED/GREEN verification uses `cargo nextest run -p ares-core printable_height`. Related placeholder regression uses `cargo nextest run -p ares-core max_print_height_placeholder_gcode max_print_z_placeholder_gcode`.

Full verification before commit uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust file LOC checks.

## Docs Impact

Update `docs/roadmap.md` after implementation to record that `printable_height` now has concrete slicing-height validation, not only machine-start placeholder rendering.
