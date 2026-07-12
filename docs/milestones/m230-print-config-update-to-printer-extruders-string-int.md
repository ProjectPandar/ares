# M230: DynamicPrintConfig update_values_to_printer_extruders string/int copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the guard, variant-index preparation, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9489`, with declaration context from `PrintConfig.hpp:663`, prerequisite helper context from `PrintConfig.cpp:8744-8818`, vector `get_at` fallback semantics from `Config.hpp:624-630`, and representative string/int option context from `PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an `ares-core` mutating API for the M230 slice of `update_values_to_printer_extruders`, limited to `OptionValueKind::Strings` and `OptionValueKind::Ints` keys.
- Preserve source behavior where no mutation occurs unless `printer_config.support_different_extruders(...)` reports more than one extruder or different extruders.
- Preserve source skip behavior when required `printer_config` enum vectors `extruder_type` or `nozzle_volume_type` are absent.
- Prepare variant indices for either a requested 1-based `extruder_id` within range or all printer extruders.
- Use previously ported `get_index_for_extruder` semantics to map each printer extruder to the source config's variant index, including fallback to `0` for invalid all-extruder transient states.
- Copy supported string/int source values from `variant_index[e] * stride + i` into a new vector sized `extruder_count * stride`, using source vector `get_at` first-value fallback.
- Iterate a sorted/unique key set, skip unknown definitions, skip missing source keys, and skip unsupported option kinds.
- Return `SliceError::InvalidInput` without partial mutation for malformed handled source values, malformed required present enum vectors, invalid stride indexing, or out-of-range integer values.
- Defer `coFloats`, `coPercents`, `coFloatsOrPercents`, `coBools`, `coEnums`, `update_values_to_printer_extruders_for_multiple_filaments`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code behavior, crate changes, and dependency changes.
