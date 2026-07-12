# Consume Machine End G-code Design

## Purpose

Consume the existing `machine_end_gcode` option in concrete print-finish G-code output. This continues the source-cited OrcaSlicer custom G-code rewrite by turning an already-recorded option into runtime slicing behavior before any more option metadata is added.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1299` declares `machine_end_gcode` as a `ConfigOptionString` in `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1940-1947` defines `machine_end_gcode` as the advanced multiline "End G-code" option with Orca's default end template.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3433-3446` builds finish-time placeholder values and writes `machine_end_gcode` after filament end G-code and before progress/postamble output.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11114` lists custom placeholders for `machine_end_gcode`: `layer_num`, `layer_z`, `max_layer_z`, and `filament_extruder_id`.

## Ares Boundary

Destination files:

- `crates/ares-core/src/options/custom_gcode.rs` exposes `SliceOptions::machine_end_gcode()` as a string-only internal accessor with default `""`.
- `crates/ares-core/src/gcode_placeholders.rs` renders `machine_end_gcode` with direct finish placeholders.
- `crates/ares-core/src/gcode_finish.rs` is introduced to own print-finish commands. This keeps `crates/ares-core/src/gcode.rs` below the 400 LOC project limit while adding the new machine-end sequence.
- `crates/ares-core/src/gcode.rs` calls the finish helper after all layers have been emitted.
- `crates/ares-core/src/tests/custom_gcode_end.rs` is introduced for machine-start, file-start, and machine-end custom G-code tests. This keeps `crates/ares-core/src/tests/custom_gcode.rs` below the 400 LOC project limit.
- `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs` adds string-only accessor coverage for `machine_end_gcode`.

Current file sizes require a small focused split before adding behavior: `crates/ares-core/src/gcode.rs` is 399 LOC and `crates/ares-core/src/tests/custom_gcode.rs` is 389 LOC.

## Behavior

At print finish, Ares emits:

1. Chamber shutdown when currently emitted by Ares.
2. Exhaust fan completion command when currently emitted by Ares.
3. Auxiliary fan completion command when currently emitted by Ares.
4. Rendered `machine_end_gcode`.
5. `M2`.

This placement is the closest current Ares boundary to Orca's finish block: it emits the custom machine end template before the terminal postamble command and after print-body output. Ares has no filament end G-code, progress writer, or Orca postamble parity yet; those are explicitly deferred.

The renderer must:

- Return no output when `machine_end_gcode` is absent or `""`.
- Require the configured value to be a JSON string; non-string values return `SliceError::InvalidInput` mentioning `machine_end_gcode`.
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
- Do not implement Orca's default `machine_end_gcode` profile fallback; Ares remains no-op when the option is absent.
- Do not implement `filament_end_gcode`, `printing_by_object_gcode`, `machine_pause_gcode`, or toolchange custom G-code in this slice.
- Do not implement the full Orca placeholder parser, arithmetic, conditionals, or filament-indexed runtime state.
- Do not change generated G-code when `machine_end_gcode` is absent or empty, except for mechanical code movement that preserves output byte-for-byte after filtering the existing `; option_count =` line.
- Do not change existing `file_start_gcode`, `machine_start_gcode`, `before_layer_change_gcode`, `layer_change_gcode`, or `time_lapse_gcode` behavior.

## Acceptance Criteria

- `machine_end_gcode` appears after the last emitted print-body layer content.
- `machine_end_gcode` appears after current Ares completion shutdown commands and before `M2`.
- Brace and bracket direct finish placeholders render correctly for the existing square pyramid STL test fixture.
- Unknown placeholders, conditionals, and expression placeholders are preserved.
- Invalid non-string `machine_end_gcode` reaches `SliceError::InvalidInput`.
- Absent and empty `machine_end_gcode` produce identical output after filtering the existing `; option_count =` line.
- Existing custom G-code tests continue to pass after moving start/file tests into `custom_gcode_end.rs`.
- `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification

Run fresh verification before commit:

- `cargo fmt --check`
- `cargo test -p ares-core --lib machine_end_gcode`
- `cargo test -p ares-core --lib custom_gcode`
- `cargo test -p ares-core --lib custom_gcode_end`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `bad=0; while IFS= read -r -d '' f; do n=$(wc -l < "$f"); if [ "$n" -gt 400 ]; then printf '%s %s\n' "$n" "$f"; bad=1; fi; done < <(find crates/ares-core/src -name '*.rs' -print0); exit "$bad"`

## Documentation Impact

No user-facing documentation update is required beyond this spec and the implementation plan. This slice consumes an already accepted Orca option in runtime behavior and does not change CLI usage or public API shape.
