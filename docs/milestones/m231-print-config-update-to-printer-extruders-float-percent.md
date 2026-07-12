# M231: DynamicPrintConfig update_values_to_printer_extruders float/percent copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloats` and `coPercents` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9491-9517`, with function setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, float/nullable-float storage context from `Config.hpp:812-870`, percent-vector storage represented by Orca option definitions, and representative option context from `PrintConfig.cpp:2227-2237`, `4591-4599`, `4651-4658`, `737-747`, and `6839-6845`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the M230 `update_values_to_printer_extruders` API to handle `OptionValueKind::Floats`, `OptionValueKind::FloatsNullable`, `OptionValueKind::Percent`, `OptionValueKind::Percents`, and `OptionValueKind::PercentsNullable` in addition to existing string/int branches.
- Preserve M230 guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback, sorted/unique key processing, source `get_at` fallback, and no-partial-mutation behavior.
- Copy handled float/percent source values from `variant_index[e] * stride + i` into a new vector sized `extruder_count * stride` or `stride` for selected `extruder_id`.
- Preserve JSON numbers for finite numeric values and preserve string `"nil"` for nullable float/percent vectors.
- Reject malformed handled float/percent vectors, empty handled vectors, and non-finite numeric values with `SliceError::InvalidInput` without partial mutation.
- Keep `FloatOrPercent`, bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, and dependency behavior deferred.
- Verify with targeted tests, full workspace tests, `cargo fmt --check`, clippy, wasm check, `git diff --check`, and changed/new Rust file LOC checks.
