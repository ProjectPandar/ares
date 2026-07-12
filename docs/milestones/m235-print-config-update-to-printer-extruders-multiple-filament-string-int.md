# M235: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments string/int copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the guard, filament-map setup, per-filament variant-index preparation, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9675`, with declaration context from `PrintConfig.hpp:664`, `filament_map` option context from `PrintConfig.cpp:2401-2405`, `filament_extruder_variant` / `filament_self_index` context from `PrintConfig.cpp:5292-5304`, vector `get_at` fallback semantics from `Config.hpp:624-630`, and existing extruder variant lookup behavior from `PrintConfig.cpp:8744-8818`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an `ares-core` helper mirroring the M235 source slice for multiple-filament updates, handling only `OptionValueKind::Strings` and `OptionValueKind::Ints`.
- Preserve the support-different-extruders guard and no-op behavior when neither multiple extruders nor different extruder variants are present.
- Skip without mutation when `printer_config` lacks `filament_map`, `extruder_type`, or `nozzle_volume_type`.
- Build one variant index per filament from `filament_map[f] - 1` mapped through `extruder_type` / `nozzle_volume_type` first-value fallback and `get_index_for_extruder(f + 1, id_name, ..., variant_name)`.
- When lookup is negative, fall back to source index `0`, or to the index in `id_name` whose value equals `f + 1` when `id_name` exists.
- Copy handled string/int values into vectors sized to `filament_count`, preserving Orca's skip-on-out-of-range behavior by leaving default entries when a variant index exceeds the source vector length.
- Preserve sorted/unique key processing, unknown-key skip, missing-source skip, and no-partial-mutation on malformed handled vectors.
- Keep float, percent, FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and independent Ares pipeline behavior deferred.
- Verify with targeted tests, full workspace tests, `cargo fmt --check`, clippy, wasm check, `git diff --check`, and changed/new Rust file LOC checks.
