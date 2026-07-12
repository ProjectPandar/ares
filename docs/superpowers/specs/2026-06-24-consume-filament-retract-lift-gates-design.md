# Consume Filament Retract Lift Gates Design

## Scope

Consume OrcaSlicer's `filament_retract_lift_above`, `filament_retract_lift_below`, and `filament_retract_lift_enforce` filament-prefixed nullable overrides into Ares' existing Z-hop lift gate path. This slice makes already-registered filament options change concrete ordinary travel and layer-change lift/restore G-code; it is a source-cited Rust rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retract_lift_above`, `filament_retract_lift_below`, and `filament_retract_lift_enforce` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5173-5200` defines unprefixed `retract_lift_above`, `retract_lift_below`, and `retract_lift_enforce`, with default lower gate `0`, default upper gate `0`, and default enforce mode `All Surfaces`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7122-7152` creates filament-prefixed nullable override definitions from the unprefixed extruder option type and default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes the unprefixed lift gate keys in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8201` includes the three filament-prefixed lift gate keys in `filament_options_with_variant`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` applies `retract_lift_above` and `retract_lift_below` before scheduling a lazy lift.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:652-674` applies the same lower/upper gates before eager lift.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7606-7637` applies `retract_lift_enforce` to decide whether a retraction may lift on all, top, bottom, or top-and-bottom surfaces.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Nullable parser helpers belong in `crates/ares-core/src/options/layer_change_retraction/parsing.rs` so `layer_change_retraction.rs` stays below the 400 LOC guard.
- Existing ordinary travel retraction output in `crates/ares-core/src/gcode_travel_retraction.rs` and layer-change retraction output in `crates/ares-core/src/gcode.rs` remain the emitters; this slice changes only the effective lift gate values feeding those paths.
- Focused behavior tests belong under `crates/ares-core/src/tests/travel_retraction_gcode/` and `crates/ares-core/src/tests/layer_change_retraction_gcode/`.
- `docs/roadmap.md` records the completed runtime slice after implementation review approval.

## Included Behavior

1. When a filament-prefixed lift gate option is absent, keep the current unprefixed `retract_lift_above`, `retract_lift_below`, or `retract_lift_enforce` behavior unchanged.
2. When the first configured `filament_retract_lift_above` value is an explicit finite non-negative number, use it as the effective lower Z-hop lift gate for Ares' current single-active-filament path.
3. When the first configured `filament_retract_lift_below` value is an explicit finite non-negative number, use it as the effective upper Z-hop lift gate for Ares' current single-active-filament path. A value of `0` keeps the existing no-upper-bound semantics.
4. When the first configured `filament_retract_lift_enforce` value is an explicit enum string, use it as the effective lift enforcement mode. Accepted values are `All Surfaces`, `Top Only`, `Bottom Only`, and `Top and Bottom`.
5. Treat a first `null` or serialized `nil` prefixed value as no filament override and fall back to the corresponding unprefixed gate.
6. Validate all configured prefixed values before G-code output. Float gate empty arrays, invalid string tokens, non-number/non-null array members, non-finite values, and negative values fail with `SliceError::InvalidInput` containing the option key. Enum gate empty arrays, invalid enum strings, non-string/non-null array members, and invalid serialized tokens fail with `SliceError::InvalidInput` containing the option key.
7. The effective prefixed gates affect both existing ordinary travel retraction Z-hop lift/restore G-code and existing layer-change retraction Z-hop lift/restore G-code.
8. Preserve existing `z_hop`, `filament_z_hop`, retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, wipe, reduce-infill retraction, minimum-travel, pending layer-change, and current non-gap-fill role sequencing.

## Deferred Behavior

- Full Orca dynamic config merge and per-current-filament selection beyond Ares' current first-value single-active-filament path.
- `filament_z_hop_types`, `z_hop_types`, `travel_slope`, non-vertical lift modes, slope lift, spiral lift, and auto lift.
- Toolchange, cut, wipe-tower, seam/scarf, avoid-crossing, support/internal exceptions, ironing-specific top eligibility, and full Orca `GCode::retract` orchestration.
- Changing existing unprefixed `retract_lift_above`, `retract_lift_below`, or `retract_lift_enforce` parser behavior beyond moving helper code required for LOC.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_retract_lift_gates` fails because prefixed lift gate options are ignored or unvalidated.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_retract_lift_gates` passes.
- Ordinary travel G-code proves `filament_retract_lift_above = 0` can override a suppressing unprefixed lower gate and emit the existing lift/restore lines.
- Ordinary travel G-code proves `filament_retract_lift_below = 0` can override a suppressing unprefixed upper gate and emit the existing lift/restore lines.
- Layer-change G-code proves `filament_retract_lift_enforce = Top Only` can override unprefixed `All Surfaces` and suppress non-top layer-change lift while leaving retract/unretract intact.
- A first `null` / `nil` prefixed value falls back to the corresponding unprefixed gate.
- Invalid prefixed float and enum values are rejected with the relevant option key in the error.
- Adjacent retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
