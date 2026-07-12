# Consume `before_layer_change_gcode` Design

## Goal

Port OrcaSlicer's `before_layer_change_gcode` runtime behavior into Ares so the already registered option affects generated G-code at each layer transition. This is concrete G-code generation behavior, not new option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1294` defines `before_layer_change_gcode` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1110-1118` defines `before_layer_change_gcode` as a multiline string option with default empty string and tooltip: inserted at every layer change before the Z lift.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4498-4506` checks `m_config.before_layer_change_gcode.value`, injects `layer_num`, `layer_z`, and `max_layer_z`, processes the custom G-code through the placeholder parser, and appends it before `change_layer(print_z)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11115` lists `before_layer_change_gcode` with placeholders `layer_num`, `layer_z`, and `max_layer_z`.
- `OrcaSlicer/resources/profiles/*/machine/*.json` includes common `before_layer_change_gcode` templates using legacy bracket placeholders such as `[layer_z]` and `[layer_num]`, for example Lulzbot, Anycubic, Elegoo, Creality, and Snapmaker profiles.

## Ares Boundary

- Add a string-only runtime accessor for `before_layer_change_gcode` in `crates/ares-core/src/options/custom_gcode.rs`.
- Extend `crates/ares-core/src/gcode_placeholders.rs` with a focused layer custom-G-code renderer for `before_layer_change_gcode`.
- Wire rendered `before_layer_change_gcode` in `crates/ares-core/src/gcode.rs` inside the layer loop after Ares emits the `;LAYER_CHANGE`, `;LAYER`, and `;Z` markers and before `writer.travel_to_z_with_comment(...)`.
- Add focused runtime and G-code tests beside the existing custom-G-code tests.

## Behavior

- If `before_layer_change_gcode` is absent or `""`, generated G-code remains unchanged except for normal option-count comments when the empty option is explicitly provided.
- `before_layer_change_gcode` accepts only JSON strings. A non-string value returns `SliceError::InvalidInput` and mentions `before_layer_change_gcode`.
- A non-empty template is emitted once for each generated layer.
- For each layer, the rendered custom G-code appears after Ares' layer marker comments and before the Z travel command.
- The renderer replaces both `{layer_num}` and `[layer_num]` with Orca's one-based value for the upcoming layer, matching `m_layer_index + 1` before Orca increments the layer index.
- The renderer replaces both `{layer_z}` and `[layer_z]` with the layer's un-offset `print_z` using Ares' existing G-code decimal formatting.
- The renderer replaces both `{max_layer_z}` and `[max_layer_z]` with the maximum un-offset `print_z` seen through the current layer, using Ares' existing G-code decimal formatting.
- Unknown brace and bracket placeholders are preserved unchanged.
- A trailing newline is added when the rendered template does not already end with `\n`.
- Docs impact: no user docs or option metadata changes are required; this slice only consumes an already registered runtime option.

## Explicit Deferrals

- Do not add new option registry entries or metadata.
- Do not implement `layer_change_gcode`, `time_lapse_gcode`, `machine_end_gcode`, or filament custom G-code in this slice.
- Do not implement a general Orca placeholder parser or expression evaluator; this slice performs exact scalar replacement for the cited layer placeholders only.
- Do not implement extra placeholders beyond `layer_num`, `layer_z`, and `max_layer_z`.
- Do not change Z-offset behavior: placeholder values use upstream-style un-offset `print_z`, while the actual Z travel command continues to use `print_z + z_offset`.
- Do not add filesystem, terminal, UI, OpenGL, or WASM-hostile behavior to `ares-core`.

## Acceptance Criteria

- A unit test proves `SliceOptions::before_layer_change_gcode()` accepts strings, defaults absent values to `""`, and rejects non-strings with an error mentioning `before_layer_change_gcode`.
- A G-code integration test proves a non-empty template is emitted before every layer Z travel command.
- A G-code integration test proves `{layer_num}`, `{layer_z}`, and `{max_layer_z}` are replaced with deterministic layer-specific values.
- A G-code integration test proves `[layer_num]`, `[layer_z]`, and `[max_layer_z]` are replaced with deterministic layer-specific values.
- A G-code integration test proves unknown brace and bracket placeholders remain unchanged.
- A G-code integration test proves absent and empty `before_layer_change_gcode` produce equivalent output after removing only the option-count comment.
- Existing `file_start_gcode`, `machine_start_gcode`, and auxiliary fan tests still pass.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the 400 LOC gate pass.
