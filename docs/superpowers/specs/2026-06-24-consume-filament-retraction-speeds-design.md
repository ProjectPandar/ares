# Consume filament retraction speeds in G-code

## Goal

Make Orca's filament retraction speed override options affect concrete Ares retraction G-code instead of remaining metadata-only options.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retraction_speed` and `filament_deretraction_speed` as filament extruder override keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5322-5337` defines the base `retraction_speed` and `deretraction_speed` options, including the rule that zero deretraction speed means the same speed as retraction.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7167-7224` includes `retraction_speed` and `deretraction_speed` in extruder and filament retract key lists.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:184-198` reads the active extruder's retraction and deretraction speeds and falls back from zero deretraction speed to retraction speed.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1078` emits E-axis retract and unretract moves using `filament()->retract_speed() * 60` and `filament()->deretract_speed() * 60`.

## Ares Boundary

- Parse the first single-extruder runtime values in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Reuse the existing `LayerChangeRetraction` feedrate fields already consumed by ordinary travel retraction and layer-change retraction G-code.
- Add focused tests under the existing `crates/ares-core/src/tests/travel_retraction_gcode/` and `crates/ares-core/src/tests/layer_change_retraction_gcode/` test module structure.

## Selected Approach

Use the filament-prefixed speed options as runtime overrides when present:

- `filament_retraction_speed` overrides `retraction_speed`.
- `filament_deretraction_speed` overrides `deretraction_speed`.
- If the effective deretraction speed is zero, use the effective retraction speed.
- Values remain millimeters per second in options and are converted to G-code feedrates by multiplying by 60, matching the existing Ares retraction speed path and Orca's `GCodeWriter`.

This is the smallest source-cited slice because Ares already has the G-code plumbing for retraction feedrates. The slice should not introduce a new extruder model, new config merge layer, or independent retraction pipeline.

## Alternatives Rejected

- Full Orca filament override merge: too broad for this slice because it would need multi-extruder config selection, nullable override fallback, and broader dynamic config behavior.
- Adding separate G-code writer fields for filament speeds: unnecessary because `LayerChangeRetraction` already carries the effective retract/unretract feedrates to the travel and layer-change paths.
- Metadata-only option updates: rejected by the active goal because the user asked for concrete slicing behavior before adding more options.

## Included Behavior

- A configured first value for `filament_retraction_speed` changes ordinary travel retract feedrate output.
- A configured first value for `filament_deretraction_speed` changes ordinary travel unretract feedrate output.
- The same effective feedrates are used for layer-change retraction output.
- `filament_deretraction_speed = 0` falls back to the effective `filament_retraction_speed` when that override is present.
- Invalid configured filament speed values are rejected before G-code output with `SliceError::InvalidInput` mentioning the offending option key.
- Existing unprefixed `retraction_speed` and `deretraction_speed` behavior remains unchanged when filament-prefixed overrides are absent.

## Deferred Behavior

- Full Orca `DynamicPrintConfig` merging from nullable `filament_*` keys into unprefixed extruder keys.
- Multi-extruder/current-filament speed selection beyond Ares' current first-value runtime convention.
- Nullable `nil` filament override fallback semantics.
- Toolchange, cutter, extruder-change, wipe tower, and long/nozzle-cut retractions.
- `z_hop_types`, `travel_slope`, seam/scarf behavior, and full Orca `GCode::retract` orchestration.

## Testing

- Add RED tests proving filament-prefixed speed values change ordinary travel retraction G-code feedrates.
- Add RED tests proving filament-prefixed speed values change layer-change retraction G-code feedrates.
- Add a fallback test for `filament_deretraction_speed = 0`.
- Add validation coverage for invalid `filament_retraction_speed` and `filament_deretraction_speed` values.
- Use `cargo nextest run -p ares-core filament_retraction_speed` for the focused RED/GREEN loop.
- Run full verification before commit: `cargo fmt --check`, focused nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, cached diff check, and touched Rust LOC guard.

## Docs Impact

Update `docs/roadmap.md` with a dated source-cited runtime slice entry after the implementation is independently approved.
