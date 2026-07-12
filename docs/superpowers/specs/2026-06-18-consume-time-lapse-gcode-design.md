# Consume Time Lapse G-code Design

## Purpose

Consume the existing `time_lapse_gcode` option in concrete layer-change G-code output. This continues the source-cited rewrite of OrcaSlicer custom layer G-code behavior and turns existing option metadata into runtime slicing output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1359` declares `time_lapse_gcode` as a `ConfigOptionString` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4304-4310` defines `time_lapse_gcode` as a multiline advanced string option with default empty string.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4533-4540` consumes non-empty `time_lapse_gcode` after `change_layer(print_z)` and before `layer_change_gcode` when `!is_BBL_Printer()`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11117` lists the supported custom placeholders for `timelapse_gcode`: `layer_num`, `layer_z`, and `max_layer_z`.
- `OrcaSlicer/resources/profiles/BBL/machine/fdm_bbl_3dp_002_common.json:382` shows real profile content using `time_lapse_gcode` with Orca conditionals and `{layer_z}`. This slice preserves unsupported conditionals and only replaces direct layer placeholders.

## Ares Boundary

Destination files:

- `crates/ares-core/src/options/custom_gcode.rs` exposes `SliceOptions::time_lapse_gcode()` as a string-only internal accessor with default `""`.
- `crates/ares-core/src/gcode_placeholders.rs` renders `time_lapse_gcode` using the same narrow direct layer placeholder replacement as `layer_change_gcode`.
- `crates/ares-core/src/gcode_layer_custom.rs` adds an after-Z helper that emits `time_lapse_gcode` before `layer_change_gcode`.
- `crates/ares-core/src/gcode.rs` calls the combined after-Z helper once per layer, keeping the core file at or below the 400 LOC project limit.
- `crates/ares-core/src/tests/custom_gcode.rs` adds focused behavior tests for `time_lapse_gcode`.

`crates/ares-core/src/gcode.rs` is currently 399 LOC, so implementation must not add a separate new call directly in that file. Move the post-Z custom G-code sequence into `gcode_layer_custom.rs` and have `gcode.rs` call a single after-Z helper.

## Behavior

For every generated layer:

1. Ares continues to emit `before_layer_change_gcode` before the Z travel command.
2. Ares emits the existing Z travel command.
3. Ares emits rendered `time_lapse_gcode`.
4. Ares emits rendered `layer_change_gcode`.
5. Ares then emits part cooling fan and auxiliary fan commands as it does today.

The renderer must:

- Return no output when `time_lapse_gcode` is absent or `""`.
- Require the configured value to be a JSON string; non-string values return `SliceError::InvalidInput` mentioning `time_lapse_gcode`.
- Ensure non-empty output ends with the existing trailing-newline helper behavior.
- Replace direct layer placeholders in both syntaxes:
  - `{layer_num}` and `[layer_num]`
  - `{layer_z}` and `[layer_z]`
  - `{max_layer_z}` and `[max_layer_z]`
- Use current Ares layer custom values:
  - `layer_num`: one-based layer number.
  - `layer_z`: formatted current layer print Z without Z offset.
  - `max_layer_z`: formatted current layer print Z until the upstream max-layer accumulator is ported.
- Preserve unknown placeholders and Orca expressions/conditionals unchanged.

## Non-Goals

- Do not add option metadata or registry entries.
- Do not implement the full Orca placeholder parser, arithmetic, conditionals, or filament-indexed placeholders.
- Do not implement Orca's BBL printer branch, traditional timelapse insertion paths, wipe-tower timelapse behavior, or `is_BBL_Printer()` classification in this slice.
- Do not change generated G-code when `time_lapse_gcode` is absent or empty.
- Do not change existing `before_layer_change_gcode`, `layer_change_gcode`, `file_start_gcode`, or `machine_start_gcode` behavior except for routing post-Z layer custom emission through the focused helper.

## Acceptance Criteria

- `time_lapse_gcode` appears after each layer Z travel line.
- `time_lapse_gcode` appears before `layer_change_gcode` when both options are non-empty.
- `time_lapse_gcode` appears before the first fan command or `; segment_count =` line for that layer.
- Brace and bracket direct layer placeholders render correctly on both generated layers in the existing square pyramid STL test fixture.
- Unknown placeholders, conditionals, and expression placeholders are preserved.
- Invalid non-string `time_lapse_gcode` reaches `SliceError::InvalidInput`.
- Absent and empty `time_lapse_gcode` produce identical output after filtering the existing `; option_count =` line.
- Existing custom G-code tests continue to pass.
- `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification

Run fresh verification before commit:

- `cargo fmt --check`
- `cargo test -p ares-core --lib time_lapse_gcode`
- `cargo test -p ares-core --lib custom_gcode`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `bad=0; while IFS= read -r -d '' f; do n=$(wc -l < "$f"); if [ "$n" -gt 400 ]; then printf '%s %s\n' "$n" "$f"; bad=1; fi; done < <(find crates/ares-core/src -name '*.rs' -print0); exit "$bad"`

## Documentation Impact

No user-facing documentation update is required beyond this spec and the implementation plan. This slice consumes an already accepted Orca option in runtime behavior and does not change CLI usage or public API shape.
