# Consume Machine Minimum Rate Time Design

## Scope

Consume the existing OrcaSlicer machine-limit options `machine_min_extruding_rate` and `machine_min_travel_rate` into Ares runtime print-time estimation. This is a source-cited `libslic3r` rewrite slice for the G-code statistics path, not new option metadata.

Upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1272-1275` declares `machine_min_travel_rate` and `machine_min_extruding_rate` as `ConfigOptionFloats` under the M205 machine-limit group.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4461-4475` defines the two options as non-negative mm/s values with defaults `{ 0., 0. }`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:2361-2367` copies both option vectors into the time processor machine limits.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:3809`, `4219`, and `4251` convert G-code `F` feedrates from mm/min into mm/s before time processing.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:3917-3918` and `4277-4278` choose travel or extrusion minimum feedrate clamps for time blocks.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5629-5643` implements `minimum_feedrate` and `minimum_travel_feedrate` as `std::max(feedrate, configured_minimum)`.

Ares destination boundary:

- `crates/ares-core/src/gcode_filament_stats.rs`
- `crates/ares-core/src/gcode_stat_placeholders.rs`
- focused tests under `crates/ares-core/src/pipeline/tests/` and `crates/ares-core/src/tests/`

## Current Behavior

Ares already has `LayerSpeedMoves` in mm/s and uses `gcode_filament_stats::normal_print_time_s` for:

- `time_cost` in final filament statistics.
- reserved stat placeholders rendered through `[print_time_sec]` / `{print_time_sec}` after earlier placeholder expansion.

That time estimate currently uses each move's raw `SpeedMove::speed_mm_s()` and ignores `machine_min_extruding_rate` and `machine_min_travel_rate`.

## Required Behavior

- Parse `machine_min_extruding_rate` and `machine_min_travel_rate` from `SliceOptions` as non-negative numeric vectors using the existing Orca-style numeric-vector parser.
- Default both options to Orca's normal/stealth pair `[0.0, 0.0]` when absent.
- Use the first vector value only for Ares's current Normal-mode estimate.
- Reject empty, non-numeric, non-finite, or negative vectors with `SliceError::InvalidInput` naming the offending option key.
- When estimating print time:
  - Print moves use `max(move.speed_mm_s(), machine_min_extruding_rate[0])`.
  - Travel moves use `max(move.speed_mm_s(), machine_min_travel_rate[0])`.
  - Preserve Ares's existing counted-move boundary: layer time starts once the first print move has been seen.
- Apply the clamped estimate everywhere Ares currently exposes the normal print time:
  - final `; total filament cost = ...` when `time_cost` is configured.
  - reserved stat placeholder rendering for `print_time_sec`.
- Do not change emitted movement feedrates or extrusion amounts.

## Deferred Behavior

- Full Orca `GCodeProcessor` / `TimeProcessor` acceleration, jerk, block planning, and mode-specific machine simulation.
- Stealth/silent time mode output; the second vector value is preserved as parsed data but not consumed in this slice.
- Dynamic `M205 S` / `M205 T` parsing from emitted or user custom G-code.
- Emitting `M205 S` / `M205 T` commands.
- Per-extruder or multi-mode machine-limit indexing beyond the first Normal value.
- Any UI, filesystem, terminal, OpenGL, or platform-specific behavior.

## Acceptance Criteria

- Focused RED/GREEN tests prove that `machine_min_extruding_rate` and `machine_min_travel_rate` reduce the printed-time estimate used by stats/placeholders when they exceed generated speeds.
- Tests prove that high minimum rates do not alter emitted G-code movement commands.
- Tests prove invalid minimum-rate values are rejected with the corresponding option key.
- Focused nextest passes:
  `cargo nextest run -p ares-core machine_min_rate_time_gcode`
- Adjacent statistics tests pass:
  `cargo nextest run -p ares-core filament_cost_gcode machine_start_stat_reserved_placeholders_gcode`
- Full verification passes:
  `cargo fmt --check`
  `cargo nextest run --workspace`
  `cargo clippy --workspace --all-targets -- -D warnings`
  `cargo check -p ares-core --target wasm32-unknown-unknown`
  `git diff --check`
  `git diff --cached --check`
- Touched Rust files remain at or below 400 LOC.
