# Consume Filament Retract Before Wipe Design

## Scope

Consume OrcaSlicer's `filament_retract_before_wipe` filament-prefixed nullable percent override into Ares' existing ordinary travel-retraction wipe G-code path. This slice makes the already-registered filament option change concrete pre-wipe versus during-wipe retraction splitting; it is a source-cited Rust rewrite slice, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84` lists `filament_retract_before_wipe` in `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5055-5062` defines unprefixed `retract_before_wipe` as `coPercents`, default `100%`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7122-7152` creates filament-prefixed nullable override definitions from the unprefixed extruder option type/default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7224` includes `retract_before_wipe` in extruder and filament retract key sets.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8188-8208` includes `filament_retract_before_wipe` in `filament_options_with_variant`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:329-350` uses `extruder->retract_before_wipe()` to split the remaining retraction length between a pre-wipe retract command and wipe-path retraction.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7589-7599` runs wipe retraction only when filament wipe and wipe distance gates pass, then consumes the calculated pre-wipe and during-wipe retraction lengths.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/layer_change_retraction.rs`.
- Nullable percent helper code belongs in `crates/ares-core/src/options/layer_change_retraction/parsing.rs` so `layer_change_retraction.rs` stays below the 400 LOC guard.
- Existing ordinary travel wipe emission remains in `crates/ares-core/src/gcode_travel_retraction.rs`; this slice changes only the effective `LayerChangeRetraction.retract_before_wipe` value feeding that path and does not add a new emitter.
- Focused G-code behavior tests belong under `crates/ares-core/src/tests/travel_retraction_gcode/`.
- `docs/roadmap.md` records the completed runtime slice after implementation review approval.

## Included Behavior

1. When `filament_retract_before_wipe` is absent, keep the current unprefixed `retract_before_wipe` behavior unchanged.
2. When the first configured `filament_retract_before_wipe` value is an explicit finite percent in `0..=100`, use it as the effective `LayerChangeRetraction.retract_before_wipe`, overriding the unprefixed option.
3. Accept the same Orca-style nullable number runtime shapes Ares already uses for nullable number vectors: numeric scalar, `null`, numeric/null arrays, and comma-separated number/`nil` strings. Values are percentages, not fractions, at the input boundary.
4. Treat a first `null` / `nil` prefixed value as no filament override for Ares' current single-active-filament path and fall back to the unprefixed `retract_before_wipe` value.
5. Validate all configured prefixed values before G-code output. Empty arrays, invalid string tokens, non-number/non-null array members, non-finite values, negative values, and values above `100` fail with `SliceError::InvalidInput` containing `filament_retract_before_wipe`.
6. Preserve the existing `wipe`, `filament_wipe`, `wipe_distance`, `filament_wipe_distance`, `role_based_wipe_speed`, `wipe_speed`, retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, Z-hop, lift gates, reduce-infill retraction, and pending layer-change sequencing.
7. The visible behavior change is limited to Ares' existing ordinary travel wipe path: `filament_retract_before_wipe = 0` moves all speed-allowed retraction into the wipe move, `50` splits the retraction like the current unprefixed value, and `100` performs the full retraction before wipe while keeping the wipe move as a zero-E wipe move.

## Deferred Behavior

- Full Orca dynamic config merge and per-current-filament selection beyond Ares' current first-value single-active-filament path.
- Toolchange wipe, layer-change-specific wipe output, wipe tower behavior, MMU/MMU2 wipe behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters interactions, and full Orca `GCode::retract` parity.
- Full Orca wipe-path storage from arbitrary polylines beyond Ares' existing previous straight-segment travel wipe model.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core filament_retract_before_wipe` fails because the prefixed option is ignored.
- GREEN: after implementation, `cargo nextest run -p ares-core filament_retract_before_wipe` passes.
- `filament_retract_before_wipe = 0` overrides unprefixed `retract_before_wipe = 100` and emits the existing wipe move with during-wipe retraction in the synthetic straight-segment fixture.
- `filament_retract_before_wipe = 100` overrides unprefixed `retract_before_wipe = 0` and emits full pre-wipe retraction followed by a zero-E wipe move.
- A serialized nullable number string such as `"0,100"` is accepted and uses the first percent value.
- A first `null` / `nil` prefixed value falls back to the unprefixed `retract_before_wipe` value.
- Invalid prefixed values are rejected with the option key in the error.
- Adjacent retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode layer_change_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
