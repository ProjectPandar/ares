# Consume Filament End G-code Design

## Purpose

Consume the existing `filament_end_gcode` option in concrete print-finish G-code output. This continues converting recorded OrcaSlicer options into runtime slicing behavior before adding more option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1300` declares `filament_end_gcode` as `ConfigOptionStrings` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1958-1965` defines `filament_end_gcode` as the advanced multiline "End G-code" option for finishing a filament, with Orca's default `ConfigOptionStrings { " " }`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3433-3446` writes `filament_end_gcode` at print finish before `machine_end_gcode`, using finish placeholders and `filament_extruder_id`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11133` lists custom placeholders for `filament_end_gcode`: `layer_num`, `layer_z`, `max_layer_z`, and `filament_extruder_id`.

## Ares Boundary

Destination files:

- `crates/ares-core/src/options/custom_gcode.rs` exposes `SliceOptions::filament_end_gcode()` as an internal accessor that accepts a string or string array and returns the active first filament template with default `""`.
- `crates/ares-core/src/gcode_placeholders.rs` renders `filament_end_gcode` with the same finish placeholders as `machine_end_gcode`.
- `crates/ares-core/src/gcode_finish.rs` emits rendered `filament_end_gcode` before rendered `machine_end_gcode`.
- `crates/ares-core/src/tests/custom_gcode_end.rs` adds finish-time runtime tests for `filament_end_gcode`.
- `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs` adds accessor coverage.

Current relevant file sizes leave enough room for this narrow slice: `gcode_finish.rs` is 45 LOC, `gcode_placeholders.rs` is 144 LOC, `options/custom_gcode.rs` is 76 LOC, `tests/custom_gcode_end.rs` is 243 LOC, and all must remain at or below 400 LOC.

## Behavior

At print finish, Ares emits:

1. Chamber shutdown when currently emitted by Ares.
2. Exhaust fan completion command when currently emitted by Ares.
3. Auxiliary fan completion command when currently emitted by Ares.
4. Rendered `filament_end_gcode`.
5. Rendered `machine_end_gcode`.
6. `M2`.

The accessor must:

- Return `""` when `filament_end_gcode` is absent.
- Accept a JSON string as a compatibility shorthand for the active filament template.
- Accept a JSON array of strings and use the first entry as the active filament template.
- Treat an empty array as `""`.
- Reject non-string scalar values, arrays containing non-strings, and object/null values with `SliceError::InvalidInput` mentioning `filament_end_gcode`.

The renderer must:

- Return no output when the selected active template is `""`.
- Ensure non-empty output ends with the existing trailing-newline helper behavior.
- Replace direct finish placeholders in both syntaxes:
  - `{layer_num}` and `[layer_num]`
  - `{layer_z}` and `[layer_z]`
  - `{max_layer_z}` and `[max_layer_z]`
  - `{filament_extruder_id}` and `[filament_extruder_id]`
- Use current Ares finish values:
  - `layer_num`: number of generated layers.
  - `layer_z`: formatted print Z of the last generated layer, without Z offset.
  - `max_layer_z`: same last-layer print Z until Orca's max-layer accumulator is ported.
  - `filament_extruder_id`: `0` until filament/extruder runtime state is ported.
- Preserve unknown placeholders and Orca expressions/conditionals unchanged.

## Non-Goals

- Do not add option metadata or registry entries.
- Do not implement Orca's default `filament_end_gcode` profile fallback of `" "`; Ares remains no-op when the option is absent.
- Do not emit every configured filament end template in this slice. Ares currently lacks Orca's active filament, extruder, and single-extruder-multi-material finish state, so this slice consumes only the active first filament template with `filament_extruder_id = 0`.
- Do not implement `filament_start_gcode`, `change_filament_gcode`, `printing_by_object_gcode`, `machine_pause_gcode`, role-change custom G-code, or toolchange custom G-code.
- Do not implement the full Orca placeholder parser, arithmetic, conditionals, or filament-indexed runtime state.
- Do not change generated G-code when `filament_end_gcode` is absent or empty, except for the existing `; option_count =` metadata count.
- Do not change existing `file_start_gcode`, `machine_start_gcode`, layer custom G-code, `time_lapse_gcode`, or `machine_end_gcode` behavior.

## Acceptance Criteria

- `filament_end_gcode` appears after existing Ares completion shutdown commands and before `machine_end_gcode`.
- `filament_end_gcode` appears before `M2` when `machine_end_gcode` is absent.
- String and string-array option forms are accepted; the array form uses the first template.
- Empty array, absent option, and empty selected template produce no custom filament end output after filtering the existing `; option_count =` line.
- Brace and bracket direct finish placeholders render correctly for the existing square pyramid STL test fixture.
- Unknown placeholders, conditionals, and expression placeholders are preserved.
- Invalid non-string `filament_end_gcode` values reach `SliceError::InvalidInput`.
- Existing machine-end and custom G-code tests continue to pass.
- `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification

Run fresh verification before commit:

- `cargo fmt --check`
- `cargo test -p ares-core --lib filament_end_gcode`
- `cargo test -p ares-core --lib machine_end_gcode`
- `cargo test -p ares-core --lib custom_gcode_end`
- `cargo test -p ares-core --lib custom_gcode`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `bad=0; while IFS= read -r -d '' f; do n=$(wc -l < "$f"); if [ "$n" -gt 400 ]; then printf '%s %s\n' "$n" "$f"; bad=1; fi; done < <(find crates/ares-core/src -name '*.rs' -print0); exit "$bad"`

## Documentation Impact

No user-facing documentation update is required beyond this spec and the implementation plan. This slice consumes an already accepted Orca option in runtime behavior and does not change CLI usage or public API shape.
