# Consume max print Z placeholder

## Goal

Render OrcaSlicer's `[max_print_z]` machine-start custom G-code placeholder from Ares' actual planned print layers instead of leaving it literal.

This is a concrete `libslic3r` rewrite slice. It consumes existing slicing output in startup G-code behavior and does not add option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2912-2918` initializes `max_print_z` to `0`, scans every print object's layers, takes the maximum `Layer::print_z`, applies `std::ceil`, and registers `max_print_z` as a `ConfigOptionInt`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082` processes `print.config().machine_start_gcode.value` through the placeholder parser before writing startup G-code.

## Ares Boundary

- `crates/ares-core/src/gcode.rs` owns the in-memory `SlicingPipeline` to G-code formatting path and already has access to `pipeline.layers()`.
- `crates/ares-core/src/gcode_start_custom.rs` carries machine-start context into startup G-code formatting.
- `crates/ares-core/src/gcode_adaptive_bed_mesh.rs` forwards existing machine-start placeholder context.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders machine-start placeholders.
- Runtime tests live in a new `crates/ares-core/src/tests/max_print_z_placeholder_gcode.rs` module.

## Included Behavior

1. `[max_print_z]` in `machine_start_gcode` renders as the integer ceiling of the maximum planned `Layer::print_z` from `pipeline.layers()`.
2. The value is computed from planned slice state, not from `printable_height`, `z_offset`, or user-supplied text.
3. A default two-layer square-pyramid slice with max layer Z below `1.0` renders `[max_print_z]` as `1`.
4. A taller slice whose final planned print Z is above `1.0` renders `[max_print_z]` as the corresponding ceiling, for example `2` for a `1.2` final planned print Z.
5. `[max_print_z]` composes with existing machine-start placeholders such as `[max_print_height]` and `[total_layer_count]`.
6. `[max_print_z]` remains literal in `layer_change_gcode`; this slice does not widen placeholder scope beyond machine-start G-code.

## Deferred Behavior

- Orca's exact per-`PrintObject` layer scan is deferred until Ares has an equivalent `PrintObject` layer ownership model in the active slicing pipeline. This slice uses Ares' current global planned layer list as the destination boundary.
- Wipe tower, support-object, object-specific, sequential-object, and multi-material tool-order effects on maximum print Z.
- Other nearby Orca placeholders from the same `GCode.cpp` block, including `has_wipe_tower`, `total_toolchanges`, `current_hotend`, `first_tools`, `first_filaments`, `first_non_support_tools`, `first_non_support_filaments`, `initial_no_support_*`, `in_head_wrap_detect_zone`, and `first_layer_center_no_wipe_tower`.
- Full Orca placeholder parser parity, expression evaluation, conditionals, vector indexing, and non-machine-start placeholder scopes.
- New dependencies, new crates, public API changes, file I/O, terminal behavior, UI behavior, OpenGL behavior, or WASM-incompatible behavior.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core max_print_z_placeholder` fails before implementation because `[max_print_z]` remains literal in `machine_start_gcode`.
- After implementation, the same focused nextest command passes.
- Adjacent machine-start placeholder tests pass with `cargo nextest run -p ares-core total_layer_count_gcode initial_extruder_placeholders_gcode`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust LOC guard with every touched Rust file at or below 400 LOC

## Test Cases

- `machine_start_max_print_z_renders_ceiled_planned_layer_z`: a taller in-memory STL renders `;MAXZ 2` before the first `;LAYER_CHANGE`.
- `max_print_z_composes_with_machine_start_height_and_layer_count`: the default fixture renders a single machine-start line containing `[max_print_z]`, `[max_print_height]`, and `[total_layer_count]` as concrete values.
- `max_print_z_stays_literal_in_layer_change_scope`: layer-change custom G-code keeps `[max_print_z]` literal while still rendering `[layer_num]`.

## Docs Impact

No user-facing documentation update is required because the repository does not currently have a dedicated placeholder reference document. The source-cited SDD spec, plan, and regression tests document the behavior.

## Workflow Completion

After implementation acceptance, the active user objective and `$sdd-workflow` require a Lore-protocol commit and push to `origin/codex/consume-slicing-options`. That repository side effect is part of workflow completion, not a G-code behavior acceptance criterion.

## Safety

The implementation is a narrow in-memory data-flow and string replacement through the existing machine-start placeholder chain. It does not alter geometry generation, path planning, temperature commands, fan commands, tool changes, filesystem behavior, terminal behavior, UI behavior, OpenGL behavior, or platform compatibility.
