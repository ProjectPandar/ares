# Consume Slow Down Layers Design

## Goal

Port the concrete OrcaSlicer `slow_down_layers` speed interpolation behavior into Ares' current single-object, non-raft G-code pipeline. This slice must make the existing option affect generated print feedrates for early non-first layers, not add more inert option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1627` declares `slow_down_layers` as a `ConfigOptionInt` in `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3306-3314` defines "Number of slow layers", minimum `0`, default `0`, and describes linear speed increase over the specified number of layers.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6502-6515` implements the non-raft branch: when `slow_down_layers > 1`, `raft_layers == 0`, and the current layer id is greater than `0` but less than `slow_down_layers`, it linearly interpolates from the first-layer speed to the selected normal path speed using `layer_id / slow_down_layers`.

This Ares slice implements only the non-raft branch that has an existing Ares destination: generated print moves and their emitted G-code feedrates.

## Current Ares Boundary

- `crates/ares-core/src/options.rs` already parses speed options into `SpeedOptions`.
- `crates/ares-core/src/speeds.rs` selects role speeds and first-layer speeds for `LayerExtrusionMoves`.
- `crates/ares-core/src/speeds/volumetric.rs` currently applies a later volumetric cap after configured speed selection.
- `crates/ares-core/src/gcode_move_emit.rs` emits the chosen feedrate in G-code `F` values and `;SPEED:*` comments.
- Existing Ares roles covered by this slice are `ExternalPerimeter`, `InternalPerimeter`, `Brim`, `SparseInfill`, `Bridge`, and `InternalBridge`.

## Design

Add `slow_down_layers` to `SpeedOptions` as a non-negative `u32` parsed from `SliceOptions::speed_options()`. Missing values default to Orca's `0`; `0` and `1` both disable the interpolation because upstream only enters the branch for values greater than `1`.

During `generate_speed_moves`, choose the configured speed in this order:

1. Use the existing first-layer speed rules for layer `0`.
2. Use the existing role-speed rules for later layers.
3. For print moves on layers where `layer_id > 0 && layer_id < slow_down_layers`, compute the role's first-layer reference speed:
   - perimeter-like roles use `initial_layer_speed`: `ExternalPerimeter`, `InternalPerimeter`, and `Brim`.
   - non-perimeter print roles use `initial_layer_infill_speed`: `SparseInfill`, `Bridge`, and `InternalBridge`.
4. If the first-layer reference speed is lower than the selected normal role speed, set speed to:
   `first_layer_speed + (normal_speed - first_layer_speed) * (layer_id / slow_down_layers)`.
5. If the first-layer reference speed is greater than or equal to the selected normal role speed, leave the normal role speed unchanged.
6. Apply the existing volumetric cap after this interpolation so the filament max volumetric speed remains the final print-speed limiter.

Travel moves must not use `slow_down_layers`; they continue to use existing travel and first-layer travel speed rules.

Skirt moves must not use `slow_down_layers` in this slice. Upstream applies the non-raft slow-layer interpolation at `OrcaSlicer/src/libslic3r/GCode.cpp:6502-6515`, then immediately overrides skirt speed when `skirt_speed > 0` at `OrcaSlicer/src/libslic3r/GCode.cpp:6528-6533`. Ares' current `SpeedOptions` stores the effective skirt speed after fallback, not whether the user configured a positive skirt speed, and Orca's default `skirt_speed` is positive. Excluding Skirt avoids producing non-Orca final feedrates while keeping the slice focused on roles whose existing Ares speed model maps cleanly to the cited branch.

The interpolation is layer-index based, not Z-height based, matching Orca's `_layer` branch. Layer `0` remains controlled by existing first-layer logic. Layer `slow_down_layers` and later layers use normal role speeds.

## File Placement and Size Constraints

- Add focused `slow_down_layers` parsing in `crates/ares-core/src/options/slow_down_layers.rs`.
- Wire it into `crates/ares-core/src/options.rs` via `mod slow_down_layers;` and `SliceOptions::speed_options()`.
- Keep speed application close to `SpeedOptions` in `crates/ares-core/src/speeds.rs` unless the file would exceed the 400 LOC gate; if it would, split the interpolation helper into `crates/ares-core/src/speeds/slow_down_layers.rs`.
- Add parser tests in `crates/ares-core/src/options/tests/slow_down_layers.rs`.
- Add speed-stage tests in `crates/ares-core/src/speeds/tests.rs` or a focused `speeds/slow_down_layers/tests.rs` if needed for the LOC gate.
- Add pipeline G-code tests in `crates/ares-core/src/pipeline/tests/slow_down_layers.rs`.

## Parsing Rules

- Accept integer JSON numbers and integer numeric strings.
- Reject negative values, non-integer values, values greater than `u32::MAX`, non-numeric strings, booleans, null, arrays, objects, and non-finite values.
- Missing option defaults to `0`.
- Parsed values are stored as `u32`.

## Docs Impact

No user-facing documentation, registry metadata, or roadmap update is required for this slice. The existing option metadata already records the upstream `PrintConfig.hpp` tuple; this work changes runtime consumption and is documented by the spec, plan, tests, and commit.

## Out of Scope

- Raft-aware slow layers from `OrcaSlicer/src/libslic3r/GCode.cpp:6516-6525`.
- `slow_down_for_layer_cooling`, `slow_down_layer_time`, `slow_down_min_speed`, `dont_slow_down_outer_wall`, CoolingBuffer behavior, curled perimeter slowdown, and resonance avoidance.
- Support, support interface, top/bottom solid infill, gap fill, ironing, overhang, and any other roles Ares does not currently generate.
- Skirt slow-layer interpolation, because Orca's adjacent post-slowdown skirt-speed override is not represented in Ares' current `SpeedOptions` boundary.
- Any new Ares-owned speed model independent of the cited `libslic3r` behavior.
- Adding new option metadata or milestone-only scaffolding unrelated to runtime speed consumption.

## Acceptance Criteria

- `slow_down_layers = 0` and `slow_down_layers = 1` leave non-first-layer print feedrates unchanged.
- For `slow_down_layers > 1`, layer `1..slow_down_layers-1` print feedrates interpolate linearly from the matching first-layer reference speed to the normal role speed.
- Layer `0` remains controlled by existing first-layer speed behavior.
- Layer `slow_down_layers` and later layers use the normal role speed unless limited by other existing caps.
- Print roles whose first-layer reference speed is not lower than their normal speed are not increased or otherwise changed by `slow_down_layers`.
- Skirt print feedrates are not interpolated by `slow_down_layers`.
- Travel feedrates are not interpolated by `slow_down_layers`.
- The existing filament volumetric cap still applies after slow-layer interpolation.
- Invalid `slow_down_layers` values, including values greater than `u32::MAX`, are rejected at option parsing.
- Tests cover parser behavior, speed-stage interpolation, pipeline G-code feedrate emission, and volumetric-cap composition.
- Fresh verification must include targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate for changed Rust files.
