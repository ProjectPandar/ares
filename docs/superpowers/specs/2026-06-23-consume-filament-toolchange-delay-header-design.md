# Filament Toolchange Delay Header Export Design

## Goal

Consume the existing `filament_toolchange_delay` option into concrete Ares G-code header output, matching the narrow Orca full-config export behavior before adding more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2636-2644` defines `filament_toolchange_delay` as `coFloats`, label `Delay after unloading`, unit seconds, minimum `0`, advanced mode, default `{ 0. }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1440` includes `((ConfigOptionFloats, filament_toolchange_delay))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes non-banned non-nil full-config keys into G-code config comments as `; key = serialized_value`.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1357-1363` also reads `filament_toolchange_delay` into `m_filpar[idx].delay` for single-extruder MM wipe-tower behavior; that runtime delay path is explicitly outside this slice.

## Current Ares State

- Ares already carries source-cited metadata and registry entries for `filament_toolchange_delay`.
- Recent runtime header slices added `FilamentConfigExports` and `gcode_header.rs` wiring for adjacent Orca `ConfigOptionFloats` keys: `filament_loading_speed`, `filament_loading_speed_start`, `filament_unloading_speed`, and `filament_unloading_speed_start`.
- Ares does not currently emit `; filament_toolchange_delay = ...` in generated G-code, so the option remains metadata-only for observable output.

## Design

Add `filament_toolchange_delay` to the existing filament config export boundary:

- Parse only when the user supplies the key.
- Require a JSON array of finite non-negative numbers, using the existing `optional_float_vector_export` behavior shared by adjacent `ConfigOptionFloats` header options.
- Serialize values with existing Orca-compatible float vector formatting, comma-separated, preserving empty vectors as an empty header value.
- Append `; filament_toolchange_delay = <serialized>` in `format_header` after `filament_unloading_speed_start`, preserving the source order around `PrintConfig.hpp:1436-1441`.
- Reject invalid input before BTT thumbnail header suppression, matching the current header-export validation order.

## Included Behavior

- Single value export, including fractional seconds.
- Multiple filament value export.
- Zero value export.
- Empty vector export.
- Missing option produces no `filament_toolchange_delay` header line.
- Invalid scalar, string, bool, string array, negative, object, or null input returns `SliceError::InvalidInput`.
- Invalid values are rejected even when `thumbnails` would otherwise skip the generated header.

## Deferred Behavior

- Wipe-tower single-extruder MM delay execution from `WipeTower2.cpp:1362`.
- Toolchange, ramming, loading/unloading path generation, and cooling moves.
- `filament_cooling_moves`, `filament_cooling_initial_speed`, `filament_cooling_final_speed`, stamping, and tower interface behavior.
- Full exhaustive `GCode::append_full_config` parity and flush matrix correction.
- UI/preset behavior, public generated config classes, and any new Ares-owned pipeline abstraction.

## Acceptance Criteria

- Focused RED run: after adding tests and before production wiring, `cargo nextest run -p ares-core filament_toolchange_delay_gcode` fails because the header line is missing.
- Focused GREEN run passes with the same command after implementation.
- Adjacent filament header export tests pass with `cargo nextest run -p ares-core filament_toolchange_delay_gcode filament_load_unload_speed_gcode filament_adhesiveness_category_gcode`.
- Full verification before commit passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files remain at or below 400 LOC
- `docs/roadmap.md` records the new source-cited runtime slice and deferred behavior.
