# Consume Machine Start Stat Reserved Placeholders Design

## Goal

Consume OrcaSlicer's `print_time_sec` and `used_filament_length` machine-start placeholders into concrete Ares generated G-code output. This is a narrow runtime G-code placeholder slice, not a new option-metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3082` sets `print_time_sec` and `used_filament_length` on the machine-start placeholder parser before rendering `machine_start_gcode`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:58-79` defines the reserved tags `@PRINT_TIME_SEC@` and `@USED_FILAMENT_LENGTH@`.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1108-1140` replaces those reserved tags with final post-processed print-time seconds and filament-length meters.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10971-10978` defines adjacent `print_time_sec` and `used_filament_length` string placeholders and documents their post-processing semantics.

## Rust Destination Boundary

- `crates/ares-core/src/gcode_placeholders.rs` owns the Ares reserved tag constants already used by `file_start_gcode`.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` owns Ares `machine_start_gcode` placeholder rendering.
- `crates/ares-core/src/tests/machine_start_stat_reserved_placeholders_gcode.rs` will own focused integration coverage.
- `crates/ares-core/src/tests/mod.rs` will register the new test module.

## Included Behavior

1. `[print_time_sec]` in `machine_start_gcode` renders as `@PRINT_TIME_SEC@`.
2. `[used_filament_length]` in `machine_start_gcode` renders as `@USED_FILAMENT_LENGTH@`.
3. Both placeholders compose with existing machine-start placeholders such as `[total_layer_count]` and `[num_extruders]`.
4. Both rendered reserved tags stay before the first layer, inside the existing machine-start custom G-code block.
5. Non-machine-start scopes remain unchanged; for this slice, `layer_change_gcode` keeps `[print_time_sec]` and `[used_filament_length]` literal except for already-supported layer placeholders.

## Deferred Behavior

- Final post-processing that replaces `@PRINT_TIME_SEC@` with actual seconds and `@USED_FILAMENT_LENGTH@` with final filament meters.
- Full Orca `GCodeProcessor` statistics parity.
- Full Orca placeholder parser parity, brace expressions, conditionals, config override precedence, and typed placeholder metadata.
- Public option storage/export changes for `print_time_sec` or `used_filament_length`.
- Any `file_start_gcode` behavior change; it already supports the brace-form reserved placeholders and is intentionally left unchanged.
- UI/preset behavior, model/plate metadata behavior, movement/extrusion behavior, and temperature command generation.

## Acceptance Criteria

- A focused RED nextest run fails before implementation because `machine_start_gcode` keeps `[print_time_sec]` and `[used_filament_length]` literal.
- After implementation, the focused nextest run passes and proves the placeholders render as Ares reserved tags.
- Tests prove the placeholders compose with existing machine-start placeholders without changing those outputs.
- Tests prove the placeholders remain literal in `layer_change_gcode`.
- Full verification uses `cargo nextest run`, not `cargo test`.
- Touched Rust files remain at or below 400 LOC; `gcode_machine_start_placeholders.rs` is already near the limit, so implementation must not push it above 400 LOC.

## Self-Review

- No placeholder or TODO text is left in this spec.
- Scope is limited to the source-cited Orca `GCode.cpp` machine-start runtime placeholder assignment and Ares machine-start rendering.
- The spec does not add a new Ares-owned pipeline or a broad placeholder parser rewrite.
- The behavior is externally visible in generated G-code and directly consumes existing Orca runtime placeholders into concrete output.
