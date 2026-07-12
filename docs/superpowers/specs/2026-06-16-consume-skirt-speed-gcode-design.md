# Consume Skirt Speed G-code Behavior Design

## Purpose

Consume the existing Orca `skirt_speed` option at the same option-aggregation boundary as the other speed options. Ares already applies `skirt_speed` in `pipeline.rs` by reading `skirt_options()` and then mutating `SpeedOptions`, so G-code output mostly has the right behavior. The problem is that `SliceOptions::speed_options()` does not represent the complete speed configuration: pipeline code owns a local skirt-speed special case. This slice centralizes the source-cited `skirt_speed > 0` override in `SliceOptions::speed_options()` and simplifies the pipeline to consume speed behavior through one `SpeedOptions` value.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1556`
  - `((ConfigOptionFloat,              skirt_speed))`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5609-5616`
  - Defines `skirt_speed`, minimum `0`, units `mm/s`, default `50.0`, and documents that zero means use default layer extrusion speed.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6528-6533`
  - Overrides speed for `erSkirt` paths only when `skirt_speed > 0.0`.

## Rust Destination Boundary

- `crates/ares-core/src/options.rs`
  - Parse `skirt_speed` inside `speed_options()` and apply it through the existing `SpeedOptions::with_skirt_speed` API when the parsed value is greater than zero.
  - Use the resolved external perimeter speed when `skirt_speed == 0`, matching Orca's zero fallback.
  - Preserve the current validation: non-negative numeric or numeric-string values only.
  - Keep the file at or below 400 LOC, preferably by editing the existing one-line `speed_options()` implementation without expanding the method.
- `crates/ares-core/src/speeds.rs`
  - Keep `SpeedOptions::new` defaulting skirt speed to external perimeter speed, matching the zero/fallback behavior.
- `crates/ares-core/src/pipeline.rs`
  - Remove the local `skirt_options.speed_mm_s()` to `SpeedOptions::with_skirt_speed` special case.
  - Generate speed moves with `generate_speed_moves(&layer_extrusion_moves, options.speed_options()?)`.
  - Keep `skirt_options()` in the pipeline only for skirt artifact generation.
- `crates/ares-core/src/tests/skirt_gcode.rs`
  - Add or retain pipeline/G-code coverage proving positive `skirt_speed`, default `skirt_speed`, and zero fallback feedrates survive the relocation.

## Included Behavior

1. Missing `skirt_speed` keeps the existing default of `50.0` mm/s inside `SliceOptions::speed_options()`, producing a `3000` mm/min skirt feedrate with current defaults.
2. Positive `skirt_speed` overrides only skirt print moves in speed generation through `SpeedOptions`.
3. `skirt_speed = 0` keeps using the existing external perimeter speed fallback for skirt print moves.
4. Invalid negative or non-numeric `skirt_speed` inputs continue to return `SliceError::InvalidInput` through the existing parser.
5. G-code output reflects the configured skirt speed through both `;SPEED:print:skirt:...` comments and `G1 F...` feedrate commands before skirt extrusion moves.
6. `pipeline.rs` no longer duplicates skirt-speed option handling after `options.speed_options()?`.

## Deferred Behavior

- Orca first-layer slowdown interpolation, raft-specific speed behavior, max volumetric speed caps, autospeed, and non-skirt role speed interactions in `GCode.cpp`.
- New speed options, acceleration options, jerk options, or UI behavior.
- Changing skirt artifact geometry, skirt ordering, or `skirt_speed` metadata modules.
- Introducing a new Ares-owned speed pipeline or widening speed behavior beyond the cited Orca `skirt_speed` override.

## Docs Impact

No architecture or roadmap update is required. This is a narrow `ares-core` behavior wiring slice documented by this SDD spec, the implementation plan, and focused regression tests.

## Acceptance Criteria

- An option test proves `SliceOptions { skirt_speed: 35 } .speed_options()` returns `35.0` mm/s for skirt print speed.
- An option test proves `SliceOptions { skirt_speed: 0, outer_wall_speed: 40 } .speed_options()` returns `40.0` mm/s for skirt print speed.
- Existing `skirt_speed = 0` fallback behavior remains covered and passing.
- A G-code pipeline test proves positive `skirt_speed` changes skirt feedrate comments and emitted `G1 F...` speed commands.
- `pipeline.rs` no longer contains a local `skirt_speed` branch or calls `with_skirt_speed` after `options.speed_options()?`.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- Modified Rust files stay at or below 400 LOC.

## Safety and Rollback

The change is isolated to speed option wiring and skirt G-code speed output. Rollback is a single commit revert. Existing defaults remain compatible because `skirt_speed` already defaults to `50.0` and `0` retains the current external-perimeter fallback.
