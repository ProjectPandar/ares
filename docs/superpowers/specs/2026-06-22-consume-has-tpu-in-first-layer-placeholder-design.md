# Consume has_tpu_in_first_layer Placeholder Design

## Source Boundary

- Upstream behavior: `OrcaSlicer/src/libslic3r/GCode.cpp:3032-3034` computes `first_layer_filaments = print.get_slice_used_filaments(true)`, checks whether any referenced `m_config.filament_type.values[idx]` equals `"TPU"`, and registers `has_tpu_in_first_layer` as a `ConfigOptionBool`.
- Upstream option definition: `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2784-2796` defines `filament_type` as `coStrings`, populates it from the shared material type database, and defaults it to `"PLA"`.
- Upstream bool serialization: `OrcaSlicer/src/libslic3r/Config.hpp:1823-1826` serializes `ConfigOptionBool` as `"1"` for true and `"0"` for false.
- Rust destination boundary: `crates/ares-core/src/options/filament_type.rs` owns filament string-vector accessors, and `crates/ares-core/src/gcode_placeholders.rs` owns `machine_start_gcode` placeholder replacement.

## Problem

Ares already accepts `filament_type` and consumes it in filament display/header paths, but `machine_start_gcode` does not consume Orca's `[has_tpu_in_first_layer]` placeholder. User start G-code that branches on first-layer TPU presence is left with a literal placeholder even when the existing option contains TPU.

## Design

Add a narrow runtime helper on `SliceOptions` that reads `filament_type` as a non-empty string vector and returns whether any currently modeled first-layer filament is exactly `"TPU"`. In Ares' current single-path slicer, every configured `filament_type` entry is treated as part of the first-layer filament set until full Orca `Print::get_slice_used_filaments(true)` object/region tracking is ported. Missing `filament_type` follows Orca's default `"PLA"`, producing false.

`machine_start_gcode` will replace `[has_tpu_in_first_layer]` with `"1"` when that helper returns true and `"0"` otherwise. The replacement is scoped only to machine start G-code; layer-change and other custom G-code scopes continue to leave this placeholder literal unless those scopes are separately ported.

Because `crates/ares-core/src/options/filament_type.rs` and `crates/ares-core/src/gcode_placeholders.rs` are close to the 400 LOC project limit, this slice may factor the existing `is_all_bbl_filament` string-vector validation into a small private helper reused by both placeholder helpers. The helper must stay private to `filament_type.rs` and must not add dependencies, public API, file I/O, terminal behavior, UI behavior, or WASM-incompatible code.

## Included Behavior

- `[has_tpu_in_first_layer]` in `machine_start_gcode` renders `1` when any configured `filament_type` entry is exactly `"TPU"`.
- The same placeholder renders `0` when no configured entry is `"TPU"`, including Orca's default `"PLA"`.
- Invalid provided `filament_type` values used by this placeholder return `SliceError::InvalidInput` with the option name.
- Existing `filament_type` display/header behavior and existing `[is_all_bbl_filament]` behavior remain unchanged.
- No new option metadata is added.

## Deferred Behavior

- Full `Print::get_slice_used_filaments(true)` parity is deferred until Ares ports Orca's multi-material object/region first-layer usage tracking.
- Support-only filament usage, wipe tower first-layer usage, object-specific first-layer filtering, and region-specific material tracking are deferred.
- Other nearby `GCode.cpp` placeholders such as `outer_wall_volumetric_speed`, `scan_first_layer`, `has_wipe_tower`, `is_extruder_used`, and bed/first-layer geometry placeholders are out of scope.
- Non-machine-start custom G-code placeholder scopes are out of scope.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core has_tpu_in_first_layer` fails before implementation because `[has_tpu_in_first_layer]` remains literal.
- After implementation, the focused run passes and covers true, false, default, invalid, and non-machine-start scope behavior.
- Adjacent filament placeholder/header tests still pass with `cargo nextest run -p ares-core filament_type_gcode is_all_bbl_filament has_tpu_in_first_layer`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
