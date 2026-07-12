# Consume Wipe Tower Toolchange Placeholders Design

## Source Boundary

Port the machine start/end custom G-code placeholder slice from `OrcaSlicer/src/libslic3r/GCode.cpp:2858-2861`:

- `has_wipe_tower`
- `has_single_extruder_multi_material_priming`
- `total_toolchanges`

Orca installs these into the placeholder parser for start/end G-code so custom startup logic can decide whether to prime or perform final filament pulls when no wipe tower is available.

## Rust Destination

Implement the currently truthful Ares runtime values in `crates/ares-core/src/gcode_machine_start_placeholders.rs`, which owns machine-start custom G-code placeholder rendering for the platform-neutral core.

Add focused integration coverage under `crates/ares-core/src/tests/` and register it from `crates/ares-core/src/tests/mod.rs`.

## Included Behavior

1. `[has_wipe_tower]` renders as `0` in `machine_start_gcode`.
2. `[has_single_extruder_multi_material_priming]` renders as `0` in `machine_start_gcode`.
3. `[total_toolchanges]` renders as `0` in `machine_start_gcode`.
4. These placeholders compose with existing machine-start placeholders such as `[num_extruders]` and `[total_layer_count]`.
5. A multi-nozzle configuration still renders `[num_extruders]` from the current configuration while the wipe-tower/toolchange placeholders remain `0`, because Ares has no active wipe tower or tool-ordering planner yet.
6. These placeholders remain literal in `layer_change_gcode`; this slice does not widen non-start custom G-code scopes.

## Deferred Behavior

- Full wipe tower generation and `WipeTowerType` selection.
- `single_extruder_multi_material_priming` behavior.
- Tool ordering, wipe tower data, and nonzero `number_of_toolchanges`.
- Physical extruder/hotend maps and `is_extruder_used`.
- Machine-end placeholder rendering parity; this slice only touches the existing Ares machine-start placeholder renderer.
- Custom G-code expression evaluation for conditions such as `{if total_toolchanges > 0}`.

## Acceptance Criteria

- RED: after adding focused tests but before implementation, `cargo nextest run -p ares-core wipe_tower_placeholders` fails because machine-start output keeps at least one new placeholder literal.
- GREEN: after implementation, `cargo nextest run -p ares-core wipe_tower_placeholders` passes.
- Adjacent placeholder tests pass with `cargo nextest run -p ares-core initial_extruder_placeholders_gcode num_extruders_gcode total_layer_count_gcode`.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.

## Safety

This is an additive placeholder-rendering slice in `ares-core`. It does not add dependencies, file I/O, terminal behavior, UI, OpenGL, or platform-specific code. The output values are current-state invariants of the active Ares pipeline: no wipe tower exists and no toolchange planner emits toolchange counts.
