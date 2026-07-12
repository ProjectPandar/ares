# Consume Travel Slope Z-Hop Type Design

## Scope

Consume OrcaSlicer's `z_hop_types`, `filament_z_hop_types`, and `travel_slope` into Ares' existing ordinary-travel Z-hop retraction path. This slice makes the already-registered lift-type options change concrete G-code for ordinary travel retractions instead of always emitting a separate vertical `lift Z` before the XY travel. It is a source-cited Rust rewrite slice, not a new Ares-owned travel pipeline.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:246-250` defines `LiftType` as `NormalLift`, `SpiralLift`, and `SlopeLift`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1377-1378` owns the `z_hop_types` and `travel_slope` GCodeConfig fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:527-530` maps `ZHopType` strings: `Auto Lift`, `Normal Lift`, `Slope Lift`, and `Spiral Lift`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5169` defines `z_hop_types` with default `Slope Lift` and `travel_slope` with default `3` degrees and range `1..=90`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-84`, `7122-7152`, `7164-7224`, and `8188-8201` make `filament_z_hop_types` a nullable filament-prefixed override for `z_hop_types` and keep `travel_slope` in the extruder/printer variant path.
- `OrcaSlicer/src/libslic3r/Extruder.cpp:215-218` converts `travel_slope` degrees to radians for G-code movement.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7443-7455` maps `ZHopType` to `LiftType`, with unknown or auto fallback to `NormalLift`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7458-7578` chooses the lift type before ordinary travel retraction.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:623-648` schedules lazy lift, and `GCodeWriter.cpp:719-747` consumes scheduled `SlopeLift` during travel by emitting an early sloped XYZ move before the remaining travel.

## Rust Destination Boundary

- Runtime lift-type parsing belongs in a new focused module under `crates/ares-core/src/options/layer_change_retraction/` so existing `layer_change_retraction.rs` and `parsing.rs` stay below the 400 LOC guard.
- `crates/ares-core/src/options/layer_change_retraction.rs` should carry only the new constants, struct fields, and calls into that parser.
- Ordinary-travel slope movement belongs in `crates/ares-core/src/gcode_travel_retraction.rs` and should reuse `GCodeWriter` for position updates.
- If `GCodeWriter` needs a helper, add only a small generic `travel_to_xyz_with_comment` method in `crates/ares-core/src/gcode_writer.rs`.
- Focused behavior tests belong in a new `crates/ares-core/src/tests/travel_retraction_gcode/z_hop_type.rs` module registered from `travel_retraction_gcode.rs`.
- `docs/roadmap.md` records the completed runtime slice after implementation review approval.

## Included Behavior

1. Parse `z_hop_types` as Orca strings `Auto Lift`, `Normal Lift`, `Slope Lift`, and `Spiral Lift`. Use Orca's default `Slope Lift` when the option is absent.
2. Parse nullable `filament_z_hop_types` as the first configured single-active-filament override. A first explicit value overrides `z_hop_types`; a first `nil` or `null` falls back to `z_hop_types`.
3. Validate all configured `z_hop_types` and `filament_z_hop_types` values before G-code output. Empty arrays, invalid strings, empty serialized tokens, non-string/non-null filament members, and non-string unprefixed members fail with `SliceError::InvalidInput` containing the option key.
4. Parse `travel_slope` as the first configured finite value in degrees, default `3`, range `1..=90`; reject invalid values with `SliceError::InvalidInput` containing `travel_slope`.
5. Preserve current explicit `Normal Lift` ordinary-travel behavior: after retract, emit the vertical Z lift before XY travel, then restore before unretract.
6. For ordinary-travel `Slope Lift`, emit the retract first, then split the lift into the XY travel path only when the XY distance is long enough for the configured angle. Convert `travel_slope` degrees to radians, compute `slope_xy_distance = z_hop / tan(travel_slope_radians)`, move from the current XY position toward the travel target by that distance while raising to `current_z + z_hop`, then emit the remaining XY travel at the raised Z. Restore to the original Z before unretract.
7. If the ordinary-travel XY distance is zero or the travel angle `atan2(z_hop, xy_distance)` is greater than or equal to `travel_slope_radians`, emit no separate slope-top move and emit the main travel as a single raised XYZ move to the travel target, then restore before unretract. This matches Orca's `SlopeLift` guard and following `xy_z_move` in `GCodeWriter.cpp:731-763`.
8. In this slice, accepted `Auto Lift` and `Spiral Lift` values fall back to `Normal Lift` for ordinary travel output. This keeps parser parity with Orca's accepted keys while deferring Auto overhang selection and spiral arc output without guessing incomplete geometry.
9. Preserve existing `z_hop`, `filament_z_hop`, lower/upper lift gates, lift-enforce gates, retraction length, restart extra, retraction speed, deretraction speed, firmware retraction, wipe, reduce-infill retraction, minimum-travel, pending layer-change, and current role sequencing.

## Deferred Behavior

- `Spiral Lift` arc output and `Auto Lift` overhang-sensitive selection. They are parsed in this slice but intentionally emitted as `Normal Lift` until those source boundaries are implemented.
- Layer-change slope/spiral lift and eager lift.
- Full Orca unknown-position handling, bed-area spiral safety checks, arcs, non-straight/multi-segment travel paths, avoid-crossing-perimeters, support/internal exceptions, seam/scarf behavior, toolchange/cut/wipe-tower retractions, and full Orca `GCode::retract` orchestration.
- Multi-extruder/current-filament selection beyond Ares' current first-value single-active-filament path.

## Acceptance Criteria

- RED: after adding focused tests, `cargo nextest run -p ares-core z_hop_type` fails because `z_hop_types` / `filament_z_hop_types` / `travel_slope` are ignored or unvalidated.
- GREEN: after implementation, `cargo nextest run -p ares-core z_hop_type` passes.
- A focused ordinary-travel test proves explicit `Normal Lift` still emits the existing vertical `lift Z` before the XY travel.
- A focused ordinary-travel test proves default or explicit `Slope Lift` emits a sloped XYZ travel lift-top move before the remaining XY travel and does not emit the separate pre-travel `lift Z` line.
- A focused ordinary-travel test proves too-short `Slope Lift` travel emits one raised XYZ travel to the target and no separate pre-travel `lift Z` line.
- Focused ordinary-travel tests prove accepted `Auto Lift` and `Spiral Lift` values fall back to the explicit `Normal Lift` sequence for this slice.
- A focused ordinary-travel test proves `filament_z_hop_types = Normal Lift` overrides unprefixed/default slope behavior.
- Invalid `z_hop_types`, `filament_z_hop_types`, and `travel_slope` values are rejected with the relevant option key in the error.
- Adjacent ordinary travel retraction coverage passes with `cargo nextest run -p ares-core travel_retraction_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.
- No new dependency, crate, filesystem access, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned pipeline replacement is introduced.
