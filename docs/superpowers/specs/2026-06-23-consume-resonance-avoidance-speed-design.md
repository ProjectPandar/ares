# Consume Resonance Avoidance Speed Design

## Goal

Consume OrcaSlicer `resonance_avoidance`, `min_resonance_avoidance_speed`, and `max_resonance_avoidance_speed` as concrete Ares speed/G-code behavior instead of leaving them as staged option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1276-1279` declares the `MachineEnvelopeConfig` resonance avoidance options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4516-4539` defines defaults: disabled by default, minimum speed `70`, maximum speed `120`, and non-negative float bounds for min/max.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6544-6581` consumes the options during print speed selection after filament max volumetric speed capping. It applies only when `path.role() == erExternalPerimeter`, skips adjustment when the reference speed is above the max avoidance speed, snaps speeds below the midpoint down to the minimum side, and snaps speeds in the upper half up to the maximum.

## Rust Destination Boundary

- Parse the three options in `crates/ares-core/src/options/speed.rs` as part of `SpeedOptions`.
- Store the runtime settings in `crates/ares-core/src/speeds/config.rs` with accessors in `crates/ares-core/src/speeds/config/accessors.rs`.
- Add a small `crates/ares-core/src/speeds/resonance_avoidance.rs` helper, register it from `crates/ares-core/src/speeds.rs`, and call it from `crates/ares-core/src/speeds/volumetric.rs` after the existing volumetric cap and before volumetric rate smoothing/layer-time slowdown.
- Add focused runtime/G-code tests under `crates/ares-core/src/pipeline/tests/resonance_avoidance.rs` and register the module from `crates/ares-core/src/pipeline/tests.rs`.

## Included Behavior

- Default Ares output remains unchanged because `resonance_avoidance` defaults to `false`.
- When enabled, only `ToolpathMoveKind::Print` moves with `PrintPathRole::ExternalPerimeter` are adjusted.
- If the capped external-perimeter speed is below `max_resonance_avoidance_speed`, speeds below `min + ((max - min) / 2)` clamp to `min(capped_speed, min_resonance_avoidance_speed)`, matching Orca's `std::min(speed, min)` branch.
- Speeds at or above that midpoint, while still below max, snap to `max_resonance_avoidance_speed`.
- If the reference speed entering resonance avoidance is greater than max, the external perimeter keeps that capped speed for this move.
- Volumetric capping still happens before resonance avoidance and may keep speed below min in the lower-half branch.
- Later Ares behavior, including volumetric rate smoothing and layer-time slowdown, may further reduce the selected speed.
- `min_resonance_avoidance_speed` and `max_resonance_avoidance_speed` accept numeric JSON or numeric strings, default to `70` and `120`, and must be finite non-negative values.
- `resonance_avoidance` accepts only booleans and defaults to `false`.

## Deferred Behavior

- Do not implement Orca's mutable `m_resonance_avoidance` loop-wide state beyond the observable per-move skip when the current reference speed exceeds max. Ares does not yet model one Orca `ExtrusionPath` as a mutable G-code loop object, so this slice applies the same decision to each speed move.
- Do not change `machine_min_travel_rate` or `machine_min_extruding_rate`; Orca consumes those in `GCodeProcessor` time estimation rather than in `GCode::print_machine_envelope`.
- Do not add UI behavior, firmware machine-limit output, time-estimator behavior, or new crates/dependencies.

## Docs Impact

No user-facing docs beyond this source-cited spec and implementation plan are required; this is an internal option-to-runtime-behavior slice with tests as executable documentation.

## Acceptance Criteria

- A focused RED test proves enabled `resonance_avoidance` changes external-perimeter G-code feedrates before implementation.
- Runtime/G-code tests use a non-first layer, or explicitly set `initial_layer_speed` alongside `outer_wall_speed`, so the tested reference speed is the intended external perimeter speed rather than Ares first-layer speed substitution.
- Enabled resonance avoidance snaps a 100 mm/s external perimeter to 120 mm/s when min is 70 and max is 120.
- Enabled resonance avoidance preserves a lower-half external perimeter at 60 mm/s when min is 70 and max is 120, matching Orca's `std::min(speed, min)` branch instead of raising speed to min.
- Enabled resonance avoidance does not change internal perimeter feedrates.
- Enabled resonance avoidance does not change external perimeter feedrates when `outer_wall_speed` is above `max_resonance_avoidance_speed`.
- Invalid resonance option values return `SliceError::InvalidInput`.
- Verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust files stay at or below 400 LOC.
