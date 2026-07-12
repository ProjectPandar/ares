# Consume MMU Scalar Config Header Design

## Source Boundary

This slice ports the header/config serialization surface for the Orca GCodeConfig MMU scalar options immediately before the filament loading-speed vector block:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1427-1435` declares the `// Orca: mmu` GCodeConfig tuple lines for `cooling_tube_retraction`, `cooling_tube_length`, `high_current_on_filament_swap`, `parking_pos_retraction`, `extra_loading_move`, `machine_load_filament_time`, `machine_tool_change_time`, and `machine_unload_filament_time`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2472-2497` defines the three machine timing scalar float options and their non-negative defaults.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4779-4819` defines the cooling tube, high-current swap, parking retraction, and extra-loading scalar options. `cooling_tube_retraction`, `cooling_tube_length`, and `parking_pos_retraction` are non-negative floats, `high_current_on_filament_swap` is a bool, and `extra_loading_move` is a float that may be negative.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes non-banned full config key/value pairs into G-code config output.

## Current Ares Gap

Ares already has registry coverage for these keys, and adjacent filament loading-speed options already reach concrete G-code header output. The former source-line-only `PrintConfig.hpp` modules were removed by the Option pinning cleanup. The eight MMU scalar GCodeConfig values still do not reach the Ares G-code header, so profiles can carry these existing Orca options without visible config output.

## Behavior

Add concrete G-code config header exports for:

- `cooling_tube_retraction`: scalar float, finite, `>= 0`, emits `; cooling_tube_retraction = <value>`.
- `cooling_tube_length`: scalar float, finite, `>= 0`, emits `; cooling_tube_length = <value>`.
- `high_current_on_filament_swap`: scalar bool, emits `; high_current_on_filament_swap = 1` or `0`.
- `parking_pos_retraction`: scalar float, finite, `>= 0`, emits `; parking_pos_retraction = <value>`.
- `extra_loading_move`: scalar float, finite, may be negative, emits `; extra_loading_move = <value>`.
- `machine_load_filament_time`: scalar float, finite, `>= 0`, emits `; machine_load_filament_time = <value>`.
- `machine_tool_change_time`: scalar float, finite, `>= 0`, emits `; machine_tool_change_time = <value>`.
- `machine_unload_filament_time`: scalar float, finite, `>= 0`, emits `; machine_unload_filament_time = <value>`.

Missing options emit no config line. Invalid values return `SliceError::InvalidInput` naming the offending key. Validation must still run before BTT thumbnail header suppression, matching the existing `format_header` behavior that calls `options.filament_config_exports()` before deciding whether config lines are appended.

Preserve upstream-adjacent order in the Ares config header:

1. `cooling_tube_retraction`
2. `cooling_tube_length`
3. `high_current_on_filament_swap`
4. `parking_pos_retraction`
5. `extra_loading_move`
6. `machine_load_filament_time`
7. `machine_tool_change_time`
8. `machine_unload_filament_time`
9. `filament_loading_speed`

Reuse the existing dedicated config-header and filament-config-export modules from the previous header slices. Do not add new crates, dependencies, CLI flags, public API surface, or independent Ares pipeline behavior.

## Deferred Behavior

This slice does not implement MMU loading/unloading motion, high-current firmware commands, parking-position movement, cooling-tube movement, filament swap timing statistics, tool-change timing estimates, wipe tower path generation, WipeTower2 behavior, full Orca `append_full_config` exhaustive export, or UI/preset behavior. Those remain separate source-cited slices against Orca `GCode.cpp`, `GCode/WipeTower2.cpp`, and related timing/statistics code.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for the eight consumed MMU scalar config header options. No user-facing CLI/API documentation changes are required because this slice only makes already accepted Orca config keys visible in existing G-code header output and does not add CLI flags, public API surface, or new user workflows.

## Acceptance Criteria

- RED: a focused `cargo nextest run -p ares-core mmu_scalar_config_header_gcode` fails before production changes because the eight new config header lines are absent and invalid values are not rejected.
- GREEN: the same focused nextest command passes after implementation.
- Adjacent header tests pass with `cargo nextest run -p ares-core mmu_scalar_config_header_gcode filament_load_unload_speed_gcode`.
- Full verification uses `cargo nextest run --workspace` and avoids the legacy Cargo test runner.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks pass.
- `gcode_config_header.rs`, `filament_config_export.rs`, `filament_config_export/serialization.rs`, and any touched test module stay at or below 400 LOC.
