# Consume Dont Slow Down Outer Wall Design

## Goal

Consume the existing `dont_slow_down_outer_wall` option in Ares' current speed stage so it has concrete generated G-code impact instead of remaining metadata-only. This slice ports OrcaSlicer's rule that external perimeters are not eligible for slowdown when the option is enabled, mapped narrowly onto Ares' existing `slow_down_layers` interpolation path.

## Source Boundary

This slice is source-cited to:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1520`, which declares `dont_slow_down_outer_wall` as `ConfigOptionBools` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2340-2347`, which defines the option label, tooltip, and default `false`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:80-84`, where `CoolingLine::adjustable(bool slowdown_external_perimeters)` excludes `TYPE_EXTERNAL_PERIMETER` lines when external perimeters must not be slowed.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:346-350`, where the current extruder's `dont_slow_down_outer_wall` value is copied into `PerExtruderAdjustments`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:418-435`, where external perimeter G-code lines are detected and omitted from `TYPE_ADJUSTABLE` when the option is enabled.

Existing Ares `slow_down_layers` behavior is source-cited separately to `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1627` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3306-3314`. This slice does not claim to implement Orca's full layer-time `CoolingBuffer` slowdown algorithm.

## Ares Destination Boundary

The Rust destination boundary is limited to:

- `crates/ares-core/src/options/slow_down_layers.rs`: parse the per-filament-style boolean option using first-entry semantics suitable for `ConfigOptionBools`.
- `crates/ares-core/src/options.rs`: pass the parsed flag into `SpeedOptions`.
- `crates/ares-core/src/options/tests/slow_down_layers.rs`: add parser tests for `dont_slow_down_outer_wall`.
- `crates/ares-core/src/speeds/config.rs`: store and expose the flag on `SpeedOptions`.
- `crates/ares-core/src/speeds/slow_down_layers.rs`: skip `PrintPathRole::ExternalPerimeter` interpolation when the flag is enabled.
- `crates/ares-core/src/speeds/slow_down_layers/tests.rs`: speed-stage tests for enabled and disabled behavior.
- `crates/ares-core/src/pipeline/tests/slow_down_layers.rs`: pipeline G-code tests proving emitted feedrates differ only for external perimeters.

No option metadata, registry definitions, public API, CLI, WASM bindings, new crates, new dependencies, profile loading, full layer-time estimation, `slow_down_for_layer_cooling`, `slow_down_layer_time`, `slow_down_min_speed`, `fan_cooling_layer_time`, or independent Ares speed model is in scope.

## Runtime Behavior

Ares must parse `dont_slow_down_outer_wall` as a strict boolean first-entry option:

- Missing value defaults to Orca's `false`.
- A scalar JSON boolean is accepted.
- A JSON boolean array uses the first entry, matching Ares' existing first-entry behavior for vector-like per-filament options.
- Empty arrays, non-boolean array first entries, strings, numbers, null, and objects return `SliceError::InvalidInput` mentioning `dont_slow_down_outer_wall`.

When `dont_slow_down_outer_wall` is `false`, the existing `slow_down_layers` interpolation remains unchanged:

- layer `0` uses existing first-layer speed logic;
- layers `1..slow_down_layers-1` interpolate external perimeter speed from `initial_layer_speed` toward `outer_wall_speed`;
- layer `slow_down_layers` and later use the normal role speed.

When `dont_slow_down_outer_wall` is `true`, Ares must leave `PrintPathRole::ExternalPerimeter` print moves at their normal non-first-layer role speed for layers `1..slow_down_layers-1`. Other roles already eligible for Ares' existing `slow_down_layers` interpolation remain unchanged. Travel moves, Skirt moves, disabled `slow_down_layers` values `0` and `1`, and the existing volumetric cap behavior remain as they are today.

The volumetric cap still applies after configured speed selection. If the normal external perimeter speed is then capped by `filament_max_volumetric_speed`, that cap may still lower the final feedrate; `dont_slow_down_outer_wall` only opts external perimeters out of the slow-layer interpolation step.

## Deferred Behavior

This slice does not implement:

- Orca's full `CoolingBuffer::calculate_layer_slowdown` layer-time algorithm.
- `slow_down_for_layer_cooling`, `slow_down_layer_time`, or `slow_down_min_speed`.
- Per-extruder runtime switching beyond Ares' existing first-entry option model.
- External perimeter markers in generated G-code comments such as `;_EXTERNAL_PERIMETER`.
- Support, ironing, gap fill, top/bottom solid infill, or roles Ares does not currently generate.
- Any changes to `first_x_layer_fan_speed`; that option is primarily consumed by Orca profile layer-change templates and remains outside this `libslic3r` runtime slice.

## Acceptance Criteria

- Omitted `dont_slow_down_outer_wall` defaults to `false`.
- Scalar boolean and first boolean array entry forms parse into `SpeedOptions`.
- Invalid forms return `SliceError::InvalidInput` mentioning `dont_slow_down_outer_wall`.
- With `slow_down_layers = 4`, `outer_wall_speed = 90`, and `initial_layer_speed = 30`, layer-1 external perimeter speed remains interpolated to `45 mm/s` when the option is omitted or false.
- With the same speed settings and `dont_slow_down_outer_wall = true`, layer-1 external perimeter speed remains `90 mm/s`.
- Internal perimeter and sparse infill slow-layer interpolation are unchanged when the option is true.
- First-layer external perimeter speed remains controlled by `initial_layer_speed`.
- `slow_down_layers = 0` and `1` still disable interpolation.
- The emitted pipeline G-code `;SPEED:print:external_perimeter:*` feedrate proves the option changes output G-code, not only internal structs.
- The existing volumetric cap still runs after the option suppresses slow-layer interpolation.
- No new option metadata is added.
- All changed Rust files under `crates/` stay at or below 400 LOC.

## Verification Criteria

The implementation is not complete until fresh verification includes:

- `cargo test -p ares-core options::tests::slow_down_layers --lib`
- `cargo test -p ares-core speeds::slow_down_layers::tests --lib`
- `cargo test -p ares-core pipeline::tests::slow_down_layers --lib`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; bad=1 } END { exit bad }'`
