# Consume Z Offset Placeholder Design

## Goal

Port the OrcaSlicer `z_offset` machine-start placeholder slice into Ares so the existing `z_offset` option reaches concrete `machine_start_gcode` output. This is a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Upstream Boundary

Pinned upstream revision: OrcaSlicer `main` commit `a40979182684e500ff5c3c3ec920c0e6d44fcb66`.

- `OrcaSlicer/src/libslic3r/GCode.cpp:3042-3047` at that commit registers machine-start placeholder values including `max_print_height`, `z_offset`, `model_name`, `plate_number`, `plate_name`, and `first_layer_height`; line 3043 is `this->placeholder_parser().set("z_offset", new ConfigOptionFloat(m_config.z_offset));`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3115-3134` at that commit processes `print.config().machine_start_gcode.value` through `placeholder_parser_process("machine_start_gcode", ..., initial_extruder_id)` and writes the rendered `machine_start_gcode` with `file.writeln(machine_start_gcode)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5982-5990` at that commit defines `z_offset` as `coFloat`, labels it `Z offset`, documents that it is added to or subtracted from output G-code Z coordinates, and defaults it to `0`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1479-1483` and `PrintConfig.hpp:1619-1622` at that commit place `z_offset` on `PrintConfig`, which derives from `GCodeConfig` through `PRINT_CONFIG_CLASS_DERIVED_DEFINE`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11261-11264` at that commit leaves `machine_start_gcode` with an empty custom-placeholder specific set, so global placeholders registered in `GCode.cpp` are available there.

## Ares Destination Boundary

- Reuse the existing `SliceOptions::z_offset()` parser in `crates/ares-core/src/options.rs`. It already accepts finite numeric and numeric-string values, defaults to `0.0`, and rejects invalid values with `SliceError::InvalidInput`.
- Render `[z_offset]` in `crates/ares-core/src/gcode_placeholders.rs::machine_start_gcode` using the existing `format_placeholder_number` helper used by other machine-start numeric placeholders.
- Add focused G-code tests under `crates/ares-core/src/tests/` and register the module in `crates/ares-core/src/tests/mod.rs`.
- Update `docs/roadmap.md` with this consumed option runtime slice after implementation review.

## Included Behavior

1. `machine_start_gcode` containing `[z_offset]` renders the configured `z_offset` value before the first layer.
2. Missing `z_offset` renders Orca's default `0`.
3. Negative configured values render as negative numbers, matching Orca's documented compensation use case.
4. Numeric-string values already accepted by Ares' `z_offset` parser render in machine-start output.
5. `[z_offset]` composes with existing machine-start placeholders such as `[retract_length]`, `[num_extruders]`, and `[total_layer_count]`.
6. `[z_offset]` remains literal in `layer_change_gcode`; this slice only covers Orca's machine-start global placeholder registration path.
7. Existing generated Z-move offset behavior remains unchanged.

## Deferred Behavior

- Brace expression syntax such as `{z_offset}` or arithmetic expressions involving `z_offset`.
- Adjacent `GCode.cpp` placeholders from the same block: `max_print_height`, `model_name`, `plate_number`, `plate_name`, `first_layer_height`, temperature vectors, and bed/chamber placeholders.
- Full Orca `PlaceholderParser` parity.
- Exact Orca config serialization punctuation beyond existing Ares numeric placeholder formatting.
- Any changes to layer Z movement logic, first-layer height resolution, model metadata extraction, plate metadata, or UI/runtime behavior.

## Acceptance Criteria

- RED tests fail before production code because `[z_offset]` remains literal in machine-start output.
- GREEN tests pass after adding the replacement.
- Focused verification uses `cargo nextest run -p ares-core machine_start_z_offset`.
- Existing `z_offset_gcode` tests continue to cover validation and generated Z-move offset behavior.
- Full verification uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
- No new dependencies are added.
- All touched Rust files stay at or below 400 LOC.
- `ares-core` remains platform-neutral and WASM-compatible, with no file I/O, terminal, UI, or OpenGL behavior.

## Test Plan

Add tests named with the `machine_start_z_offset` prefix:

- `machine_start_z_offset_renders_configured_value`: `machine_start_gcode = ";ZOFF [z_offset]"` and `z_offset = 0.15` render `;ZOFF 0.15` before `;LAYER_CHANGE`.
- `machine_start_z_offset_defaults_to_zero`: missing `z_offset` renders `;ZOFF 0`.
- `machine_start_z_offset_renders_negative_value`: `z_offset = -0.05` renders `;ZOFF -0.05`.
- `machine_start_z_offset_accepts_numeric_string_and_composes`: `";START [z_offset] [retract_length] [num_extruders] [total_layer_count]"`, `z_offset = "0.2"`, `retraction_length = 1.1`, and two nozzle diameters render `;START 0.2 1.1 2 2`.
- `machine_start_z_offset_does_not_expand_in_layer_change_scope`: `layer_change_gcode = ";LC [z_offset] [layer_num]"` preserves `[z_offset]` while expanding `[layer_num]`.

## Documentation Impact

Update `docs/roadmap.md` with a short completed-slice note naming the upstream `GCode.cpp`, `PrintConfig.cpp`, and `PrintConfig.hpp` boundaries and the deferred adjacent behavior.
