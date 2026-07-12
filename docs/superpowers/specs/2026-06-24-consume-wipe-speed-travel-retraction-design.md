# Consume Wipe Speed Travel Retraction Design

## Goal

Consume OrcaSlicer's existing `role_based_wipe_speed` and `wipe_speed` options in Ares' ordinary travel-retraction wipe G-code path. The previous wipe slice emits concrete wipe moves, but it still uses the travel move feedrate and a distance-only retraction split. This slice replaces that temporary behavior with the upstream wipe-speed selection and speed-limited retraction-during-wipe calculation for the existing straight-segment travel wipe path.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1183-1184` declares `role_based_wipe_speed` as `ConfigOptionBool` and `wipe_speed` as `ConfigOptionFloatOrPercent`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5502-5539` defines `role_based_wipe_speed`, default `true`, and `wipe_speed`, default `80%`, with `ratio_over = "travel_speed"` and `min = 0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:340-341` selects wipe speed from the writer's current speed when `role_based_wipe_speed` is enabled, otherwise from `config.get_abs_value("wipe_speed")`, then clamps to at least `10.0` mm/s.
- `OrcaSlicer/src/libslic3r/GCode.cpp:344-360` calculates wipe path length and limits `retractionDuringWipe` to `retraction_speed * wipe_path_length / wipe_speed`, moving any excess remaining retraction into `retractionBeforeWipe`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:432-476` applies the same speed selection/clamp before emitting wipe movement at `_wipe_speed * 60`.

## Ares Destination Boundary

- `crates/ares-core/src/options/layer_change_retraction.rs` parses `role_based_wipe_speed` and `wipe_speed` into `LayerChangeRetraction`.
- `crates/ares-core/src/gcode_travel_retraction.rs` stores the previous printed segment's feedrate, chooses the wipe feedrate, clamps it to `600` mm/min, and computes the speed-limited wipe retraction split.
- `crates/ares-core/src/gcode.rs` passes the completed move feedrate into `TravelRetractionState` without growing the file beyond the 400 LOC repository limit.
- `crates/ares-core/src/tests/travel_retraction_gcode/wipe.rs` becomes the executable acceptance surface for wipe-speed behavior.

## Included Behavior

1. `role_based_wipe_speed` defaults to `true`, accepts only boolean values through the same runtime option parser style as nearby travel-retraction booleans, and rejects invalid values with the option key in the error.
2. `wipe_speed` defaults to `80%`, accepts a non-negative numeric value in mm/s or a percent string resolved over the parsed `travel_speed`, and rejects invalid or non-finite values with the option key in the error.
3. When `role_based_wipe_speed = true`, ordinary travel wipe moves use the previous completed print move's feedrate. For the existing role fixture, a sparse infill print at `60` mm/s produces `F3600` on the wipe line.
4. When `role_based_wipe_speed = false`, ordinary travel wipe moves use `wipe_speed`; with default `travel_speed = 120` mm/s and `wipe_speed = "80%"`, the wipe line uses `F5760`.
5. The selected wipe speed is clamped to at least `10` mm/s, so an absolute `wipe_speed = 5` still emits `F600`.
6. During-wipe retraction capacity is computed from the selected wipe speed:
   - `base_before = retraction_length * retract_before_wipe`
   - `remaining = retraction_length - base_before`
   - `available_wipe_distance = min(wipe_distance, previous_print_segment_length)`
   - `wipe_speed_mm_s = selected_wipe_feedrate / 60.0`
   - `retraction_speed_mm_s = retract_feedrate / 60.0`
   - `max_during_wipe = retraction_speed_mm_s * available_wipe_distance / wipe_speed_mm_s`
   - `during_wipe = min(remaining, max_during_wipe)`
   - `before_wipe = base_before + (remaining - during_wipe)`
7. Excess remaining retraction that cannot fit into the speed-limited wipe is emitted before the wipe, matching Orca's `retractionBeforeWipe += retraction_length_remaining - retractionDuringWipe` behavior. The previous Ares leftover-after-wipe simplification is removed for this path.
8. `retract_before_wipe = 100%` still emits a non-extruding wipe move at the selected wipe feedrate when a previous print segment is available.
9. Existing suppressions remain unchanged: omitted or false `wipe`, `wipe_distance = 0`, firmware retraction, no previous print segment, travel below `retraction_minimum_travel`, and `reduce_infill_retraction` suppression do not emit wipe moves.

## Deferred Behavior

- Full Orca multi-point wipe path storage, clipping, and `Wipe::path` lifecycle.
- Toolchange wipe and multi-material wipe tower/MMU behavior.
- Loop-specific wipe options such as `wipe_on_loops` and `wipe_before_external_loop`.
- Cooling marker tags, GCodeProcessor reserved wipe tags, and adaptive pressure-advance wipe markers.
- Avoid-crossing-perimeters interactions and any UI/viewer behavior.

## Concrete Acceptance Examples

Given the existing travel-retraction fixture with retraction length `0.8` mm, retraction speed `30` mm/s, previous sparse-infill print speed `60` mm/s, travel speed `120` mm/s, and a one-millimeter previous print segment:

```gcode
; wipe=true, wipe_distance=0.5, retract_before_wipe=50, role_based_wipe_speed=true
G1 E-0.55 F1800 ; retract
G1 X0.5 Y0 E-0.25 F3600 ; wipe and retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract
```

```gcode
; wipe=true, wipe_distance=0.5, retract_before_wipe=100, role_based_wipe_speed=true
G1 E-0.8 F1800 ; retract
G1 X0.5 Y0 F3600 ; wipe and retract
G1 X2 Y0 F7200 ; travel
G1 E0.8 F1800 ; unretract
```

```gcode
; wipe=true, wipe_distance=0.5, retract_before_wipe=50, role_based_wipe_speed=false, wipe_speed="80%"
G1 E-0.64375 F1800 ; retract
G1 X0.5 Y0 E-0.15625 F5760 ; wipe and retract
```

```gcode
; wipe=true, wipe_distance=0.5, retract_before_wipe=50, role_based_wipe_speed=false, wipe_speed=5
G1 E-0.4 F1800 ; retract
G1 X0.5 Y0 E-0.4 F600 ; wipe and retract
```

## Tests

- Update existing wipe tests that currently expect travel-speed wipe lines and leftover-after-wipe retraction.
- Add focused tests for default role-based wipe speed, explicit `wipe_speed` percent over `travel_speed`, minimum wipe-speed clamp, and invalid runtime values for `role_based_wipe_speed` / `wipe_speed`.
- Record RED before implementation with `cargo nextest run -p ares-core travel_retraction_wipe`.
- Record GREEN after implementation with the same focused command.
- Final verification must include `cargo fmt --check`, `cargo nextest run -p ares-core travel_retraction_wipe`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks.

## Documentation Impact

After implementation review approval, update `docs/roadmap.md` to record that `role_based_wipe_speed` and `wipe_speed` now affect ordinary Ares travel-retraction wipe feedrate and retraction split behavior, while full Orca wipe-path/toolchange/wipe-tower behavior remains deferred.
