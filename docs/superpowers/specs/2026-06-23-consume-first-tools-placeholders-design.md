# Consume First Tools Placeholders Design

## Source Boundary

This slice ports the machine-start placeholder boundary from `OrcaSlicer/src/libslic3r/GCode.cpp`:

- `GCode.cpp:2686-2693` creates `first_filaments` and sizes it to `print.config().nozzle_diameter.size()` with `-1` entries.
- `GCode.cpp:2717` and `GCode.cpp:2751` let `ToolOrdering::cal_non_support_filaments` fill the first filament vector from the print's tool ordering.
- `GCode.cpp:2798-2807` remaps the vector through `physical_extruder_map`.
- `GCode.cpp:2808-2809` registers that same integer vector as both `first_tools` and `first_filaments` for custom G-code placeholders.

## Rust Destination Boundary

Implement the slice in `ares-core` only:

- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders `[first_tools]` and `[first_filaments]` during `machine_start_gcode` placeholder replacement.
- `crates/ares-core/src/tests/first_tools_placeholders_gcode.rs` covers the rendered G-code behavior through the public async `slice` API.
- `crates/ares-core/src/tests/mod.rs` registers the new focused test module.

No CLI, WASM adapter, filesystem, UI, OpenGL, or native-only behavior changes are included.

## Current Ares Runtime Mapping

Ares does not yet have Orca's `ToolOrdering`, real multi-material tool changes, or `physical_extruder_map`. It currently emits all sliced geometry with the initial tool `0`.

For this slice, Ares renders the current first-tool vector as:

- `0` for the initial tool slot.
- `-1` for every additional configured nozzle/extruder slot that Ares does not yet use.
- The vector length equals the existing `[num_extruders]` machine-start placeholder value.

Examples:

- Default single-extruder print: `[first_tools]` and `[first_filaments]` render `0`.
- `nozzle_diameter = ["0.4", "0.6", "0.8"]`: both render `0,-1,-1`.

This keeps the placeholders concrete without inventing a multi-tool planner.

## Included Behavior

1. `[first_tools]` renders in `machine_start_gcode`.
2. `[first_filaments]` renders in `machine_start_gcode`.
3. Both placeholders render the same comma-separated integer vector, matching Orca's `GCode.cpp:2808-2809` aliasing.
4. The vector composes with existing `[num_extruders]`, `[initial_tool]`, and `[total_layer_count]` replacements.
5. `[first_tools]` and `[first_filaments]` remain literal in `layer_change_gcode`; this slice does not broaden the layer-change placeholder scope.

## Deferred Behavior

- Orca `ToolOrdering` parity, including real first-used tool selection across objects, regions, support, and custom tool changes.
- `physical_extruder_map` and filament remapping.
- `first_non_support_tools`, `first_non_support_filaments`, `initial_no_support_tool`, and `initial_no_support_extruder`.
- Wipe tower priming effects, single-extruder multi-material priming, nonzero toolchange counts, and runtime toolchange G-code.
- Placeholder expression/index evaluation beyond the existing literal replacement path.

## Acceptance Criteria

- A RED nextest run proves the new machine-start placeholder tests fail while placeholders are still literal.
- A GREEN nextest run proves `machine_start_gcode` renders `0` for the default single-extruder case.
- A GREEN nextest run proves a three-nozzle configuration renders `0,-1,-1` for both placeholders and still renders `[num_extruders]` as `3`.
- A GREEN nextest run proves layer-change G-code leaves `[first_tools]` and `[first_filaments]` literal.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, diff checks, and the touched Rust LOC guard.

## Safety And Rollback

The change is additive placeholder replacement in platform-neutral `ares-core`. Rollback is removing the two replacements, helper, test module registration, test file, and this spec/plan pair. No persisted data, external services, dependencies, or public crate boundaries are changed.
