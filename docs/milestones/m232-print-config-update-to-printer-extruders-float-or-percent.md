# M232: DynamicPrintConfig update_values_to_printer_extruders FloatOrPercent copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloatsOrPercents` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9519-9532`, with function setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, `FloatOrPercent` scalar/vector storage and serialization context from `Config.hpp:31-42` and `Config.hpp:1318-1450`, and representative `FloatOrPercent` option context from `PrintConfig.cpp:3017-3043`, `3045-3066`, `3104-3112`, `4016-4026`, and `6936-6947`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing `update_values_to_printer_extruders` helper to handle `OptionValueKind::FloatOrPercent` in addition to existing string/int/float/percent branches.
- Preserve M230/M231 guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback, sorted/unique key processing, source `get_at` fallback, and no-partial-mutation behavior.
- Copy handled FloatOrPercent source values from `variant_index[e] * stride + i` into a new vector sized `extruder_count * stride` or `stride` for selected `extruder_id`.
- Preserve absolute FloatOrPercent values as JSON numbers and percent FloatOrPercent values as strings ending in `%`.
- Reject malformed handled FloatOrPercent vectors, empty handled vectors, `nil`, and non-finite numeric values with `SliceError::InvalidInput` without partial mutation.
- Keep bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, and dependency behavior deferred.
- Verify with targeted tests, full workspace tests, `cargo fmt --check`, clippy, wasm check, `git diff --check`, and changed/new Rust file LOC checks.
