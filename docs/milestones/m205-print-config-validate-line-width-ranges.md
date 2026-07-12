# M205: PrintConfig validate line-width numeric ranges

## Goal
Port the first bounded slice of OrcaSlicer's generic numeric out-of-range validation into Ares for line-width `FloatOrPercent` options.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`, with range predicate context from `Config.cpp:321-338` / `Config.hpp:2476-2481`, `ConfigOptionFloatOrPercent` value/serialization context from `Config.hpp:1259-1299`, and option-definition/default context from `PrintConfig.cpp:2027-2037`, `2322-2332`, `3251-3261`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, `6543-6553`, plus `PrintConfig.hpp:960`, `1093`, `1122`, `1155`, `1162`, `1166`, `1527`. It covers only the generic-loop `coFloatOrPercent` range check for `line_width` and the seven finite-max M204 line-width keys. No generic validation for other numeric types/keys, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_line_width_range_options()` returns a key-to-message map like Orca generic range validation for the included line-width keys.
- Missing keys use source-cited registry defaults and pass.
- Raw `FloatOrPercent` values below each option's source minimum report `{serialized_value} not in range [{min:.6},{max:.6}]`.
- Raw `FloatOrPercent` values above each option's source maximum report `{serialized_value} not in range [{min:.6},{max:.6}]` where the source option has an explicit finite maximum.
- Percent values are range-checked by raw percent value, not by absolute width.
- Finite in-range values pass, and values approximately equal to min/max within the source 1e-4 epsilon pass.
- Malformed/non-finite line-width values return `SliceError::InvalidInput`.
- `skin_infill_line_width` and `skeleton_infill_line_width` are not reported by this API and remain deferred to a later min-only range milestone.
- Existing M196-M204 validation behavior remains intact.
- Generic range validation for all other keys/types remains deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
