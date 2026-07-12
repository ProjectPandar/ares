# Consume Time Cost Total Filament Cost Design

## Goal

Consume OrcaSlicer `time_cost` as runtime G-code footer behavior in Ares. The slice adds Orca-compatible total cost reporting to the existing filament statistics footer, so the configured printer hourly cost contributes to `; total filament cost = ...` instead of remaining option metadata only.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1357`: `((ConfigOptionFloat,               time_cost))` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3758-3768`: `time_cost` option definition, label, tooltip, `money/h` sidetext, minimum `0`, advanced mode.
- `OrcaSlicer/src/libslic3r/GCode.cpp:1903-1937`: `DoExport::update_print_estimated_stats` computes `total_cost` from material cost plus `config.time_cost.getFloat() * (normal_print_time / 3600.0)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3476-3482`: non-BBL export writes `; total filament used [g] = ...` and `; total filament cost = ...` after filament stats.
- `OrcaSlicer/src/libslic3r/Print.cpp:3668-3683` and `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1099-1104`: the post-processor recognizes total filament cost tags separately from per-filament `; filament cost = ...`.

## Ares Boundary

- Modify `crates/ares-core/src/gcode_filament_stats.rs`.
- Modify `crates/ares-core/src/gcode_finish.rs`.
- Modify `crates/ares-core/src/gcode.rs` only to thread finalized `layer_speed_moves` into finish G-code formatting.
- Modify focused tests in `crates/ares-core/src/pipeline/tests/filament_cost_gcode.rs`.
- Keep `ares-core` platform-neutral and WASM-compatible: no file I/O, no terminal behavior, no UI, no OpenGL.

## Behavior

1. Ares continues to emit existing footer lines:
   - `; filament used [mm] = ...`
   - `; filament used [cm3] = ...`
   - `; filament used [g] = ...` only when material weight is positive
   - `; filament cost = ...` only when material weight and material cost are positive
2. Ares adds Orca-style total footer lines after per-filament stats and before `M2`:
   - `; total filament used [g] = ...` when material weight is positive
   - `; total filament cost = ...` when material cost plus time cost is positive
3. `time_cost` is parsed as a non-negative finite numeric value, accepting the first value from Ares' existing Orca numeric forms and defaulting to `0`.
4. `time_cost` contributes only to the total filament cost line:
   - `total_cost = material_cost + time_cost * (normal_print_time_s / 3600.0)`
   - The normal print time is estimated from finalized `LayerSpeedMoves` by summing each layer after its first print move appears.
   - For each layer, start with no previous point. For each finalized `SpeedMove`, use the previous move point as the segment start, or the current move point for the first move so the first segment length is zero. Once a move with `ToolpathMoveKind::Print` has appeared in that layer, add Euclidean segment length divided by the current move's `speed_mm_s` when both length and speed are positive. Ignore pre-print travel, cross-layer gaps, zero-length segments, and non-positive speeds. Acceleration and jerk are not included in this slice.
5. The existing per-filament `; filament cost = ...` line remains material-only, matching Orca's separate per-filament and total-cost channels.
6. A zero `time_cost` preserves current material-only behavior except for adding total lines when the corresponding positive material statistics exist.
7. Invalid `time_cost` values fail at the slicing/G-code boundary with `SliceError::InvalidInput`.

## Deferred Behavior

- Full Orca `GCodeProcessor` post-processing, reserved tag replacement, and file-based post-process passes are deferred.
- Full `PrintStatistics` public object exposure, filename placeholders such as `{total_cost}`, and UI consumption are deferred.
- Multi-extruder per-tool cost lists, wipe tower cost, support/model volume split statistics, and BBL-specific export differences are deferred.
- This slice does not change movement, extrusion, speed planning, thermal behavior, or any Ares-owned pipeline design.

## Docs Impact

- The source-cited design and implementation plan are the required docs for this narrow runtime footer slice.
- `docs/roadmap.md` is not updated because this is one option-consumption slice inside the existing roadmap direction, not a new milestone or priority change.
- `docs/architecture/` is not updated because this slice does not introduce a new architectural decision; it threads existing pipeline artifacts into existing G-code footer formatting.

## Acceptance Criteria

- A focused RED nextest run proves `time_cost` currently does not affect total footer cost.
- After implementation, focused nextest proves:
  - `time_cost` adds an hourly-cost component to `; total filament cost = ...`.
  - `; filament cost = ...` remains material-only.
  - zero material cost with positive `time_cost` still emits `; total filament cost = ...`.
  - invalid `time_cost` values are rejected.
  - movement and extrusion command lines do not change when only `time_cost` changes.
- Full verification passes:
  - `cargo fmt --check`
  - focused `cargo nextest run -p ares-core filament_cost`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - Rust LOC guard for touched Rust files
