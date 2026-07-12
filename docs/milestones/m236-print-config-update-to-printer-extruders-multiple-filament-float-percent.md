# M236: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments float/percent copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloats` and `coPercents` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9676-9717`, with setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, vector `get_at` fallback semantics from `Config.hpp:624-630`, float/nullable float and percent storage context from `Config.hpp:812-1091 and Config.hpp:1204-1257`, and representative filament float/percent option context from `PrintConfig.cpp:2462-2470`, `5055-5060`, and `5068-5075`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing M235 multiple-filament helper to handle `OptionValueKind::Floats`, `OptionValueKind::FloatsNullable`, `OptionValueKind::Percent`, `OptionValueKind::Percents`, and `OptionValueKind::PercentsNullable`.
- Preserve M235 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, and no-partial-mutation behavior.
- Copy handled numeric source values into vectors sized to `filament_count`, preserving Orca's skip-on-out-of-range behavior by leaving numeric `0` default entries.
- Allow empty handled numeric vectors and leave every output slot as numeric `0`.
- Preserve nullable `"nil"` entries only for nullable numeric kinds.
- Reject malformed handled numeric vectors and non-nullable `"nil"` with `SliceError::InvalidInput` without partial mutation.
- Keep FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and independent Ares pipeline behavior deferred.
