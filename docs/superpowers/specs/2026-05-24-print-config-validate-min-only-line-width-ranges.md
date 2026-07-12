# M206 Spec: PrintConfig validate min-only line-width numeric ranges

## Goal
Extend the source-cited line-width range validation slice from M205 to cover the two remaining Orca `coFloatOrPercent` line-width options in `PrintConfig.cpp:10263-10294`: `skin_infill_line_width` and `skeleton_infill_line_width`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`: generic numeric out-of-range validation loop, especially the `coFloat` / `coPercent` / `coFloatOrPercent` branch and error insertion format.
- `OrcaSlicer/src/libslic3r/Config.cpp:321-338` and `Config.hpp:2476-2481`: `ConfigOptionDef::is_value_valid` min/max predicate, default `min = -FLT_MAX`, default `max = FLT_MAX`, and 1e-4 approximate boundary behavior.
- `OrcaSlicer/src/libslic3r/Config.hpp:1259-1299`: `ConfigOptionFloatOrPercent` raw value, percent flag, and serialization behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3944-3962`: `skin_infill_line_width` and `skeleton_infill_line_width` option definitions, `ratio_over = "nozzle_diameter"`, `min = 0`, default `100%`, and omitted finite `max`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1130-1131`: `FullPrintConfig` field type context for the two included min-only line-width keys.

Related upstream behavior explicitly deferred:

- Generic range validation for every other numeric key and type in `PrintConfig.cpp:10263-10294`, including `coFloat`, `coPercent`, `coFloats`, `coPercents`, `coInt`, and `coInts` keys outside the line-width slice.
- Existing-error suppression across a full `error_message` map from earlier validation blocks; this standalone API returns this slice's own map and does not compose full `DynamicPrintConfig::validate` ordering.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation/range.rs`: extend the existing M205 `SliceOptions::validate_line_width_range_options(&self) -> Result<BTreeMap<String, String>, SliceError>` implementation with per-key line-width ranges.
- `crates/ares-core/src/options/tests/validation/line_width_range.rs`: extend focused tests for min-only skin/skeleton line-width behavior.
- `docs/roadmap.md` and `docs/milestones/m206-print-config-validate-min-only-line-width-ranges.md`: milestone sequencing docs.

## Functional requirements

1. Keep the public API name from M205: `SliceOptions::validate_line_width_range_options()`.
2. Add exactly these two keys to that API's validated line-width set: `skin_infill_line_width` and `skeleton_infill_line_width`.
3. Preserve the existing M205 finite-max behavior for `line_width`, `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, `top_surface_line_width`, `support_line_width`, and `initial_layer_line_width`.
4. Parse raw `FloatOrPercent` values from JSON numbers, numeric strings, percent strings, or source-cited registry defaults.
5. Range-check the raw stored value, not the absolute width, using the source `ConfigOptionDef::is_value_valid` behavior: reject negative values when `min == 0`, accept values approximately equal to min/max within `1e-4`, otherwise require `min <= value <= max`.
6. Use source ranges for this slice:
   - M205 finite-max keys: `[0,1000]`.
   - `skin_infill_line_width` and `skeleton_infill_line_width`: `[0,FLT_MAX]`, because their source definitions set `min = 0` and do not override `ConfigOptionDef::max = FLT_MAX`.
7. If out of range, insert `{serialized_value} not in range [{min:.6},{max:.6}]`; for the min-only keys, `{max:.6}` must be `340282346638528859811704183484516925440.000000`.
8. Percent values serialize with `%`, non-percent values serialize without `%` using Rust `ToString` formatting for the value portion.
9. Malformed or non-finite included line-width values return `SliceError::InvalidInput`.
10. Preserve existing M196-M205 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
11. Do not add full generic range validation, full validation dispatch, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
12. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent min-only skin/skeleton values return no validation errors.
- Tests prove negative skin/skeleton raw values report exact serialized-value range messages with `[0.000000,340282346638528859811704183484516925440.000000]`.
- Tests prove raw skin/skeleton values over `FLT_MAX` report exact range messages.
- Tests prove raw skin/skeleton percent values are range-checked by raw percent value, not absolute nozzle width.
- Tests prove raw skin/skeleton values exactly at min and approximately at max within source epsilon pass, while values outside max epsilon fail.
- Tests prove malformed/non-finite skin/skeleton values return `SliceError::InvalidInput`.
- Tests prove existing M205 finite-max behavior remains intact.
- Plan/spec explicitly account for deferred full generic range validation and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
