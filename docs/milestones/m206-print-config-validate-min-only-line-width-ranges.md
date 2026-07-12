# M206: PrintConfig validate min-only line-width numeric ranges

## Goal
Complete the line-width `FloatOrPercent` slice of OrcaSlicer's generic numeric out-of-range validation by adding the two min-only skin/skeleton line-width options deferred by M205.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`, with range predicate context from `Config.cpp:321-338` / `Config.hpp:2476-2481`, `ConfigOptionFloatOrPercent` value/serialization context from `Config.hpp:1259-1299`, default `ConfigOptionDef` max context from `Config.hpp:2476-2481`, and option-definition/default context from `PrintConfig.cpp:3944-3962` plus `PrintConfig.hpp:1130-1131`. It extends the existing M205 `SliceOptions::validate_line_width_range_options()` API to include only `skin_infill_line_width` and `skeleton_infill_line_width`. No generic validation for other numeric types/keys, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_line_width_range_options()` includes `skin_infill_line_width` and `skeleton_infill_line_width` in addition to the M205 finite-max line-width keys.
- Missing skin/skeleton keys use source-cited registry defaults and pass.
- Raw skin/skeleton `FloatOrPercent` values below source minimum `0` report `{serialized_value} not in range [0.000000,340282346638528859811704183484516925440.000000]`.
- Raw skin/skeleton values above source default max `FLT_MAX` report the same range message with the source default max serialized to six decimals.
- Raw percent values are range-checked by raw percent value, not by absolute width.
- Finite in-range values pass, and values approximately equal to min/max within the source 1e-4 epsilon pass.
- Malformed/non-finite skin/skeleton line-width values return `SliceError::InvalidInput`.
- Existing M196-M205 validation behavior remains intact.
- Generic range validation for all other keys/types remains deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
