# M237: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments FloatOrPercent copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloatsOrPercents` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9718-9738`, with setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, vector `get_at` fallback semantics from `Config.hpp:624-630`, FloatOrPercent storage and serialization context from `Config.hpp:31-42` and `Config.hpp:1318-1450`, and representative FloatOrPercent option context from `PrintConfig.cpp:3017-3043`, `3045-3066`, `3104-3112`, `4016-4026`, and `6936-6947`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing M235/M236 multiple-filament helper to handle `OptionValueKind::FloatOrPercent`.
- Preserve M235/M236 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, and no-partial-mutation behavior.
- Copy handled FloatOrPercent source values into vectors sized to `filament_count`, preserving Orca's skip-on-out-of-range behavior by leaving default `0` absolute entries.
- Allow empty handled FloatOrPercent source vectors and leave every output slot as numeric `0`.
- Preserve absolute JSON numbers and percent strings such as `"20%"` while copying.
- Reject malformed FloatOrPercent values, non-finite numbers, and string `"nil"` with `SliceError::InvalidInput` without partial mutation.
- Keep bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and independent Ares pipeline behavior deferred.
