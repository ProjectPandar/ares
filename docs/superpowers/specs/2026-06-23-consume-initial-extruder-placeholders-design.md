# Consume initial extruder placeholders

## Goal

Render OrcaSlicer machine-start custom G-code placeholders for the current Ares initial tool state instead of leaving them literal in `machine_start_gcode`.

This is a concrete `libslic3r` rewrite slice: it consumes already-known custom G-code placeholder behavior in generated G-code output. It does not add new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2810-2811` registers `initial_tool` and `initial_extruder` from `initial_extruder_id`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2821` registers `current_extruder` from the same initial extruder before `machine_start_gcode` is processed.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2857` registers `current_object_idx` as `0` for the machine-start phase.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082` processes `print.config().machine_start_gcode.value` through the placeholder parser before writing startup G-code.

## Ares Boundary

- Destination module: `crates/ares-core/src/gcode_machine_start_placeholders.rs`.
- Runtime tests: `crates/ares-core/src/tests/initial_extruder_placeholders_gcode.rs`.
- Ares currently has a single initial print tool in the emitted G-code path. This slice renders `initial_tool`, `initial_extruder`, and `current_extruder` as `0`, matching Orca's fallback when no printable tool is found and Ares' existing first-extruder placeholder behavior.
- `current_object_idx` renders as `0` in machine-start scope, matching Orca's startup value before any by-object print loop.

## Included Behavior

1. `[initial_tool]` in `machine_start_gcode` renders `0`.
2. `[initial_extruder]` in `machine_start_gcode` renders `0`.
3. `[current_extruder]` in `machine_start_gcode` renders `0`.
4. `[current_object_idx]` in `machine_start_gcode` renders `0`.
5. These replacements compose with existing machine-start placeholders such as `[num_extruders]` and `[total_layer_count]`.
6. These placeholders remain literal in `layer_change_gcode`; this slice does not widen placeholder scope.

## Deferred Behavior

- Tool ordering, support/non-support initial tool selection, `first_tools`, `first_filaments`, `first_non_support_tools`, and `first_non_support_filaments`.
- `initial_no_support_tool`, `initial_no_support_extruder`, `initial_no_support_hotend`, and `current_hotend`.
- Physical extruder mapping, hotend mapping, toolchange processing, wipe tower behavior, multi-material priming, and updating `current_extruder` after tool changes.
- Sequential by-object printing and later updates of `current_object_idx` during `printing_by_object_gcode`.
- Full Orca placeholder expression parser parity and non-machine-start placeholder scopes.
- New dependencies, new crates, public API changes, file I/O, UI, terminal behavior, or WASM-incompatible behavior.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core initial_extruder_placeholders` fails before implementation because the machine-start placeholders remain literal.
- After implementation, the same focused nextest command passes.
- Adjacent machine-start placeholder tests pass with `cargo nextest run -p ares-core num_extruders_gcode total_layer_count_gcode`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust LOC guard with every touched Rust file at or below 400 LOC

## Test Cases

- `machine_start_initial_extruder_placeholders_render_zero_ids`: machine-start template containing all four placeholders renders `;TOOLS 0 0 0 0` before the first layer marker.
- `initial_extruder_placeholders_compose_with_existing_start_placeholders`: template containing `[initial_tool]`, `[current_extruder]`, `[num_extruders]`, and `[total_layer_count]` renders the expected single initial tool and effective nozzle count.
- `initial_extruder_placeholders_stay_literal_in_layer_change_scope`: layer-change template keeps these placeholders literal while still rendering `[layer_num]`.

## Safety

The implementation is a narrow string replacement in the existing machine-start placeholder rendering chain. It does not alter slicing geometry, path planning, temperature commands, fan commands, or automatic tool changes. Rollback is deleting the new replacements and focused tests.
