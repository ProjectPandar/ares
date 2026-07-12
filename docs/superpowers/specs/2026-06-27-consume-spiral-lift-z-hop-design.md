# Consume Spiral Lift Z-hop Design

## Source Boundary

- Upstream enum: `OrcaSlicer/src/libslic3r/PrintConfig.hpp:382-388` defines
  `ZHopType::{zhtAuto,zhtNormal,zhtSlope,zhtSpiral,zhtCount}`.
- Upstream option tuple: `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1377-1378` owns
  `z_hop_types` and `travel_slope`.
- Upstream defaults and constraints:
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5162` defines `z_hop_types`,
    includes `"Spiral Lift"`, and defaults to `zhtSlope`.
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5164-5173` defines `travel_slope`,
    min `1`, max `90`, default `3`, and states it applies to Slope and Spiral Z-hop.
  - `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5039-5047` defines `resolution`,
    default `0.01`; `PrintConfig.cpp:8372-8374` and `8445-8447` clamp it to at least
    `0.001`.
- Upstream behavior owners:
  - `OrcaSlicer/src/libslic3r/GCode.cpp:7443-7454` maps `zhtSpiral` to
    `LiftType::SpiralLift`.
  - `OrcaSlicer/src/libslic3r/Extruder.cpp:215-218` exposes `travel_slope()` as the
    configured degree value converted to radians.
  - `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:668-674` and `725-731` choose a spiral
    lift radius from `z_hop / (2 * PI * atan(travel_slope))` and emit spiral lift before
    the following travel.
  - `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:849-904` implements
    `_spiral_travel_to_z`; when arc fitting is disabled it emits a feedrate line and
    linearly approximates one full XY circle while interpolating Z, with segment count
    `round(16 * (0.01 / resolution))` clamped to `4..=24`.

## Ares Destination Boundary

- Replace the current Ares compatibility behavior where `"Spiral Lift"` is parsed but
  treated as normal vertical lift.
- Extend the platform-neutral `crates/ares-core/src/gcode_lift.rs` lift representation to
  distinguish normal, slope, and spiral lift modes.
- Keep parsing under `crates/ares-core/src/options/layer_change_retraction/lift_type.rs`
  and continue returning the first effective single-extruder value from `z_hop_types`,
  `filament_z_hop_types`, and `travel_slope`.
- Add `SliceOptions::resolution()` under the existing G-code output option boundary, reusing
  Orca default `0.01` and lower clamp `0.001`.
- Emit spiral lift G-code from `crates/ares-core/src/gcode_writer/travel.rs` and consume it
  through the existing travel/layer-change lift paths in `gcode_travel_retraction.rs`,
  `gcode_layer_change_retraction.rs`, and `gcode_move_emit.rs`.

## Included Behavior

1. `"Normal Lift"` keeps the current separate vertical `G1 Z... ; lift Z` behavior.
2. Default and explicit `"Slope Lift"` keep the current sloped-travel behavior.
3. `"Spiral Lift"` emits a source-cited linearized spiral lift before the next travel move:
   - The lift starts at the current XY position and current Z.
   - The emitted radius is `z_hop / (2 * PI * atan(travel_slope_radians))`.
   - The center offset is perpendicular to the next travel vector, matching Orca's
     `{-normalized_y * radius, normalized_x * radius}` shape.
   - The writer emits `;spiral lift Z` when G-code comments are enabled, then one feedrate
     line, then one full-circle sequence of `G1 X... Y... Z...` lines that raises to the
     lifted Z.
   - The following travel stays at the lifted Z and the existing restore/unretract path
     restores the layer Z before printing.
4. Segment count is controlled by `resolution` exactly as the upstream non-arc path:
   `round(16 * (0.01 / resolution))` clamped to `4..=24`.
5. If travel distance is zero, spiral lift falls back to the existing raised target travel
   path instead of inventing a static spiral. This keeps the Ares path deterministic until
   Ares has Orca's current-position-clear state and eager-lift injection boundary.
6. `filament_z_hop_types` overrides unprefixed `z_hop_types`; `nil` continues to fall back
   to the unprefixed mode.

## Deferred Behavior

- `Auto Lift` selection remains deferred. Orca chooses slope or spiral based on overhang
  intersection checks in `GCode::needs_retraction`; Ares does not yet carry that upstream
  overhang travel-analysis boundary.
- Arc fitting remains deferred. This slice implements the upstream `!enable_arc_fitting`
  linearized path only; it does not emit `G17` or `G2`/`G3`.
- Static/eager spiral lift for G-code injection remains deferred.
- Full bed-area safety checks, current-position-clear state, multi-extruder per-filament
  switching, and native viewer/UI behavior are out of scope.
- This slice adds no crates, dependencies, file I/O, terminal behavior, OpenGL, UI code, or
  WASM-incompatible APIs.

## Acceptance Criteria

- Focused RED/GREEN test:
  `cargo nextest run -p ares-core z_hop_type`
  initially fails after tests are updated because `"Spiral Lift"` still falls back to normal
  vertical lift.
- After implementation:
  - Travel retraction with `"z_hop_types": ["Spiral Lift"]`, `z_hop = 0.4`,
    `travel_slope = 45`, `resolution = 0.01`, and comments enabled emits `;spiral lift Z`,
    a `G1 F7200` feedrate line, fifteen intermediate spiral segment moves, one final segment
    at the original XY and lifted Z, then the ordinary XY travel at lifted Z, then restore and
    unretract.
  - The same travel retraction with `resolution = 0.02` emits eight spiral segment moves
    total, proving `resolution` changes concrete G-code behavior.
  - Layer-change retraction with `"Spiral Lift"` emits the same spiral lift before the pending
    layer-change travel and restores before unretracting.
  - `"Auto Lift"` remains explicitly tested as the existing normal fallback and does not emit
    spiral lift output.
  - Existing normal, slope, filament override, nil fallback, invalid `z_hop_types`, invalid
    `travel_slope`, and invalid `resolution` behavior still passes.
- Full verification before commit:
  - `cargo fmt --check`
  - focused nextest for `z_hop_type`
  - adjacent nextest for `travel_retraction_gcode` and `layer_change_retraction_gcode`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust files stay at or under 400 LOC.

## Safety And Rollback

- The new path is gated by explicit `"Spiral Lift"` selection. Defaults continue to use slope
  lift.
- The new code only generates deterministic G-code strings from existing in-memory positions,
  options, and toolpath moves.
- Rollback is deleting the spiral branch and restoring the former `"Spiral Lift"` normal-lift
  compatibility tests.

## Docs Impact

- This spec and its reviewed implementation plan are the documentation update for the slice.
- No `docs/roadmap.md` update is required because this consumes existing source-cited option
  metadata into runtime G-code behavior rather than adding a new milestone.
- No architecture ADR is required because the work stays inside the existing G-code
  writer/retraction boundary and does not introduce a new crate, dependency, or irreversible
  architecture decision.
