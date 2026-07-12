# Small Area Flow Model Header Design

## Goal

Consume OrcaSlicer's `small_area_infill_flow_compensation_model` option in concrete Ares G-code header output, so profiles that already use the option for small-area extrusion behavior also preserve the configured model in the generated config block.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1463-1464` declares the `GCodeConfig` small-area flow compensation group and `small_area_infill_flow_compensation_model` as `ConfigOptionStrings`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4359-4371` defines the option, marks it serialized, and gives the default model entries.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends serialized non-banned, non-nil config values into the G-code header with `; key = value`.

## Ares Boundary

- Destination is `ares-core` only.
- Extend the existing config-header export path in `crates/ares-core/src/options/filament_config_export.rs`, `crates/ares-core/src/options/small_area_infill_flow.rs`, `crates/ares-core/src/gcode_config_header.rs`, and focused tests under `crates/ares-core/src/tests/`.
- Reuse the existing small-area model parser and the existing Orca-compatible `ConfigOptionStrings` serialization used by `filament_ramming_parameters`.
- Do not add new crates, filesystem behavior, UI behavior, or independent slicing pipeline design.

## Behavior

When `small_area_infill_flow_compensation_model` is absent, Ares keeps the current header unchanged.

When present as a valid small-area model, Ares emits one config header line:

```text
; small_area_infill_flow_compensation_model = <serialized strings>
```

Input acceptance follows the existing `crates/ares-core/src/options/small_area_infill_flow.rs` model parser:

- JSON arrays must contain non-empty strings.
- JSON strings are split on newlines and semicolons, trimmed, and empty segments are ignored.
- Parsed entries must still satisfy the current small-area PCHIP model validation: each entry is `length,factor`, at least two points are present, lengths increase from first length `0`, factors increase, all values are finite, and the final factor is `1.0`.

Header serialization follows the existing `ConfigOptionStrings` path after entry normalization and existing semantic validation. Normalization means splitting serialized strings, trimming leading/trailing whitespace from each entry, filtering empty serialized-string fragments, and preserving the remaining entry text including valid internal spacing:

- strings with spaces or tabs inside otherwise valid model entries are quoted and escaped by the existing serializer,
- multiple strings are separated with `;`,
- string input is exported as normalized split entries, preserving already-supported scalar model input instead of rejecting it in the header export path.

Exact expected examples:

```text
["0,0", "\n0.2,0.4444", "\n10,1"] -> entries ["0,0", "0.2,0.4444", "10,1"] -> ; small_area_infill_flow_compensation_model = 0,0;0.2,0.4444;10,1
"0,0\n0.5,0.5;2,1" -> entries ["0,0", "0.5,0.5", "2,1"] -> ; small_area_infill_flow_compensation_model = 0,0;0.5,0.5;2,1
["0,0", "0.5, 0.5", "2, 1"] -> entries ["0,0", "0.5, 0.5", "2, 1"] -> ; small_area_infill_flow_compensation_model = 0,0;"0.5, 0.5";"2, 1"
```

Full `ConfigOptionStrings` quoting for backslashes, double quotes, carriage returns, and arbitrary text remains covered by the existing string-vector export tests; this slice only exports strings that are also valid small-area model entries.

Invalid values return `SliceError::InvalidInput` naming `small_area_infill_flow_compensation_model`, including semantic model validation failures from the existing PCHIP point checks. Invalid values must still be rejected when BTT thumbnail header suppression skips the visible header block, matching the existing validation-before-header pattern.

The header order is upstream-adjacent to the current Ares export order: it follows `support_multi_bed_types` and precedes `filament_colour`.

## Docs Impact

- Update `docs/roadmap.md` with the completed source-cited runtime/header slice and its deferred behavior.
- No user manual, CLI help, or API docs change is required because the public slicing API and option name already exist; this slice only preserves that already accepted option in the generated G-code config header.

## Deferred

- No changes to the already implemented small-area infill extrusion compensation math, role gates, pattern gates, or extrusion output behavior. Existing semantic validation errors may be key-named for boundary clarity.
- No implementation of `has_scarf_joint_seam`.
- No full `append_full_config` exhaustive config export.
- No UI, preset, object/material override, or WipeTower behavior.
- No default header emission for absent values; Ares continues its current explicit-value header policy.

## Acceptance Criteria

- A focused RED run with `cargo nextest run -p ares-core small_area_flow_model_header` fails before implementation because the header line is missing and existing semantic model errors are not yet consistently key-named.
- After implementation, `cargo nextest run -p ares-core small_area_flow_model_header` passes.
- Tests prove configured array input, configured string input, valid-entry quoting with spaces, absence, order, and invalid values including malformed model points and BTT-skipped headers.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC guard.
