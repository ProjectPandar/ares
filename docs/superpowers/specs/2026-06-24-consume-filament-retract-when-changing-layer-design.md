# Consume Filament Retract When Changing Layer Design

## Scope

Consume OrcaSlicer's `filament_retract_when_changing_layer` filament-prefixed retract override into Ares' existing layer-change retraction G-code path. This is a source-cited Rust rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retract_when_changing_layer` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5062-5067` defines the unprefixed `retract_when_changing_layer` `coBools` option with default `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7122-7152` creates filament-prefixed nullable override definitions from the unprefixed extruder option type/default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `retract_when_changing_layer` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8187-8208` includes `filament_retract_when_changing_layer` in `filament_options_with_variant`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5625-5628` gates layer-change retraction with `FILAMENT_CONFIG(retract_when_changing_layer)` before calling `retract(...)`.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Focused G-code behavior tests belong under `crates/ares-core/src/tests/layer_change_retraction_gcode/`.
- The existing layer-change G-code emitter remains the destination consumer; this slice should not add a new emitter path.

## Included Behavior

1. When `filament_retract_when_changing_layer` is absent, keep the current unprefixed `retract_when_changing_layer` behavior unchanged.
2. When the first configured `filament_retract_when_changing_layer` value is an explicit boolean, use it as the effective `LayerChangeRetraction.layer_change_enabled`, overriding the unprefixed option.
3. Accept the same Orca-style nullable bool runtime shapes already parsed by Ares for nullable bool vectors: scalar boolean, `null`, boolean/null arrays, and comma-separated `1`, `0`, `nil` strings.
4. Treat a first `null` / `nil` prefixed value as "no filament override for the current single-active-filament path" and fall back to the unprefixed `retract_when_changing_layer` value.
5. Validate all configured prefixed values before G-code output. Empty arrays, invalid string tokens, and non-bool/non-null array members must fail with `SliceError::InvalidInput` containing `filament_retract_when_changing_layer`.
6. Preserve the existing retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, wipe, Z-hop, lift gates, and first-layer/later-layer sequencing.

## Deferred Behavior

- Full Orca dynamic config merge and per-current-filament selection beyond Ares' current first-value single-active-filament path.
- `filament_z_hop_types`, filament-prefixed lift gates, filament-prefixed wipe settings, toolchange/cut/wipe-tower retractions, seam/scarf orchestration, avoid-crossing-perimeters interactions, and full Orca `GCode::retract` parity.
- Spiral-vase normalization changes beyond existing Ares normalization behavior.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core filament_retract_when_changing_layer` fails before implementation because the prefixed option is ignored.
- After implementation, `filament_retract_when_changing_layer = true` enables second-layer retract/unretract even when unprefixed `retract_when_changing_layer = false`.
- After implementation, `filament_retract_when_changing_layer = false` suppresses second-layer retract/unretract even when unprefixed `retract_when_changing_layer = true`.
- A serialized nullable bool string such as `"1,0"` is accepted and uses the first value.
- Invalid prefixed values are rejected with the option key in the error.
- Adjacent layer-change and ordinary travel retraction coverage passes with `cargo nextest run -p ares-core layer_change_retraction_gcode travel_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
