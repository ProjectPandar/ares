# M204: PrintConfig validate extrusion width limit

## Goal
Port OrcaSlicer's extrusion-width upper-limit validation slice into Ares as an explicit `SliceOptions::validate_extrusion_width_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10237-10261`, with limit context from `OrcaSlicer/src/libslic3r/libslic3r.h:68`, `ConfigOptionFloatOrPercent::get_abs_value(double)` context from `Config.hpp:1259-1285`, message-value lookup context from `Config.cpp:690-743`, explicit-base option lookup context from `Config.cpp:745-753`, and option-definition/default context from `PrintConfig.cpp:2027-2037`, `3251-3261`, `3944-3962`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, `6543-6553`, plus `PrintConfig.hpp:960`, `1093`, `1122`, `1130`, `1131`, `1155`, `1162`, `1166`, `1527`. It covers only the nine listed extrusion-width keys, the `MAX_LINE_WIDTH_MULTIPLIER * max_nozzle_diameter` upper-limit predicate, and the source no-argument message-value lookup for reported keys. No generic out-of-range validation, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_extrusion_width_options()` returns a key-to-message map like Orca validation for the extrusion-width source slice.
- Missing keys use source-cited registry defaults and pass.
- `max_nozzle_diameter` is computed from `nozzle_diameter` values and defaults to the registry/default nozzle vector when missing.
- For each listed width key, absolute values and percent values are resolved over `max_nozzle_diameter` for the source predicate.
- Any listed width whose predicate value is greater than `5 * max_nozzle_diameter` reports its key with `too large line width {message_value:.6}` where `message_value` follows the source no-argument `cfg.get_abs_value(key)` path for absolute values; for percent line-width values whose no-argument upstream path recurses into vector `nozzle_diameter` and is documented by `Config.cpp:735-737` as not handling extrusion widths correctly, Ares uses the explicit-base predicate value as a bounded non-panicking compatibility deviation for this validation message.
- Values exactly equal to `5 * max_nozzle_diameter` pass, matching the strict `>` predicate.
- For this source slice, malformed or non-finite nozzle/width boundary values return `SliceError::InvalidInput`; finite but source-range-invalid values are parsed only as needed for this predicate, while generic min/max range reporting remains deferred to `PrintConfig.cpp:10263+`.
- Existing M196-M203 validation behavior remains intact.
- `PrintConfig.cpp:10263+` generic out-of-range validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
