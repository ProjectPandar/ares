# M238: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments bool copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coBools` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9739-9758`, with setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, bool vector and nullable bool storage context from `Config.hpp:1857-1967`, and representative bool option context from `PrintConfig.cpp:2252-2255`, `2557-2565`, `5062-5066`, `5081-5086`, and `6628-6633`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the existing M235-M237 multiple-filament helper to handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable`.
- Preserve M235-M237 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, and no-partial-mutation behavior.
- Copy handled bool source values into vectors sized to `filament_count`, preserving Orca's skip-on-out-of-range behavior by leaving JSON `false` default entries.
- Allow empty handled bool source vectors and leave every output slot as JSON `false`.
- Preserve nullable `"nil"` entries only for nullable bool kinds.
- Reject malformed bool values and non-nullable `"nil"` with `SliceError::InvalidInput` without partial mutation.
- Keep enum, default unsupported logging, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and independent Ares pipeline behavior deferred.
