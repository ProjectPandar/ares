# Consume first-layer center without wipe tower placeholder

## Goal

Render OrcaSlicer's `[first_layer_center_no_wipe_tower]` machine-start custom G-code placeholder from Ares' planned first-layer print geometry instead of leaving it literal.

This is a concrete `libslic3r` rewrite slice. It consumes existing first-layer slice geometry in startup G-code behavior and does not add option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2938-2945` builds `bbox_wo_wt` by merging every print object's `get_first_layer_bbox(...)`, computes `bbox_wo_wt.center()`, and registers `first_layer_center_no_wipe_tower` as a two-float placeholder.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082` processes `print.config().machine_start_gcode.value` through the placeholder parser before writing startup G-code.

## Ares Boundary

- `crates/ares-core/src/gcode_first_layer_print_placeholders.rs` already computes first-layer print bounds from `LayerPrintPaths`.
- `crates/ares-core/src/gcode_adaptive_bed_mesh.rs` already creates first-layer print placeholders for the machine-start placeholder path.
- `crates/ares-core/src/gcode_machine_start_placeholders.rs` renders machine-start placeholders.
- Runtime tests extend `crates/ares-core/src/tests/first_layer_print_placeholders_gcode.rs`; helper tests remain in `crates/ares-core/src/gcode_first_layer_print_placeholders.rs`.

## Included Behavior

1. `[first_layer_center_no_wipe_tower]` in `machine_start_gcode` renders as `x,y`.
2. The value is computed as the midpoint of the first-layer print geometry bounds already used for `[first_layer_print_min]`, `[first_layer_print_max]`, and `[first_layer_print_size]`.
3. The helper computes asymmetric geometry centers correctly, not only the centered default model case.
4. Empty first-layer print geometry renders an empty string for this placeholder, matching the existing empty behavior of first-layer print min/max/size placeholders.
5. The placeholder composes with existing first-layer print machine-start placeholders.
6. `[first_layer_center_no_wipe_tower]` remains literal in `layer_change_gcode`; this slice does not widen placeholder scope beyond machine-start G-code.

## Deferred Behavior

- Orca's exact per-`PrintObject::get_first_layer_bbox` object-bbox model is deferred until Ares has an equivalent active `PrintObject` first-layer bbox boundary. This slice uses the current first-layer print path bounds as the Rust destination boundary because that is the available planned first-layer geometry in `ares-core`.
- Wipe tower geometry generation and subtraction. Ares currently has no wipe tower in the active slicing pipeline, so "no wipe tower" is represented by the existing first-layer print geometry.
- Support-object-specific, object-instance-specific, sequential-object, calibration, and multi-material differences in first-layer object bboxes.
- Full Orca placeholder parser parity, expression evaluation, conditionals, vector indexing, and non-machine-start placeholder scopes.
- New dependencies, new crates, public API changes, file I/O, terminal behavior, UI behavior, OpenGL behavior, or WASM-incompatible behavior.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core first_layer_center_no_wipe_tower` fails before implementation because `[first_layer_center_no_wipe_tower]` remains literal in `machine_start_gcode`.
- After implementation, the same focused nextest command passes.
- Adjacent first-layer and adaptive mesh placeholder tests pass with `cargo nextest run -p ares-core first_layer_print_placeholders adaptive_bed_mesh_gcode`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - `git diff --cached --check`
  - touched Rust LOC guard with every touched Rust file at or below 400 LOC

## Test Cases

- `first_layer_center_no_wipe_tower_renders_default_first_layer_center`: default first-layer print geometry renders `;CENTER 0,0` before the first `;LAYER_CHANGE`.
- `first_layer_center_no_wipe_tower_composes_with_first_layer_bounds`: machine-start G-code can render min, center, max, and size in one line.
- `first_layer_center_no_wipe_tower_stays_literal_in_layer_change_scope`: layer-change custom G-code keeps `[first_layer_center_no_wipe_tower]` literal while still rendering `[layer_num]`.
- `asymmetric_first_layer_paths_render_center_from_bounds`: a direct helper unit test constructs asymmetric first-layer paths and expects the center midpoint of their bounds.
- `empty_first_layer_paths_render_empty_placeholder_strings`: the existing empty helper test is extended to cover the center placeholder.

## Docs Impact

No user-facing documentation update is required because the repository does not currently have a dedicated placeholder reference document. The source-cited SDD spec, plan, and regression tests document the behavior.

## Workflow Completion

After implementation acceptance, the active user objective and `$sdd-workflow` require a Lore-protocol commit and push to `origin/codex/consume-slicing-options`. That repository side effect is part of workflow completion, not a G-code behavior acceptance criterion.

## Safety

The implementation is a narrow in-memory geometry-derived string replacement through the existing machine-start placeholder chain. It does not alter geometry generation, path planning, temperature commands, fan commands, tool changes, filesystem behavior, terminal behavior, UI behavior, OpenGL behavior, or platform compatibility.
