# Consume Non-Support Tool Placeholders Design

## Source Boundary

This slice ports the adjacent non-support machine-start placeholder boundary from `OrcaSlicer/src/libslic3r/GCode.cpp` and `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp`:

- `GCode.cpp:2682-2693` initializes `initial_non_support_extruder_id` and sizes `first_non_support_filaments` to `print.config().nozzle_diameter.size()` with `-1` entries.
- `GCode.cpp:2717` and `GCode.cpp:2751` call `ToolOrdering::cal_non_support_filaments` to find the first non-support filament/tool values.
- `GCode.cpp:2769-2771` falls `initial_non_support_extruder_id` back to `initial_extruder_id` when no separate non-support tool was found.
- `GCode.cpp:2813-2820` remaps and registers `first_non_support_tools`, `first_non_support_filaments`, `initial_no_support_tool`, `initial_no_support_extruder`, and `initial_no_support_hotend`.
- `ToolOrdering.cpp:974-1018` defines the support-aware source behavior behind `cal_non_support_filaments`.

## Rust Destination Boundary

Implement the slice in `ares-core` only:

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders the non-support tool placeholders during `machine_start_gcode` placeholder replacement.
- `crates/ares-core/src/tests/non_support_tool_placeholders_gcode.rs` covers rendered G-code through the public async `slice` API.
- `crates/ares-core/src/tests/mod.rs` registers the new focused test module.

No CLI, WASM adapter, filesystem, UI, OpenGL, native-only behavior, new dependencies, or option metadata additions are included.

## Current Ares Runtime Mapping

Ares does not yet have Orca's support-material tool ordering, `ToolOrdering::cal_non_support_filaments`, `physical_extruder_map`, or separate hotend mapping. It currently emits sliced geometry with the initial tool `0` and does not generate support material as a separate tool path.

For this slice, Ares renders the current non-support tool state as:

- `first_non_support_tools`: the same comma-separated vector as the current `[first_tools]` placeholder.
- `first_non_support_filaments`: the same comma-separated vector as the current `[first_filaments]` placeholder.
- `initial_no_support_tool`: `0`.
- `initial_no_support_extruder`: `0`.
- `initial_no_support_hotend`: `0`.

Examples:

- Default single-extruder print: all scalar no-support placeholders render `0`, and both vector placeholders render `0`.
- `nozzle_diameter = ["0.4", "0.6", "0.8"]`: both no-support vector placeholders render `0,-1,-1`, matching the current initial tool plus unused configured extruder slots.

## Included Behavior

1. `[first_non_support_tools]` renders in `machine_start_gcode`.
2. `[first_non_support_filaments]` renders in `machine_start_gcode`.
3. `[initial_no_support_tool]` renders in `machine_start_gcode`.
4. `[initial_no_support_extruder]` renders in `machine_start_gcode`.
5. `[initial_no_support_hotend]` renders in `machine_start_gcode`.
6. The vector placeholders compose with existing `[num_extruders]`, `[first_tools]`, `[initial_tool]`, and `[total_layer_count]` replacements.
7. These no-support placeholders remain literal in `layer_change_gcode`; this slice does not broaden the layer-change placeholder scope.

## Deferred Behavior

- Orca `ToolOrdering::cal_non_support_filaments` parity.
- Actual support generation, support material/interface material selection, and support-vs-model tool routing.
- `filament_is_support`, `filament_map`, and `physical_extruder_map` effects on the no-support vectors.
- Real hotend mapping through `hotend_id_for_gcode_placeholder`.
- `current_hotend`, runtime toolchange updates, wipe tower priming effects, and nonzero toolchange behavior.
- Placeholder expression/index evaluation beyond the existing literal replacement path.

## Acceptance Criteria

- A RED nextest run proves the new machine-start placeholder tests fail while these placeholders are still literal.
- A GREEN nextest run proves the default single-extruder case renders `0` for both vectors and all three scalar no-support placeholders.
- A GREEN nextest run proves a three-nozzle configuration renders `0,-1,-1` for both no-support vector placeholders while `[num_extruders]` remains `3`.
- A GREEN nextest run proves layer-change G-code leaves the no-support placeholders literal.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, diff checks, and the touched Rust LOC guard.

## Safety And Rollback

The change is additive placeholder replacement in platform-neutral `ares-core`. Rollback is removing the five replacements, test module registration, test file, and this spec/plan pair. No persisted data, external services, dependencies, public crate boundaries, or platform-specific behavior are changed.
