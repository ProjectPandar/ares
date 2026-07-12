# Filament Minimal Purge Header Export Design

## Goal

Consume the existing `filament_minimal_purge_on_wipe_tower` option into concrete Ares G-code header output, continuing the source-cited Orca `GCodeConfig` full-config export chain without adding new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2678-2687` defines `filament_minimal_purge_on_wipe_tower` as `coFloats`, label `Minimal purge on wipe tower`, sidetext `mm³`, minimum `0`, advanced mode, default `{ 15. }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1443` includes `((ConfigOptionFloats, filament_minimal_purge_on_wipe_tower))` in `GCodeConfig` after `filament_cooling_initial_speed` and before `filament_cooling_before_tower`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes non-banned non-nil full-config keys into G-code config comments as `; key = serialized_value`.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1343`, `2185-2198`, and `2302` use `filament_minimal_purge_on_wipe_tower` for wipe-tower purge-volume decisions; that wipe-tower execution path is outside this slice.

## Current Ares State

- Ares already carries source-cited metadata and registry entries for `filament_minimal_purge_on_wipe_tower`.
- Recent runtime header slices added `FilamentConfigExports` and `gcode_header.rs` wiring through `filament_cooling_final_speed`.
- Ares does not currently emit `; filament_minimal_purge_on_wipe_tower = ...` in generated G-code, so the option remains metadata-only for observable header output.

## Design

Add `filament_minimal_purge_on_wipe_tower` to the existing filament config export boundary:

- Parse only when the user supplies the key.
- Require a JSON array of finite non-negative numbers.
- Serialize values with existing Orca-compatible float vector formatting, comma-separated, preserving empty vectors as an empty header value.
- Append `; filament_minimal_purge_on_wipe_tower = <serialized>` in `format_header` immediately after `filament_cooling_initial_speed` and before `filament_cooling_final_speed` when both are present.
- Reject invalid input before BTT thumbnail header suppression, matching the current header-export validation order.

## Included Behavior

- Single value export.
- Multiple filament value export.
- Zero value export.
- Empty vector export.
- Missing option produces no `filament_minimal_purge_on_wipe_tower` header line.
- Invalid scalar, string, bool, string array, negative, object, or null input returns `SliceError::InvalidInput`.
- Invalid values are rejected even when `thumbnails` would otherwise skip the generated header.
- Header ordering keeps `filament_cooling_initial_speed`, `filament_minimal_purge_on_wipe_tower`, and `filament_cooling_final_speed` in consumed upstream order when all three are supplied.

## Deferred Behavior

- Wipe-tower purge-volume execution from `WipeTower2.cpp:1343`, `2185-2198`, and `2302`.
- `filament_cooling_before_tower`, tower interface pre-extrusion, tower ironing, tower purge, tower print temperature, stamping, ramming parameters, loading/unloading path generation, toolchange G-code execution, tower interface behavior, and cooling movement generation.
- Full exhaustive `GCode::append_full_config` parity and flush matrix correction.
- UI/preset behavior, public generated config classes, and any new Ares-owned pipeline abstraction.

## Acceptance Criteria

- Focused RED run: after adding tests and before production wiring, `cargo nextest run -p ares-core filament_minimal_purge_gcode` fails because the header line is missing and invalid values are not yet rejected by the header export path.
- Focused GREEN run passes with the same command after implementation.
- Adjacent filament cooling header export tests pass with `cargo nextest run -p ares-core filament_minimal_purge_gcode filament_cooling_initial_speed_gcode filament_cooling_final_speed_gcode`.
- Full verification before commit passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files remain at or below 400 LOC
- `docs/roadmap.md` records the new source-cited runtime slice and deferred wipe-tower purge behavior.
