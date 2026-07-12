# Consume Retract Length Placeholder Design

## Goal

Port the OrcaSlicer `retract_length` machine-start placeholder slice into Ares so an existing `retraction_length` option reaches concrete `machine_start_gcode` output. This is a source-cited `libslic3r` rewrite slice, not a new Ares-owned pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2888-2896` registers global start-G-code placeholders, including `num_extruders` and `retract_length`; the `retract_length` value is `new ConfigOptionFloats(print.config().retraction_length)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3115-3134` processes `print.config().machine_start_gcode.value` through `placeholder_parser_process("machine_start_gcode", ..., initial_extruder_id)` and writes the rendered `machine_start_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5156` defines `retraction_length` as `coFloats` with default `{ 0.8 }`, millimeter units, and zero meaning retraction is disabled.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11261-11264` leaves `machine_start_gcode` with an empty custom-placeholder specific set, so global placeholders registered in `GCode.cpp` are available there.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:906-944` expands legacy vector placeholders by using `current_extruder_id`, falling back to index `0` when out of range.
- `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:1099-1119` expands no-index `coFloats` expressions by using index `0` for single-value vectors, otherwise the current extruder id.

## Ares Destination Boundary

- Add a small `SliceOptions` accessor in `crates/ares-core/src/options/layer_change_retraction.rs` that parses the first/current `retraction_length` value with Orca's `0.8` default.
- Do not reuse `LayerChangeRetraction`, because that structure intentionally returns disabled state when `retract_when_changing_layer` is false or `retraction_length` is zero. Orca's placeholder exposes the configured `retraction_length` value independently from whether generated retraction moves are enabled.
- Render `[retract_length]` in `crates/ares-core/src/gcode_placeholders.rs::machine_start_gcode` using the existing number formatting helper used by other machine-start placeholders.
- Add focused G-code tests under `crates/ares-core/src/tests/` and register the module in `crates/ares-core/src/tests/mod.rs`.
- Update `docs/roadmap.md` with this consumed option runtime slice after implementation review.

## Included Behavior

1. `machine_start_gcode` containing `[retract_length]` renders the first/current configured `retraction_length` value before the first layer.
2. Missing `retraction_length` renders Orca's default `0.8`.
3. A configured zero value renders `0` in `machine_start_gcode`; it is not replaced by disabled layer-change retraction state.
4. Scalar, string/vector numeric forms already accepted by Ares' numeric vector parser remain accepted for the placeholder.
5. `[retract_length]` composes with existing machine-start placeholders such as `[num_extruders]` and `[total_layer_count]`.
6. `[retract_length]` remains literal in `layer_change_gcode`; this slice only covers Orca's machine-start global placeholder registration path.
7. Invalid non-finite or negative values continue to surface as `SliceError::InvalidInput` through the same boundary validation used by existing retraction option parsing.

## Deferred Behavior

- Explicit vector indexing such as `[retract_length_1]` and expression syntax such as `{retract_length[1]}`.
- Current-extruder selection beyond Ares' current initial extruder `0` scope.
- Tool-change placeholders for old/new retract lengths.
- Filament override routing, wipe tower behavior, multi-material priming, and tool ordering.
- Changes to generated retract/unretract moves; this slice only exposes the placeholder.
- Full Orca `PlaceholderParser` parity.

## Acceptance Criteria

- RED tests fail before production code because `[retract_length]` remains literal in machine-start output.
- GREEN tests pass after adding the accessor and replacement.
- Focused verification uses `cargo nextest run -p ares-core machine_start_retract_length`.
- Full verification uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
- No new dependencies are added.
- All touched Rust files stay at or below 400 LOC.
- `ares-core` remains platform-neutral and WASM-compatible, with no file I/O, terminal, UI, or OpenGL behavior.

## Test Plan

Add tests named with the `machine_start_retract_length` prefix:

- `machine_start_retract_length_renders_first_configured_value`: `machine_start_gcode = ";RETRACT [retract_length]"` and `retraction_length = [1.25, 9.0]` render `;RETRACT 1.25` before `;LAYER_CHANGE`.
- `machine_start_retract_length_defaults_to_orca_value`: missing `retraction_length` renders `;DEFAULT-RETRACT 0.8`.
- `machine_start_retract_length_renders_zero_value`: `retraction_length = 0` renders `;ZERO-RETRACT 0`, proving placeholder rendering is independent from generated retraction enablement.
- `machine_start_retract_length_composes_with_existing_placeholders`: `";START [retract_length] [num_extruders] [total_layer_count]"`, `retraction_length = "1.5,9"`, and two nozzle diameters render `;START 1.5 2 2`.
- `machine_start_retract_length_does_not_expand_in_layer_change_scope`: `layer_change_gcode = ";LC [retract_length] [layer_num]"` preserves `[retract_length]` while expanding `[layer_num]`.

## Documentation Impact

Update `docs/roadmap.md` with a short completed-slice note naming the upstream `GCode.cpp`, `PrintConfig.cpp`, and `PlaceholderParser.cpp` boundaries and the deferred adjacent behavior.
