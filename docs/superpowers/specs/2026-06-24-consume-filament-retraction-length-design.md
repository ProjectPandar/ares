# Consume filament retraction length in G-code

## Goal

Make Orca's `filament_retraction_length` override option affect concrete Ares retraction G-code distances instead of remaining a metadata-only filament override key.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-82` lists `filament_retraction_length` as a filament extruder override key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5068-5075` defines `retraction_length` as the retract distance in millimeters, with zero disabling retraction.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:174-177` reads the active extruder's `retraction_length`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1048` emits retract G-code through the active filament retraction length and forwards that length into `_retract`.

## Ares Boundary

- Parse the first single-extruder runtime override value in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Reuse the existing `LayerChangeRetraction.length` and `unretract_length` fields already consumed by ordinary travel retraction and layer-change retraction G-code.
- Add focused tests under `crates/ares-core/src/tests/travel_retraction_gcode/` and `crates/ares-core/src/tests/layer_change_retraction_gcode/`.

## Selected Approach

Use `filament_retraction_length` as a runtime first-value override over `retraction_length` when present. Keep the existing downstream behavior:

- `length = 0` disables ordinary travel and layer-change E-axis retraction.
- `unretract_length` remains `effective_length + retract_restart_extra`.
- All configured `filament_retraction_length` values must be non-negative finite numbers before the first value is used for Ares' current single-extruder runtime.
- Existing retraction speeds, z-hop, wipe, firmware retraction, and minimum-travel gates keep using the same Ares paths.

This keeps the slice aligned with Orca's filament override boundary while avoiding a broader DynamicPrintConfig merge or a new Ares retraction pipeline.

## Alternatives Rejected

- Full filament override merge layer: too broad because it would need nullable fallback, multi-extruder current-filament selection, and interactions with other prefixed filament override keys.
- Duplicating travel/layer-change G-code logic for filament lengths: unnecessary because the existing `LayerChangeRetraction` length already feeds both paths.
- Metadata-only documentation: rejected by the active goal because this slice must add concrete slicing/G-code behavior.

## Included Behavior

- A configured first value for `filament_retraction_length` changes ordinary travel retract and unretract E distances.
- A configured first value for `filament_retraction_length` changes layer-change retract and unretract E distances.
- `filament_retraction_length = 0` disables ordinary travel E-axis retraction just like `retraction_length = 0`.
- `filament_retraction_length = 0` disables layer-change E-axis retraction just like `retraction_length = 0`.
- Invalid configured `filament_retraction_length` values, including invalid later vector entries, are rejected before G-code output with `SliceError::InvalidInput` mentioning `filament_retraction_length`.
- Existing unprefixed `retraction_length` behavior remains unchanged when `filament_retraction_length` is absent.

## Deferred Behavior

- Full Orca `DynamicPrintConfig` merging from nullable `filament_*` keys into unprefixed extruder keys.
- Multi-extruder/current-filament length selection beyond Ares' current first-value runtime convention.
- Nullable `nil` filament override fallback semantics.
- Toolchange, cutter, extruder-change, wipe tower, and long/nozzle-cut retractions.
- `filament_retract_restart_extra`, `filament_z_hop`, `filament_retract_before_wipe`, `filament_wipe_distance`, `z_hop_types`, `travel_slope`, seam/scarf behavior, and full Orca `GCode::retract` orchestration.

## Testing

- Add RED tests proving `filament_retraction_length` changes ordinary travel retraction G-code E distances.
- Add RED tests proving `filament_retraction_length` changes layer-change retraction G-code E distances.
- Add RED tests proving `filament_retraction_length = 0` disables ordinary travel E-axis retraction.
- Add RED tests proving `filament_retraction_length = 0` disables layer-change E-axis retraction.
- Add validation coverage for invalid `filament_retraction_length` values.
- Use `cargo nextest run -p ares-core filament_retraction_length` for the focused RED/GREEN loop.
- Run full verification before commit: `cargo fmt --check`, focused nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, cached diff check, and touched Rust LOC guard.

## Docs Impact

Update `docs/roadmap.md` with a dated source-cited runtime slice entry after implementation approval.
