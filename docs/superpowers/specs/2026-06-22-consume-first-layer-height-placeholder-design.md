# Consume First Layer Height Placeholder Design

## Goal

Port the OrcaSlicer `first_layer_height` machine-start placeholder slice into Ares so the existing `initial_layer_print_height` option reaches concrete `machine_start_gcode` output. This is a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Upstream Boundary

Pinned upstream revision: OrcaSlicer submodule commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`.

- `OrcaSlicer/src/libslic3r/GCode.cpp:3009-3014` registers machine-start placeholder values including `max_print_height`, `z_offset`, `model_name`, `plate_number`, `plate_name`, and `first_layer_height`; line 3014 is `this->placeholder_parser().set("first_layer_height", new ConfigOptionFloat(m_config.initial_layer_print_height.value));`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3101` processes `print.config().machine_start_gcode.value` through `placeholder_parser_process("machine_start_gcode", ..., initial_extruder_id)` and writes the rendered `machine_start_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1527-1529` places `initial_layer_print_height` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3264-3270` defines `initial_layer_print_height` as `coFloat`, labels it `First layer height`, sets min `0`, and defaults it to `0.2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11042-11043` defines the custom-placeholder-facing `first_layer_height` name as a float placeholder.

## Ares Destination Boundary

- Render `[first_layer_height]` in `crates/ares-core/src/gcode_placeholders.rs::machine_start_gcode`.
- Read the existing `initial_layer_print_height` value from `SliceOptions` without adding new option metadata or modifying `crates/ares-core/src/options.rs`, which is already at the 400 LOC limit.
- Use the existing option registry default for `initial_layer_print_height` (`0.2`) when the key is absent.
- Use the existing `format_placeholder_number` formatting helper used by other machine-start numeric placeholders.
- Add focused G-code tests under `crates/ares-core/src/tests/` and register the module in `crates/ares-core/src/tests/mod.rs`.
- Update `docs/roadmap.md` with this consumed option runtime slice after implementation review.

## Included Behavior

1. `machine_start_gcode` containing `[first_layer_height]` renders the configured `initial_layer_print_height` before the first layer.
2. Missing `initial_layer_print_height` renders Orca's default `0.2`.
3. Numeric-string values already preserved by Ares input normalization render in machine-start output.
4. `[first_layer_height]` composes with existing machine-start placeholders such as `[z_offset]`, `[retract_length]`, `[num_extruders]`, and `[total_layer_count]`.
5. `[first_layer_height]` remains literal in `layer_change_gcode`; this slice only covers Orca's machine-start placeholder registration path.
6. Invalid non-finite, non-numeric, zero, and negative `initial_layer_print_height` values fail before G-code output with `SliceError::InvalidInput`.

## Deferred Behavior

- Moving Ares layer planning from `initial_layer_height()` to Orca `initial_layer_print_height`; existing generated layer heights remain unchanged in this slice.
- Supporting `initial_layer_print_height` percentages such as `"50%"`; Ares legacy normalization currently drops percentage forms for this key.
- Brace expression syntax such as `{first_layer_height}` or arithmetic expressions involving first-layer height.
- Adjacent `GCode.cpp` placeholders from the same block: `max_print_height`, `model_name`, `plate_number`, `plate_name`, temperature vectors, bed/chamber placeholders, and `is_all_bbl_filament`.
- Full Orca `PlaceholderParser` parity and exact Orca config serialization punctuation beyond existing Ares numeric placeholder formatting.

## Acceptance Criteria

- RED tests fail before production code because `[first_layer_height]` remains literal in machine-start output.
- GREEN tests pass after adding the replacement.
- Focused verification uses `cargo nextest run -p ares-core machine_start_first_layer_height`.
- Existing `z_offset_placeholder_gcode` and `total_layer_count_gcode` tests continue to pass.
- Full verification uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a touched Rust LOC guard.
- No new dependencies are added.
- All touched Rust files stay at or below 400 LOC.
- `ares-core` remains platform-neutral and WASM-compatible, with no file I/O, terminal, UI, or OpenGL behavior.

## Test Plan

Add tests named with the `machine_start_first_layer_height` prefix:

- `machine_start_first_layer_height_renders_configured_value`: `machine_start_gcode = ";FLH [first_layer_height]"` and `initial_layer_print_height = 0.28` render `;FLH 0.28` before `;LAYER_CHANGE`.
- `machine_start_first_layer_height_defaults_to_orca_value`: missing `initial_layer_print_height` renders `;FLH 0.2`.
- `machine_start_first_layer_height_accepts_numeric_string_and_composes`: `";START [first_layer_height] [z_offset] [retract_length] [num_extruders] [total_layer_count]"`, `initial_layer_print_height = "0.24"`, `z_offset = 0.05`, `retraction_length = 1.1`, and two nozzle diameters render `;START 0.24 0.05 1.1 2 2`.
- `machine_start_first_layer_height_does_not_expand_in_layer_change_scope`: `layer_change_gcode = ";LC [first_layer_height] [layer_num]"` preserves `[first_layer_height]` while expanding `[layer_num]`.
- `machine_start_first_layer_height_rejects_invalid_values`: invalid `initial_layer_print_height` values `0`, `-0.1`, `"abc"`, and `["0.2"]` return `SliceError::InvalidInput`.

## Documentation Impact

Update `docs/roadmap.md` with a short completed-slice note naming the upstream `GCode.cpp`, `PrintConfig.cpp`, and `PrintConfig.hpp` boundaries and the deferred adjacent behavior.
