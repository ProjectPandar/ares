# M229: DynamicPrintConfig filament identity query API

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is OrcaSlicer's zero-argument filament identity query helpers in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9373-9396`, with declaration context from `PrintConfig.hpp:678-681`, `filament_type` option context from `PrintConfig.cpp:2784-2797` and `PrintConfig.hpp:1322`, and `filament_vendor` option context from `PrintConfig.cpp:2854-2859` and `PrintConfig.hpp:1326`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add read-only `ares-core` APIs equivalent to `DynamicPrintConfig::get_filament_vendor() const` and the zero-argument `DynamicPrintConfig::get_filament_type() const`.
- Return the first string entry from present, non-empty `filament_vendor` and `filament_type` vectors.
- Return an empty string when the option is absent or the vector is empty, matching Orca's null/empty fallback.
- Reject malformed present values at the Rust public API boundary when the option is not an array or the first entry is not a string.
- Preserve the existing M209 `filament_type_display(id)` support-filament display API unchanged; do not replace it with the zero-argument raw identity query.
- Defer `update_values_to_printer_extruders`, multiple-filament query behavior, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code behavior, crate changes, and dependency changes.
