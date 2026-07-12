# Consume is_all_bbl_filament Placeholder Design

## Source Boundary

- Upstream behavior: `OrcaSlicer/src/libslic3r/GCode.cpp:3016-3019` computes `used_filaments = print.get_slice_used_filaments(false)` and sets the `is_all_bbl_filament` placeholder to true only when every used filament's `m_config.filament_vendor.values[idx]` is exactly `"Bambu Lab"`.
- Upstream option definition: `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2854-2858` defines `filament_vendor` as `coStrings` with default `"(Undefined)"`.
- Upstream placeholder serialization: `OrcaSlicer/src/libslic3r/Config.hpp:1823-1826` serializes `ConfigOptionBool` as `"1"` for true and `"0"` for false.
- Rust destination boundary: `crates/ares-core/src/options/filament_type.rs` owns filament string-vector accessors, and `crates/ares-core/src/gcode_placeholders.rs` owns `machine_start_gcode` placeholder replacement.

## Problem

Ares already accepts `filament_vendor` and exports it in the generated G-code header, but `machine_start_gcode` does not consume that option for Orca's `[is_all_bbl_filament]` placeholder. User start G-code that branches or comments on Bambu filament status is left with a literal placeholder, so the existing option is metadata/header-only for this path.

## Design

Add a narrow runtime helper on `SliceOptions` that reads `filament_vendor` as a string vector and returns whether all currently modeled used filaments are `"Bambu Lab"`. In Ares' current single-path slicer, all configured filament vendor entries are treated as the used filament set for this placeholder until full Orca multi-material used-filament tracking is ported. Missing `filament_vendor` follows Orca's default `"(Undefined)"`, producing false. Empty vectors are invalid, matching existing non-empty vector validation patterns for placeholder inputs.

`machine_start_gcode` will replace `[is_all_bbl_filament]` with `"1"` when the helper returns true and `"0"` otherwise. The replacement is scoped only to machine start G-code; layer-change and other custom G-code scopes continue to leave this placeholder literal unless those scopes are separately ported.

## Included Behavior

- `[is_all_bbl_filament]` in `machine_start_gcode` renders `1` when every configured `filament_vendor` entry is exactly `"Bambu Lab"`.
- The same placeholder renders `0` when any configured vendor differs, including Orca's default `"(Undefined)"`.
- Invalid `filament_vendor` values used by this placeholder return `SliceError::InvalidInput` with the option name.
- Existing `filament_vendor` header behavior remains unchanged.
- No new option metadata is added.

## Deferred Behavior

- Full `Print::get_slice_used_filaments(false)` parity is deferred until Ares ports Orca's multi-material object/region usage tracking.
- Wipe tower, support-only filament usage, first-layer-only material analysis, and object-specific used-filament filtering are deferred.
- Other nearby `GCode.cpp` placeholders such as `has_tpu_in_first_layer`, `has_wipe_tower`, `is_extruder_used`, `model_name`, and bed/first-layer geometry placeholders are out of scope.
- Non-machine-start custom G-code placeholder scopes are out of scope.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core is_all_bbl_filament` fails before implementation because `[is_all_bbl_filament]` remains literal.
- After implementation, the focused run passes and covers true, false, default, invalid, and non-machine-start scope behavior.
- Adjacent filament header tests still pass with `cargo nextest run -p ares-core filament_type_gcode filament_soluble_gcode filament_is_support_gcode is_all_bbl_filament`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
