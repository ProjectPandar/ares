# Consume Filament Retraction Minimum Travel Design

## Goal

Consume OrcaSlicer `filament_retraction_minimum_travel` into Ares ordinary XY travel retraction G-code so the filament-scoped override controls whether a travel move emits retract/unretract, instead of only recording the option as metadata.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retraction_minimum_travel` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5048-5054` defines `retraction_minimum_travel` as `coFloats`, default `[2.]`, label "Travel distance threshold".
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7136-7152` clones prefixed filament override definitions from the unprefixed option metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `retraction_minimum_travel` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8208` includes `filament_retraction_minimum_travel` in filament options with variants.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7280-7330` computes travel length and short-travel behavior from `EXTRUDER_CONFIG(retraction_minimum_travel)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7458-7463` suppresses retraction when travel length is less than `FILAMENT_CONFIG(retraction_minimum_travel)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7582-7602` emits the retraction sequence after the travel-retraction decision.

## Ares Boundary

- Parse the prefixed override in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Reuse the existing `LayerChangeRetraction.minimum_travel` field and `TravelRetractionCommand.minimum_travel` path.
- Keep the travel-length decision in `crates/ares-core/src/gcode_travel_retraction.rs` unchanged.
- Add focused runtime tests under `crates/ares-core/src/tests/travel_retraction_gcode/`.
- Update `docs/roadmap.md` with the completed runtime slice.

## Included Behavior

1. When `filament_retraction_minimum_travel` is absent, existing `retraction_minimum_travel` behavior remains unchanged.
2. When `filament_retraction_minimum_travel` is present, its first finite non-negative value overrides `retraction_minimum_travel` for Ares' current single-active-filament travel-retraction path.
3. A low filament threshold can enable retract/unretract around an ordinary travel move even when the unprefixed threshold is high enough to suppress it.
4. A high filament threshold can suppress retract/unretract around an ordinary travel move even when the unprefixed threshold is low enough to allow it.
5. Invalid configured `filament_retraction_minimum_travel` values are rejected before G-code output, including empty arrays, negative values, non-numeric strings, non-finite values, and invalid later vector members. The error includes the prefixed option key.
6. Existing retraction length, restart extra, firmware mode, wipe, wipe speed, Z-hop, reduce-infill retraction, and pending layer-change composition continue to use the same paths.

## Deferred Behavior

- Full Orca dynamic config merge semantics and nullable inheritance for filament overrides.
- Multi-extruder or current-filament selection beyond the current first-value Ares path.
- `filament_retract_when_changing_layer`, `filament_wipe`, `filament_retract_before_wipe`, `filament_wipe_distance`, `filament_retract_restart_extra`, and filament lift-above/below/enforce options.
- Toolchange retraction, wipe tower travel, cut/extruder-change long retraction, avoid-crossing-perimeters, support/internal travel exceptions, short-travel acceleration/jerk, and full `GCode::retract` parity.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_minimum_travel` fails because the prefixed option is ignored.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_minimum_travel` passes.
- Adjacent travel retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
