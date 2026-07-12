# M234: DynamicPrintConfig update_values_to_printer_extruders enum copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coEnums` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9549-9560`, with function setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, generic enum vector storage/serialization context from `Config.hpp:2101-2201`, and representative enum-vector option context from `PrintConfig.cpp:5149-5162` (`z_hop_types`), `PrintConfig.cpp:5187-5200` (`retract_lift_enforce`), `PrintConfig.cpp:5215-5225` (`nozzle_volume_type`), and `CommonDefs.hpp:12-20` plus `PrintConfig.cpp:3652-3669` (`nozzle_type`). It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing `update_values_to_printer_extruders` helper to handle `OptionValueKind::Enums` and `OptionValueKind::EnumsNullable` in addition to existing string/int/float/percent/FloatOrPercent/bool branches.
- Preserve M230-M233 guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback, sorted/unique key processing, source `get_at` fallback, and no-partial-mutation behavior.
- Copy handled enum source values from `variant_index[e] * stride + i` into a new vector sized `extruder_count * stride` or `stride` for selected `extruder_id`.
- Preserve Ares enum values as JSON strings and nullable `"nil"` entries as string `"nil"` only for `EnumsNullable`.
- Reject malformed handled enum vectors, empty handled vectors, non-nullable `"nil"`, and non-string enum encodings with `SliceError::InvalidInput` without partial mutation.
- Keep multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, and dependency behavior deferred.
- Verify with targeted tests, full workspace tests, `cargo fmt --check`, clippy, wasm check, `git diff --check`, and changed/new Rust file LOC checks.
