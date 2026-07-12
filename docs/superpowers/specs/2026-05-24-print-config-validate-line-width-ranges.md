# M205 Spec: PrintConfig validate line-width numeric ranges

## Goal
Port a bounded, source-cited slice of OrcaSlicer's generic out-of-range numeric validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_line_width_range_options()`, covering `line_width` plus the seven finite-max M204 line-width `FloatOrPercent` options without adding full generic validation dispatch.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`: generic numeric out-of-range validation loop, especially the `coFloat` / `coPercent` / `coFloatOrPercent` branch and error insertion format.
- `OrcaSlicer/src/libslic3r/Config.cpp:321-338` and `Config.hpp:2476-2481`: `ConfigOptionDef::is_value_valid` min/max predicate, including NaN and min/max approximation behavior.
- `OrcaSlicer/src/libslic3r/Config.hpp:1259-1299`: `ConfigOptionFloatOrPercent` raw value, percent flag, and serialization behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2037`, `2322-2332`, `3251-3261`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, and `6543-6553`: option min/max/default context for the included line-width keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:960`, `1093`, `1122`, `1155`, `1162`, `1166`, and `1527`: `FullPrintConfig` field type context for the seven finite-max M204 line-width keys.

Related upstream behavior explicitly deferred:

- Generic range validation for every other numeric key and type, including min-only `skin_infill_line_width` and `skeleton_infill_line_width` in `PrintConfig.cpp:10263-10294`, including `coFloat`, `coPercent`, `coFloats`, `coPercents`, `coInt`, and `coInts` keys outside this line-width slice.
- Existing-error suppression across a full `error_message` map from earlier validation blocks; this standalone API returns this slice's own map and does not compose full `DynamicPrintConfig::validate` ordering.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation/range.rs`: add `SliceOptions::validate_line_width_range_options(&self) -> Result<BTreeMap<String, String>, SliceError>`.
- `crates/ares-core/src/options/validation.rs`: register the new validation submodule.
- `crates/ares-core/src/options/tests/validation/line_width_range.rs`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m205-print-config-validate-line-width-ranges.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_line_width_range_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Validate exactly these `FloatOrPercent` keys: `line_width`, `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, `top_surface_line_width`, `support_line_width`, `initial_layer_line_width`.
3. Parse raw `FloatOrPercent` values from JSON numbers, numeric strings, percent strings, or source-cited registry defaults.
4. Range-check the raw stored value, not the absolute width, using the source `ConfigOptionDef::is_value_valid` behavior: reject negative values when `min == 0`, accept values approximately equal to min/max within `1e-4`, otherwise require `min <= value <= max`. For example, `1001%` is out of range for keys with `max = 1000`, while `100%` is in range.
5. Use source min/max ranges for this slice:
   - `line_width`, `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, `top_surface_line_width`, `support_line_width`, and `initial_layer_line_width`: `[0,1000]`.
6. If out of range, insert `{serialized_value} not in range [{min:.6},{max:.6}]`; percent values serialize with `%`, non-percent values serialize without `%` using Rust `ToString` formatting for the value portion.
7. `skin_infill_line_width` and `skeleton_infill_line_width` are explicitly deferred because their source definitions set `min = 0` but do not set a finite `max`, and this milestone only covers finite `[0,1000]` range messages.
8. `skin_infill_line_width` and `skeleton_infill_line_width` are not included and must not be reported by this API.
9. Malformed or non-finite line-width values return `SliceError::InvalidInput`.
10. Preserve existing M196-M204 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
11. Do not add full generic range validation, full validation dispatch, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
12. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove negative values report exact serialized-value range messages for finite-max keys.
- Tests prove over-max raw values and over-max percent values report exact range messages for finite-max keys.
- Tests prove raw values exactly at min/max and approximately at max within source epsilon pass, while values outside epsilon fail.
- Tests prove `skin_infill_line_width` and `skeleton_infill_line_width` are deferred by this API.
- Tests prove malformed/non-finite values return `SliceError::InvalidInput`.
- Tests prove existing M196/M197/M198/M199/M200/M201/M202/M203/M204 validation APIs remain intact.
- Plan/spec explicitly account for deferred full generic range validation and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
