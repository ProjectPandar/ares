# Consume Filament Wipe Distance Design

## Scope

Consume OrcaSlicer's `filament_wipe_distance` filament-prefixed nullable float override into Ares' existing ordinary travel-retraction wipe G-code path. This slice makes the already-registered filament option change concrete wipe movement distance and retraction splitting; it is a source-cited Rust rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_wipe_distance` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6635-6644` defines unprefixed `wipe_distance` as `coFloats`, minimum `0`, default `1mm`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7122-7152` creates filament-prefixed nullable override definitions from the unprefixed extruder option type/default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `wipe_distance` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8208` includes `filament_wipe_distance` in `filament_options_with_variant`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:343-350` uses `wipe_distance` to cap wipe path length and calculate the maximum retraction that can happen during wipe.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7589-7599` requires `FILAMENT_CONFIG(wipe_distance)` to be non-zero before running wipe retraction.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Nullable non-negative number helper code belongs in `crates/ares-core/src/options/layer_change_retraction/parsing.rs` so `layer_change_retraction.rs` stays below the 400 LOC guard.
- Existing ordinary travel wipe emission remains in `crates/ares-core/src/gcode_travel_retraction.rs`; this slice changes only the effective `LayerChangeRetraction.wipe_distance` value feeding that path and does not add a new emitter.
- Focused G-code behavior tests belong under `crates/ares-core/src/tests/travel_retraction_gcode/`.
- `docs/roadmap.md` records the completed runtime slice after implementation review approval.

## Included Behavior

1. When `filament_wipe_distance` is absent, keep the current unprefixed `wipe_distance` behavior unchanged.
2. When the first configured `filament_wipe_distance` value is an explicit finite non-negative number, use it as the effective `LayerChangeRetraction.wipe_distance`, overriding the unprefixed option.
3. Accept the same Orca-style nullable number runtime shapes already parsed by Ares for nullable number vectors: numeric scalar, `null`, numeric/null arrays, and comma-separated number/`nil` strings.
4. Treat a first `null` / `nil` prefixed value as no filament override for Ares' current single-active-filament path and fall back to the unprefixed `wipe_distance` value.
5. Validate all configured prefixed values before G-code output. Empty arrays, invalid string tokens, non-number/non-null array members, non-finite values, and negative values fail with `SliceError::InvalidInput` containing `filament_wipe_distance`.
6. Preserve the existing `wipe`, `filament_wipe`, `retract_before_wipe`, `role_based_wipe_speed`, `wipe_speed`, retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, Z-hop, lift gates, reduce-infill retraction, and pending layer-change sequencing.
7. The visible behavior change is limited to Ares' existing ordinary travel wipe path: a larger effective `filament_wipe_distance` can move farther along the previous printed segment and allow more retraction during wipe, while `filament_wipe_distance = 0` suppresses the existing wipe move and keeps normal retract/travel/unretract output.

## Deferred Behavior

- `filament_retract_before_wipe` runtime override.
- Full Orca dynamic config merge and per-current-filament selection beyond Ares' current first-value single-active-filament path.
- Toolchange wipe, layer-change-specific wipe output, wipe tower behavior, MMU/MMU2 wipe behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters interactions, and full Orca `GCode::retract` parity.
- Full Orca wipe-path storage from arbitrary polylines beyond Ares' existing previous straight-segment travel wipe model.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_wipe_distance` fails because the prefixed option is ignored.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_wipe_distance` passes.
- `filament_wipe_distance = 0.5` overrides unprefixed `wipe_distance = 0.25` and emits the existing `; wipe and retract` move to `X0.5` with `E-0.25` in the synthetic straight-segment fixture.
- `filament_wipe_distance = 0` suppresses existing ordinary travel wipe G-code even when unprefixed `wipe_distance = 0.5`, while preserving normal retract/travel/unretract output.
- A serialized nullable number string such as `"0.5,0.25"` is accepted and uses the first value.
- A first `null` / `nil` prefixed value falls back to the unprefixed `wipe_distance` value.
- Invalid prefixed values are rejected with the option key in the error.
- Adjacent retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
