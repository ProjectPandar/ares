# M204 Spec: PrintConfig validate extrusion width limit

## Goal
Port OrcaSlicer's extrusion-width upper-limit validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_extrusion_width_options()`, returning validation messages for this source slice without adding generic out-of-range validation or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10237-10261`: computes `max_nozzle_diameter`, iterates nine extrusion-width keys, and inserts `too large line width ...` errors when `cfg.get_abs_value(key, max_nozzle_diameter) > MAX_LINE_WIDTH_MULTIPLIER * max_nozzle_diameter`.
- `OrcaSlicer/src/libslic3r/libslic3r.h:68`: `MAX_LINE_WIDTH_MULTIPLIER = 5`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1259-1285`, `Config.cpp:690-743`, and `Config.cpp:745-753`: `ConfigOptionFloatOrPercent::get_abs_value(double)` behavior plus the source no-argument `ConfigBase::get_abs_value(opt_key)` message-value path.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2037`, `3251-3261`, `3944-3962`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, and `6543-6553`: option-definition/default context for the listed width keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:960`, `1093`, `1122`, `1130`, `1131`, `1155`, `1162`, `1166`, and `1527`: `FullPrintConfig` field type context.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10263+` generic out-of-range validation over all config keys.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Full `ConfigBase::get_abs_value(opt_key)` behavior for general consumers; this milestone only implements the bounded no-argument message-value path needed by reported width keys in this source slice. For percent line-width message values whose upstream no-argument path recurses into vector `nozzle_diameter`, Ares intentionally uses the explicit-base predicate value as a bounded non-panicking deviation because `Config.cpp:735-737` states that `XXX_extrusion_width` parameters are not handled correctly by the no-argument path.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation/extrusion_width.rs`: add `SliceOptions::validate_extrusion_width_options(&self) -> Result<BTreeMap<String, String>, SliceError>`.
- `crates/ares-core/src/options/validation.rs`: register the new validation submodule.
- `crates/ares-core/src/options/tests/validation/extrusion_width.rs`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m204-print-config-validate-extrusion-width-limit.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_extrusion_width_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Compute `max_nozzle_diameter` as the maximum parsed `nozzle_diameter` value using existing numeric-vector/default behavior.
3. Validate exactly these keys, in source order: `outer_wall_line_width`, `inner_wall_line_width`, `sparse_infill_line_width`, `internal_solid_infill_line_width`, `top_surface_line_width`, `support_line_width`, `initial_layer_line_width`, `skin_infill_line_width`, `skeleton_infill_line_width`.
4. Resolve each predicate value over `max_nozzle_diameter`: JSON number and numeric string are absolute millimeters; strings ending in `%` are percent of `max_nozzle_diameter`; missing keys use registry defaults.
5. If a predicate value is greater than `5 * max_nozzle_diameter`, insert that key with `too large line width {message_value:.6}` where `message_value` is computed through the bounded source no-argument `cfg.get_abs_value(key)` path for the reported key when source behavior is well-defined: zero `_line_width` values fall back to no-argument `line_width`, and non-percent values return themselves. For percent line-width values whose no-argument upstream path recurses into vector `nozzle_diameter`, use the explicit-base predicate value as a documented bounded non-panicking deviation because `Config.cpp:735-737` says `XXX_extrusion_width` parameters are not handled correctly by no-argument `get_abs_value`.
6. Include tests that distinguish predicate value from non-percent message value and tests that document the percent-message bounded deviation over multi-nozzle input.
7. Values exactly equal to `5 * max_nozzle_diameter` are valid.
8. Malformed or non-finite nozzle/width values return `SliceError::InvalidInput`; finite source-range-invalid values such as negative widths are not generic range errors in this API unless they fail parsing for this predicate, because `PrintConfig.cpp:10263+` generic min/max validation is deferred.
9. Preserve existing M196-M203 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
10. Do not add generic out-of-range validation, full validation dispatch, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
11. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove `max_nozzle_diameter` uses the maximum nozzle vector entry.
- Tests prove over-limit absolute widths report exact keys and six-decimal no-argument message values.
- Tests prove over-limit percent widths over multi-nozzle input use the documented explicit-base message deviation instead of relying on the upstream no-argument vector-ratio bug.
- Tests prove values at exactly `5 * max_nozzle_diameter` pass.
- Tests prove malformed/non-finite nozzle and width boundary values return `SliceError::InvalidInput`, while generic finite min/max range validation remains deferred.
- Tests prove existing M196/M197/M198/M199/M200/M201/M202/M203 validation APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10263+` generic out-of-range behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
