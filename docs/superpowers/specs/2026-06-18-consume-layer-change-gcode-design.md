# Consume Layer Change G-code Design

## Purpose

Consume the existing `layer_change_gcode` option in concrete layer-change G-code output. This is a source-cited rewrite slice of OrcaSlicer layer-change custom G-code behavior, not a new Ares pipeline feature and not another option-metadata-only milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1358` declares `layer_change_gcode` as a print configuration string.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4295-4303` defines the option label, multiline UI behavior, default empty string, and tooltip: inserted at every layer change after the Z lift.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4543-4554` consumes the option during layer changes after `change_layer(print_z)` has produced the Z movement and before the Bambu fan-speed-changing-layer marker.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:11116` lists layer custom G-code placeholders including `layer_num`, `layer_z`, and `max_layer_z`.
- `OrcaSlicer/resources/profiles/BBL/machine/fdm_bbl_3dp_002_common.json:381` uses `layer_change_gcode` with brace placeholders such as `{layer_num+1}` and `{layer_num}`. This slice supports the direct placeholders Ares can substitute now and preserves expression placeholders for a later parser slice.

## Ares Boundary

Destination files:

- `crates/ares-core/src/options/custom_gcode.rs` exposes `SliceOptions::layer_change_gcode()` as a string-only internal accessor with default `""`.
- `crates/ares-core/src/gcode_placeholders.rs` renders `layer_change_gcode` using the same narrow replacement style as `before_layer_change_gcode`.
- `crates/ares-core/src/gcode.rs` calls the renderer once per layer after the Z travel command and before part-cooling and auxiliary fan layer commands.
- `crates/ares-core/src/tests/custom_gcode.rs` owns custom G-code behavior tests, split out from the near-limit auxiliary fan test file.
- `crates/ares-core/src/tests/auxiliary_fan_gcode.rs` retains auxiliary fan tests only.
- `crates/ares-core/src/tests/mod.rs` registers the new custom G-code test module.

`crates/ares-core/src/gcode.rs` is currently near the 400 LOC project limit, so implementation must include a small source-scope split before adding the new call. The split should move layer custom G-code emission into a focused helper module rather than growing `gcode.rs`.

## Behavior

For every generated layer:

1. Ares continues to emit:
   - `;LAYER_CHANGE`
   - `;LAYER:<id>`
   - `;Z:<print_z>`
2. Ares continues to emit `before_layer_change_gcode` before the Z travel command.
3. Ares emits the existing Z travel command.
4. Ares emits rendered `layer_change_gcode`.
5. Ares then emits part cooling fan and auxiliary fan commands as it does today.

The renderer must:

- Return no output when `layer_change_gcode` is absent or `""`.
- Require the configured value to be a JSON string; non-string values return `SliceError::InvalidInput` mentioning `layer_change_gcode`.
- Ensure non-empty output ends with exactly at least one trailing newline through the existing newline helper behavior.
- Replace direct layer placeholders in both Orca syntaxes:
  - `{layer_num}` and `[layer_num]`
  - `{layer_z}` and `[layer_z]`
  - `{max_layer_z}` and `[max_layer_z]`
- Use the same values as the existing `before_layer_change_gcode` implementation for this slice:
  - `layer_num`: one-based layer number, matching current Ares before-layer behavior.
  - `layer_z`: formatted current layer print Z without Z offset.
  - `max_layer_z`: formatted current layer print Z. This preserves current Ares semantics until the upstream max-layer accumulator is ported.
- Preserve unknown placeholders unchanged, including Orca expression placeholders such as `{layer_num+1}` and unrelated placeholders such as `[total_layer_count]`.

## Non-Goals

- Do not add new option metadata or registry definitions.
- Do not implement the full Orca placeholder expression parser.
- Do not add support for `most_used_physical_extruder_id`, `[total_layer_count]`, arithmetic expressions, conditionals, or filament-indexed placeholders in this slice.
- Do not change generated G-code when `layer_change_gcode` is absent or empty.
- Do not alter existing part-cooling, auxiliary-fan, file-start, machine-start, or before-layer custom G-code behavior except for moving tests or helper code needed to stay under the file-size rule.

## Acceptance Criteria

- `layer_change_gcode` appears after each layer Z travel line and before the first fan command or `; segment_count =` line for that layer.
- Brace and bracket direct layer placeholders render correctly on both generated layers in the existing square pyramid STL test fixture.
- Unknown placeholders are preserved.
- Invalid non-string `layer_change_gcode` reaches `SliceError::InvalidInput`.
- Absent and empty `layer_change_gcode` produce identical output after filtering the existing `; option_count =` line.
- Existing custom G-code tests continue to pass after the test split.
- `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification

Run fresh verification before commit:

- `cargo fmt --check`
- `cargo test -p ares-core --lib layer_change_gcode`
- `cargo test -p ares-core --lib custom_gcode`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `bad=0; while IFS= read -r -d '' f; do n=$(wc -l < "$f"); if [ "$n" -gt 400 ]; then printf '%s %s\n' "$n" "$f"; bad=1; fi; done < <(find crates/ares-core/src -name '*.rs' -print0); exit "$bad"`

## Documentation Impact

No user-facing documentation update is required beyond this spec and the implementation plan. This slice consumes an already accepted Orca option in runtime behavior and does not change CLI usage or public API shape.
