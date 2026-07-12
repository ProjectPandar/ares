# M239: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments enum copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coEnums` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9760-9780`, with setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, enum vector and nullable enum storage context from `Config.hpp:2101-2201`, and representative enum option context from `PrintConfig.cpp:5149-5162`, `5187-5200`, `5202-5213`, `5215-5225`, and `3652-3669`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing M235-M238 multiple-filament helper to handle `OptionValueKind::Enums` and `OptionValueKind::EnumsNullable`.
- Preserve M235-M238 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, and no-partial-mutation behavior.
- Copy handled enum source values into vectors sized to `filament_count`, preserving Orca's skip-on-out-of-range behavior by leaving an empty-string default entry.
- Allow empty handled enum source vectors and leave every output slot as an empty string.
- Preserve nullable `"nil"` entries only for nullable enum kinds.
- Reject malformed enum values and non-nullable `"nil"` with `SliceError::InvalidInput` without partial mutation.
- Keep default unsupported logging, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and independent Ares pipeline behavior deferred.
