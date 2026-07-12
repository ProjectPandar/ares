# Consume Wipe Tower Config Header Design

## Source Boundary

This slice ports the header/config serialization surface for the existing Orca GCodeConfig options immediately after the filament stamping fields:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:74-77` declares `WipeTowerType::{Type1, Type2}`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:212-216` maps `WipeTowerType` keys `"type1"` and `"type2"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1457-1461` includes `wipe_tower_type`, `purge_in_prime_tower`, `enable_filament_ramming`, `tool_change_on_wipe_tower`, and `support_multi_bed_types` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3825-3830` defines `support_multi_bed_types` as a scalar bool defaulting to `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5821-5849` defines `wipe_tower_type` plus the three adjacent scalar wipe-tower bool options and their defaults.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` serializes full config key/value pairs into G-code config output.

## Current Ares Gap

Ares already has metadata and registry coverage for these keys, but concrete G-code output stops at `filament_stamping_distance` and then jumps to filament color exports. The five wipe-tower GCodeConfig options are still not consumed by the Ares header/config output. `crates/ares-core/src/gcode_header.rs` and `crates/ares-core/src/options/filament_config_export.rs` are also close to the 400 LOC guard, so continuing to append code there would violate the project module-splitting rule soon.

## Behavior

Add a narrow Ares header/config export for:

- `wipe_tower_type`: accepts the existing Orca keys `"type1"` and `"type2"` and emits the configured key as `; wipe_tower_type = type1` or `; wipe_tower_type = type2`.
- `purge_in_prime_tower`: accepts a scalar boolean and emits `; purge_in_prime_tower = 1` or `0`.
- `enable_filament_ramming`: accepts a scalar boolean and emits `; enable_filament_ramming = 1` or `0`.
- `tool_change_on_wipe_tower`: accepts a scalar boolean and emits `; tool_change_on_wipe_tower = 1` or `0`.
- `support_multi_bed_types`: accepts a scalar boolean and emits `; support_multi_bed_types = 1` or `0`.

Missing options emit no config line. Invalid values return `SliceError::InvalidInput` naming the offending key. Validation must still run before BTT thumbnail header suppression, matching the existing `format_header` behavior that calls `options.filament_config_exports()` before deciding whether lines are appended.

Preserve upstream-adjacent order in the Ares config header:

1. `filament_stamping_loading_speed`
2. `filament_stamping_distance`
3. `wipe_tower_type`
4. `purge_in_prime_tower`
5. `enable_filament_ramming`
6. `tool_change_on_wipe_tower`
7. `support_multi_bed_types`
8. `filament_colour`

Refactor the config-header append sequence out of `gcode_header.rs` into a small dedicated module so touched Rust files stay under 400 LOC and future option-consumption slices have a focused boundary. Do not change emitted header lines except for adding the five new config exports.

## Deferred Behavior

This slice does not implement wipe tower geometry, wipe tower type selection in path planning, toolchange travel to tower, single-extruder multimaterial priming, `has_wipe_tower` placeholder truth, toolchange count computation, support for multi-bed UI behavior, or WipeTower2 ramming/purge execution. Those remain separate source-cited slices against `GCode.cpp` and `GCode/WipeTower2.cpp`.

## Docs Impact

Update `docs/roadmap.md` after implementation review with a source-cited runtime-slice entry for the five consumed wipe-tower config header options. No user-facing CLI/API documentation changes are required because this slice only makes already accepted Orca config keys visible in existing G-code header output and does not add CLI flags, public API surface, or new user workflows.

## Acceptance Criteria

- RED: a focused `cargo nextest run -p ares-core wipe_tower_config_header_gcode` fails before production changes because the five new config header lines are absent and invalid values are not rejected.
- GREEN: the same focused nextest command passes after implementation.
- Adjacent header tests pass with `cargo nextest run -p ares-core filament_stamping_gcode wipe_tower_config_header_gcode`.
- Full verification uses `cargo nextest run --workspace` and avoids the legacy Cargo test runner.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust LOC checks pass.
- `gcode_header.rs`, the new config-header module, `filament_config_export.rs`, and any touched test module stay at or below 400 LOC.
