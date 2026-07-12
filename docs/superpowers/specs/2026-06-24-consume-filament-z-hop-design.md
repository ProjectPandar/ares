# Consume Filament Z-hop Design

## Goal

Consume OrcaSlicer's existing `filament_z_hop` override as concrete Ares Z-hop G-code behavior instead of leaving it as inert option data.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_z_hop` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5131` defines `z_hop` as a `ConfigOptionFloats` Z-hop height with default `0.4`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7137-7149` clones selected filament-prefixed override definitions from the unprefixed option metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7188` keeps `z_hop` in the extruder retract key set.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8201` includes `filament_z_hop` in filament option keys.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` chooses the current filament `z_hop` value as the lift distance when the lower/upper lift gates allow it.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1084-1092` restores lifted Z with `restore layer Z`.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:179-182` exposes `z_hop` as retract lift.

## Ares Boundary

- Parse the runtime option in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Reuse the existing `LayerChangeRetraction.z_hop` field and current G-code paths in `crates/ares-core/src/gcode.rs`.
- Add focused tests under `crates/ares-core/src/tests/travel_retraction_gcode/` and `crates/ares-core/src/tests/layer_change_retraction_gcode/`.
- Update `docs/roadmap.md` with this source-cited runtime slice after implementation review.

## Included Behavior

1. If `filament_z_hop` is absent, Ares keeps the existing `z_hop` behavior unchanged.
2. If `filament_z_hop` is present, the first configured finite non-negative value overrides `z_hop` for Ares' current single-active-filament runtime path.
3. The override affects ordinary travel retraction Z-hop lift and restore G-code.
4. The override affects layer-change retraction Z-hop lift and restore G-code.
5. `filament_z_hop = 0` disables the Z-hop lift/restore even when `z_hop` is positive, while preserving existing retract/unretract behavior.
6. Invalid configured `filament_z_hop` values are rejected before G-code output and the error includes `filament_z_hop`. Invalid values include empty arrays, negative values, non-finite values, unparsable strings, and invalid later vector entries.
7. Existing `retract_lift_above`, `retract_lift_below`, and `retract_lift_enforce` gates continue to apply to the effective Z-hop height.

## Deferred Behavior

- Full Orca dynamic config merge from filament-prefixed keys into unprefixed extruder keys.
- Nullable `nil` fallback semantics.
- Multi-extruder or per-current-filament selection beyond Ares' current first-value path.
- `filament_z_hop_types`, `filament_retract_lift_above`, `filament_retract_lift_below`, and `filament_retract_lift_enforce`.
- Non-vertical lift modes, `travel_slope`, slope lift, spiral lift, and auto lift.
- Toolchange, cut, wipe-tower, seam/scarf, avoid-crossing, and full Orca `GCode::retract` orchestration.
- Enforcing Orca's documented `z_hop` maximum of `5` for either the existing unprefixed or new filament-prefixed parser. This slice preserves Ares' current non-negative finite parser shape and only adds the prefixed override.

## Acceptance Criteria

- Focused tests fail before the parser change because `filament_z_hop` is ignored and invalid prefixed values are accepted.
- Focused tests pass after implementation with `cargo nextest run -p ares-core filament_z_hop`.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust file LOC check, keeping every touched Rust file at or below 400 LOC
- The implementation remains in `ares-core` and introduces no filesystem, UI, OpenGL, terminal, platform-specific, or new dependency behavior.
