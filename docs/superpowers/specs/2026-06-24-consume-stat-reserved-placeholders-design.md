# Consume Stat Reserved Placeholders Design

## Goal

Replace Orca-style reserved statistics tags that Ares already emits for `print_time_sec` and `used_filament_length` with concrete numeric values in final G-code, instead of leaving `@PRINT_TIME_SEC@` and `@USED_FILAMENT_LENGTH@` in user-visible output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10967-10975` defines `print_time_sec` and `used_filament_length` as string placeholders whose tooltips say they are replaced during post-processing.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2524-2525` maps `file_start_gcode` placeholder names to `GCodeProcessor` reserved tags before writing the top-of-file custom G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3080` maps `machine_start_gcode` placeholder names to the same reserved tags before processing machine start G-code.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:77-101` includes `@PRINT_TIME_SEC@` and `@USED_FILAMENT_LENGTH@` in the reserved tag vocabulary.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1108-1138` post-processes inline reserved tags by replacing all `print_time_sec` reserved tags with normal-mode print time formatted as `%.2f`, and all `used_filament_length` reserved tags with total filament millimeters divided by `1000.0`, also formatted as `%.2f`.

## Current Ares State

- `crates/ares-core/src/gcode_placeholders.rs` already converts `{print_time_sec}` and `{used_filament_length}` in `file_start_gcode` to reserved tags.
- `crates/ares-core/src/gcode_machine_start_stat_placeholders.rs` already converts `[print_time_sec]` and `[used_filament_length]` in `machine_start_gcode` to reserved tags.
- `crates/ares-core/src/gcode_reserved_tags.rs` defines the two reserved tag strings.
- `crates/ares-core/src/gcode_filament_stats.rs` already computes total used filament millimeters from `LayerExtrusionMoves` and normal print time from `LayerSpeedMoves` for end-of-file statistics and time cost.
- `crates/ares-core/src/gcode.rs` applies line numbering after full G-code assembly, so reserved tag replacement must run before `gcode_line_numbers::apply`.

## Included Behavior

- Add a platform-neutral `ares-core` finalization pass that replaces every Ares-emitted `@PRINT_TIME_SEC@` and `@USED_FILAMENT_LENGTH@` occurrence in the fully assembled G-code string before line numbering.
- Format `print_time_sec` as seconds with exactly two decimal places, using the same finalized `LayerSpeedMoves` print-time calculation as current Ares filament cost statistics.
- Format `used_filament_length` as meters with exactly two decimal places, using total `LayerExtrusionMoves::total_extrusion_mm()` divided by `1000.0`.
- Apply the replacement to both `file_start_gcode` reserved tags and `machine_start_gcode` reserved tags.
- Replace all repeated reserved tags in a line or file.
- Keep unknown placeholders and ordinary literal `[print_time_sec]` / `[used_filament_length]` text outside the scopes that Ares converts to reserved tags unchanged.
- Keep line numbering as the last G-code transformation so line numbers include the numeric replacement output, not reserved tag text.

## Deferred Behavior

- Full Orca `GCodeProcessor` time processor parity, acceleration-aware time estimation, silent-mode estimation, preview statistics, M73 placeholder handling beyond existing Ares behavior, pause/custom-G-code statistics, and per-extruder or per-filament statistics remain deferred.
- Full Orca placeholder parser parity, expression parsing, vector indexing, UI/preset behavior, public config export, and unrelated `PrintConfig` option storage remain deferred.
- No new crates, dependencies, file I/O, terminal behavior, UI behavior, OpenGL behavior, or Ares-owned slicing pipeline design is introduced.

## Rust Destination

- Reuse or expose the existing Ares statistics calculations in `crates/ares-core/src/gcode_filament_stats.rs` rather than duplicating print-time or filament-length math.
- Add a focused module such as `crates/ares-core/src/gcode_stat_placeholders.rs` for the reserved-tag finalization pass if that keeps file sizes below the 400 LOC guard.
- Wire the pass from `crates/ares-core/src/gcode.rs` immediately before `gcode_line_numbers::apply` without growing that file beyond 400 LOC.
- Update tests in:
  - `crates/ares-core/src/tests/machine_start_stat_reserved_placeholders_gcode.rs`
  - `crates/ares-core/src/tests/custom_gcode_end.rs`
- Update `docs/roadmap.md` to move the previously deferred final statistics replacement into completed runtime behavior.

## Acceptance Criteria

- A machine start template `;TIME [print_time_sec]` emits a line matching `;TIME <seconds-with-two-decimals>` before the first layer, and the output contains no `@PRINT_TIME_SEC@`.
- A machine start template `;FILAMENT [used_filament_length]` emits a line matching `;FILAMENT <meters-with-two-decimals>` before the first layer, and the output contains no `@USED_FILAMENT_LENGTH@`.
- A file start template containing `{print_time_sec}` and `{used_filament_length}` emits numeric values before the generated header, with no reserved tags left in output.
- A template with repeated stat placeholders replaces every generated reserved tag occurrence.
- `layer_change_gcode` containing `[print_time_sec]` and `[used_filament_length]` remains literal because that scope is not mapped to reserved tags by the cited Orca boundary.
- With `gcode_add_line_number = true`, numeric statistic lines are line-numbered and the numbered G-code contains no reserved tags.
- Existing filament statistics and time-cost tests continue to use the same underlying totals.

## Verification

- First run a targeted RED test that expects numeric reserved-tag replacement and verify it fails against the current reserved-tag-only implementation.
- After implementation, run the targeted nextest set for changed behavior.
- Run:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check` after staging
- Check touched Rust files remain at or below 400 LOC.
