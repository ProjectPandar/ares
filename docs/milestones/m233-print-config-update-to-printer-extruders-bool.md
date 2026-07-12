# M233: DynamicPrintConfig update_values_to_printer_extruders bool copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coBools` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9534-9547`, with function setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, bool vector storage and nullable bool context from `Config.hpp:1857-1967`, and representative bool-vector option context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2334-2338`, `5062-5066`, and `5081-5086`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing `update_values_to_printer_extruders` helper to handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable` in addition to existing string/int/float/percent/FloatOrPercent branches.
- Preserve M230-M232 guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback, sorted/unique key processing, source `get_at` fallback, and no-partial-mutation behavior.
- Copy handled bool source values from `variant_index[e] * stride + i` into a new vector sized `extruder_count * stride` or `stride` for selected `extruder_id`.
- Preserve boolean values as JSON booleans and nullable `"nil"` entries as string `"nil"` only for `BoolsNullable`.
- Reject malformed handled bool vectors, empty handled vectors, non-nullable `"nil"`, and unsupported bool encodings with `SliceError::InvalidInput` without partial mutation.
- Keep enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, and dependency behavior deferred.
- Verify with targeted tests, full workspace tests, `cargo fmt --check`, clippy, wasm check, `git diff --check`, and changed/new Rust file LOC checks.
