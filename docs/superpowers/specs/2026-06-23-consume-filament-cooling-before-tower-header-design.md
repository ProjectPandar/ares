# Filament Cooling Before Tower Header Export Design

## Goal

Consume the existing `filament_cooling_before_tower` option into concrete Ares G-code header output, continuing the source-cited Orca `GCodeConfig` full-config export chain without adding new option metadata or changing the existing machine-start placeholder behavior.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2689-2695` defines `filament_cooling_before_tower` as `coFloats` with `nullable = true`, label `Wipe tower cooling`, sidetext `℃`, develop mode, and default `{ 10. }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1444` includes `((ConfigOptionFloatsNullable, filament_cooling_before_tower))` in `GCodeConfig` after `filament_minimal_purge_on_wipe_tower` and before `filament_tower_interface_pre_extrusion_dist`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes non-banned non-nil full-config keys into G-code config comments as `; key = serialized_value`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:900-954`, `1200`, and `7827-7832` contain adjacent wipe-tower interface configuration use; full wipe-tower execution is outside this slice.

## Current Ares State

- Ares already carries source-cited metadata and registry entries for `filament_cooling_before_tower`.
- Ares already consumes `filament_cooling_before_tower` in `crates/ares-core/src/options/filament_cooling_before_tower.rs` for the `[filament_cooling_before_tower]` machine-start placeholder, using non-negative finite numeric vector parsing and Orca default `10`.
- Recent runtime header slices added `FilamentConfigExports` and `gcode_header.rs` wiring through `filament_minimal_purge_on_wipe_tower`.
- Ares does not currently emit `; filament_cooling_before_tower = ...` in generated G-code header output.

## Design

Add `filament_cooling_before_tower` to the existing filament config export boundary:

- Parse only when the user supplies the key.
- Use a `filament_cooling_before_tower`-specific header export parser because this option already has machine-start placeholder behavior that accepts scalar numbers and separated numeric strings.
- Support existing finite non-negative numeric vector forms for header export: scalar numbers, separated numeric strings, numeric arrays, and numeric-string arrays.
- Preserve empty JSON arrays as an empty header value for full-config export when the placeholder path is not used.
- Serialize values with existing Orca-compatible float vector formatting, comma-separated, preserving empty arrays as an empty header value.
- Append `; filament_cooling_before_tower = <serialized>` in `format_header` immediately after `filament_minimal_purge_on_wipe_tower` and before `filament_cooling_final_speed` while `filament_tower_interface_*` header exports remain deferred.
- Reject invalid input before BTT thumbnail header suppression, matching the current header-export validation order.

## Included Behavior

- Single value export.
- Multiple filament value export.
- Zero value export.
- Scalar numeric export, preserving the existing placeholder-compatible input shape.
- Separated numeric string export, preserving the existing placeholder-compatible input shape.
- Numeric-string array export, preserving the existing placeholder-compatible input shape.
- Empty array export.
- Missing option produces no `filament_cooling_before_tower` header line, preserving existing header behavior.
- Invalid bool, object, null, malformed numeric string, negative, non-finite, or non-numeric array input returns `SliceError::InvalidInput` through the header export path.
- Invalid values are rejected even when `thumbnails` would otherwise skip the generated header.
- Header ordering keeps `filament_cooling_initial_speed`, `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, and `filament_cooling_final_speed` in the consumed upstream-adjacent order when all four are supplied.

## Deferred Behavior

- Full Orca nullable `nil` serialization and `ConfigOptionFloatsNullable` parity.
- Any change to the existing `[filament_cooling_before_tower]` machine-start placeholder parser or defaulting behavior; header export compatibility must not reject existing scalar or separated-string placeholder inputs during the unconditional pre-header validation pass.
- Wipe-tower cooling-before-tower execution, tower interface pre-extrusion, tower ironing, tower purge, tower print temperature, stamping, ramming parameters, loading/unloading path generation, toolchange G-code execution, tower interface behavior, and cooling movement generation.
- Full exhaustive `GCode::append_full_config` parity and flush matrix correction.
- UI/preset behavior, public generated config classes, and any new Ares-owned pipeline abstraction.

## Acceptance Criteria

- Focused RED run: after adding tests and before production wiring, `cargo nextest run -p ares-core filament_cooling_before_tower_gcode` fails because the header line is missing and invalid values are not yet rejected by the header export path.
- Focused GREEN run passes with the same command after implementation.
- Adjacent filament cooling header export tests pass with `cargo nextest run -p ares-core filament_cooling_before_tower_gcode filament_minimal_purge_gcode filament_cooling_final_speed_gcode`.
- Existing machine-start placeholder/runtime tests for `filament_cooling_before_tower` remain passing.
- Full verification before commit passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files remain at or below 400 LOC
- `docs/roadmap.md` records the new source-cited runtime slice and deferred nullable/wipe-tower behavior.
