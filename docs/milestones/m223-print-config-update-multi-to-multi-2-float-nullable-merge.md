# M223: DynamicPrintConfig update_values_from_multi_to_multi_2 float nullable merge

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the declaration and first `coFloats` branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9221`, with `PrintConfig.hpp:676` declaration context, `PrintConfig.cpp:9172-9190` same-variant index preparation and source-key/key-set filtering, `PrintConfig.cpp:9191-9197` option-definition lookup skip behavior, `PrintConfig.cpp:9199-9221` nullable float merge branch, and nullable float `nil` semantics from `Config.hpp:837-838` and `Config.hpp:952`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an `ares-core` API for the M223 slice of `update_values_from_multi_to_multi_2`, with explicit source extruder variants, destination extruder variants, destination config, and key set inputs.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_sets`.
- Unknown option definitions are skipped without error.
- Only `OptionValueKind::Floats` and `OptionValueKind::FloatsNullable` are handled in this milestone; all other kinds are skipped.
- Compute `same_variant_indices` for every destination variant by collecting source variant indices equal to it; missing source variants produce an empty list with no fallback.
- For each handled float key, start from the destination config's float/nullable-float vector and overwrite each destination index only when one or more non-nil source values exist at same-variant source indices.
- If a handled key is missing from the destination config, return `SliceError::InvalidInput` with no partial mutation; this makes the source `dst_config.option<...>(key)->values` requirement explicit at the Ares boundary.
- Source nil values are ignored. Ares represents nullable float nil as JSON string `"nil"`, matching existing registry/default serialization conventions.
- If multiple non-nil source values match a destination variant, write the numeric minimum.
- If no matching non-nil source value exists for a destination variant, keep the destination config value unchanged.
- Length mismatches between source values and source variants or destination values and destination variants return `SliceError::InvalidInput` with no partial mutation.
- Invalid float/nullable-float arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer `coFloatsOrPercents`, `coBools`, `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
