# Consume Filament Wipe Design

## Scope

Consume OrcaSlicer's `filament_wipe` filament-prefixed nullable override into Ares' existing ordinary travel-retraction wipe G-code path. This slice turns an already-registered option into concrete runtime behavior; it is a source-cited Rust rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_wipe` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628-6633` defines the unprefixed `wipe` `coBools` option with default `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7122-7152` creates filament-prefixed nullable override definitions from the unprefixed extruder option type/default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `wipe` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8208` includes `filament_wipe` in `filament_options_with_variant`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7589-7599` gates wipe retraction with `FILAMENT_CONFIG(wipe)` before finishing the normal retract path.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Existing ordinary travel wipe emission remains in `crates/ares-core/src/gcode_travel_retraction.rs`; this slice changes the effective `LayerChangeRetraction.wipe` value feeding that path and does not add a new emitter.
- Focused G-code behavior tests belong under `crates/ares-core/src/tests/travel_retraction_gcode/`.
- `docs/roadmap.md` records the completed runtime slice after implementation review approval.

## Included Behavior

1. When `filament_wipe` is absent, keep the current unprefixed `wipe` behavior unchanged.
2. When the first configured `filament_wipe` value is an explicit boolean, use it as the effective `LayerChangeRetraction.wipe`, overriding the unprefixed option.
3. Accept the same Orca-style nullable bool runtime shapes already parsed by Ares for nullable bool vectors: scalar boolean, `null`, boolean/null arrays, and comma-separated `1`, `0`, `nil` strings.
4. Treat a first `null` / `nil` prefixed value as no filament override for Ares' current single-active-filament path and fall back to the unprefixed `wipe` value.
5. Validate all configured prefixed values before G-code output. Empty arrays, invalid string tokens, and non-bool/non-null array members fail with `SliceError::InvalidInput` containing `filament_wipe`.
6. Preserve the existing `wipe_distance`, `retract_before_wipe`, `role_based_wipe_speed`, `wipe_speed`, retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, Z-hop, lift gates, reduce-infill retraction, and pending layer-change sequencing.
7. The visible behavior change is limited to Ares' existing ordinary travel wipe path: `filament_wipe = true` can emit a `; wipe and retract` travel-retraction move when unprefixed `wipe = false`, and `filament_wipe = false` can suppress that move when unprefixed `wipe = true`.

## Deferred Behavior

- `filament_wipe_distance` and `filament_retract_before_wipe` runtime overrides.
- Full Orca dynamic config merge and per-current-filament selection beyond Ares' current first-value single-active-filament path.
- Toolchange wipe, layer-change-specific wipe output, wipe tower behavior, MMU/MMU2 wipe behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters interactions, and full Orca `GCode::retract` parity.
- Full Orca wipe-path storage from arbitrary polylines beyond Ares' existing previous straight-segment travel wipe model.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_wipe` fails because the prefixed option is ignored.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_wipe` passes.
- `filament_wipe = true` enables existing ordinary travel wipe G-code even when unprefixed `wipe = false`.
- `filament_wipe = false` suppresses existing ordinary travel wipe G-code even when unprefixed `wipe = true`, while preserving normal retract/travel/unretract output.
- A serialized nullable bool string such as `"1,0"` is accepted and uses the first value.
- A first `null` / `nil` prefixed value falls back to the unprefixed `wipe` value.
- Invalid prefixed values are rejected with the option key in the error.
- Adjacent retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
