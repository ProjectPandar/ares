# Filament Cooling Moves Header Export Design

## Goal

Consume the existing `filament_cooling_moves` option into concrete Ares G-code header output, continuing the source-cited Orca `GCodeConfig` full-config export chain without adding new option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2646-2653` defines `filament_cooling_moves` as `coInts`, label `Number of cooling moves`, minimum `0`, maximum `20`, advanced mode, default `{ 4 }`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1441` includes `((ConfigOptionInts, filament_cooling_moves))` in `GCodeConfig` immediately after `filament_toolchange_delay`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes non-banned non-nil full-config keys into G-code config comments as `; key = serialized_value`.
- `OrcaSlicer/src/libslic3r/GCode/WipeTower2.cpp:1357-1364` also reads `filament_cooling_moves` into `m_filpar[idx].cooling_moves` for single-extruder MM wipe-tower behavior; that runtime cooling-move path is explicitly outside this slice.

## Current Ares State

- Ares already carries source-cited metadata and registry entries for `filament_cooling_moves`.
- Recent runtime header slices added `FilamentConfigExports` and `gcode_header.rs` wiring for adjacent Orca GCodeConfig keys through `filament_toolchange_delay`.
- Ares does not currently emit `; filament_cooling_moves = ...` in generated G-code, so the option remains metadata-only for observable output.

## Design

Add `filament_cooling_moves` to the existing filament config export boundary:

- Parse only when the user supplies the key.
- Require a JSON array of integers.
- Enforce the upstream `PrintConfig.cpp:2650-2651` range `0..=20`.
- Serialize values with existing Orca-compatible integer vector formatting, comma-separated, preserving empty vectors as an empty header value.
- Append `; filament_cooling_moves = <serialized>` in `format_header` immediately after `filament_toolchange_delay`, preserving the source order around `PrintConfig.hpp:1440-1442`.
- Reject invalid input before BTT thumbnail header suppression, matching the current header-export validation order.

## Included Behavior

- Single value export.
- Multiple filament value export.
- Zero and maximum `20` value export.
- Empty vector export.
- Missing option produces no `filament_cooling_moves` header line.
- Invalid scalar, string, bool, string array, negative, over-maximum, float, object, out-of-i32, or null input returns `SliceError::InvalidInput`.
- Invalid values are rejected even when `thumbnails` would otherwise skip the generated header.

## Deferred Behavior

- Wipe-tower single-extruder MM cooling move execution from `WipeTower2.cpp:1363`.
- Toolchange, ramming, loading/unloading path generation, stamping, and cooling speed behavior.
- `filament_cooling_initial_speed`, `filament_cooling_final_speed`, `filament_stamping_*`, and tower interface behavior.
- Full exhaustive `GCode::append_full_config` parity and flush matrix correction.
- UI/preset behavior, public generated config classes, and any new Ares-owned pipeline abstraction.

## Acceptance Criteria

- Focused RED run: after adding tests and before production wiring, `cargo nextest run -p ares-core filament_cooling_moves_gcode` fails because the header line is missing and invalid values are not yet rejected by the header export path.
- Focused GREEN run passes with the same command after implementation.
- Adjacent filament header export tests pass with `cargo nextest run -p ares-core filament_cooling_moves_gcode filament_toolchange_delay_gcode filament_load_unload_speed_gcode`.
- Full verification before commit passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files remain at or below 400 LOC
- `docs/roadmap.md` records the new source-cited runtime slice and deferred behavior.
