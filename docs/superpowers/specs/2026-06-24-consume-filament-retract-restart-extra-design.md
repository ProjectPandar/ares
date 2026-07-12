# Consume Filament Retract Restart Extra Design

## Goal

Consume OrcaSlicer `filament_retract_restart_extra` into Ares concrete travel and layer-change unretract G-code so the filament-scoped override controls extra extrusion on restart instead of only existing as option metadata.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retract_restart_extra` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306-5313` defines `retract_restart_extra` as `coFloats`, default `[0.]`, label "Extra length on restart".
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7136-7152` clones prefixed filament override definitions from unprefixed option metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `retract_restart_extra` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8200` includes `filament_retract_restart_extra` in filament options with variants.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:200-203` reads the current extruder's `retract_restart_extra`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1004-1012` passes `filament()->retract_restart_extra()` into the ordinary retract state so restart emits the additional amount.

## Ares Boundary

- Parse the prefixed override in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Reuse the existing `LayerChangeRetraction.unretract_length` field, which is already consumed by layer-change and ordinary travel unretract commands.
- Keep `crates/ares-core/src/gcode_travel_retraction.rs` and `crates/ares-core/src/gcode_layer_change_retraction.rs` unchanged.
- Add focused runtime tests under existing retraction G-code test modules.
- Update `docs/roadmap.md` with the completed runtime slice.

## Included Behavior

1. When `filament_retract_restart_extra` is absent, existing `retract_restart_extra` behavior remains unchanged.
2. When `filament_retract_restart_extra` is present, its first finite non-negative value overrides `retract_restart_extra` for Ares' current single-active-filament retraction path.
3. Ordinary travel retraction keeps retract distance equal to the effective retraction length and emits unretract distance as effective retraction length plus effective filament restart extra.
4. Layer-change retraction keeps retract distance equal to the effective retraction length and emits unretract distance as effective retraction length plus effective filament restart extra.
5. A zero filament restart extra can override a positive unprefixed restart extra and restore plain unretract distance.
6. Invalid configured `filament_retract_restart_extra` values are rejected before G-code output, including empty arrays, negative values, non-numeric strings, non-finite values, and invalid later vector members. The error includes the prefixed option key.
7. Existing retraction length, speed, firmware mode, wipe, Z-hop, minimum-travel, reduce-infill retraction, and pending layer-change composition continue to use their current paths.

## Deferred Behavior

- Full Orca dynamic config merge semantics and nullable inheritance for filament overrides.
- Multi-extruder or current-filament selection beyond the current first-value Ares path.
- `filament_retract_restart_extra_toolchange`, `retract_restart_extra_toolchange`, toolchange retraction, wipe tower travel, cut/extruder-change long retraction, and MMU state.
- `filament_retract_when_changing_layer`, `filament_wipe`, `filament_retract_before_wipe`, `filament_wipe_distance`, filament lift-above/below/enforce, and `filament_z_hop_types`.
- Avoid-crossing-perimeters, seam/scarf behavior, support/internal travel exceptions, short-travel acceleration/jerk, and full `GCode::retract` parity.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_restart_extra` fails because the prefixed option is ignored.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_restart_extra` passes.
- Adjacent retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
