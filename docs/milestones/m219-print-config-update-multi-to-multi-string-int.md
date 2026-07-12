# M219: DynamicPrintConfig update_values_from_multi_to_multi string/int copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the guard, variant-index preparation, key lookup, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_from_multi_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9064`, with `PrintConfig.hpp:671` declaration context, `PrintConfig.cpp:8984-9017` new-extruder variant-index preparation, `PrintConfig.cpp:8988-8993` required current/new variant and new-id guards, `PrintConfig.cpp:9019-9031` config-definition/key lookup skip behavior, and representative string/int option-definition context from `PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Missing current `variant_name`, new `variant_name`, or new `id_name` returns `-1` and leaves `self` unchanged before validating copied key values.
- Present current/new variant arrays must be string arrays and present new id array must be an i32 array. Their lengths become `variant_count` and `new_variant_count` for this slice.
- The Rust API accepts `new_extruder_variants` and computes source-equivalent same-variant index groups plus new variant target indices, even though the string/int branch only needs the guards and prepares these values for later float/FloatOrPercent/bool milestones.
- Keys are processed in source-equivalent sorted/unique order, matching `std::set<std::string>& key_set`; unknown keys are skipped without error.
- Keys whose registry kind is `OptionValueKind::Strings` or `OptionValueKind::Ints` and whose source key exists in `new_config` copy the full new source JSON array into `self` under the same key.
- Missing source values for copied kinds are skipped without creating or changing `self` values.
- Invalid present copied string/int source values return `SliceError::InvalidInput`, with no partial `self` mutation on error.
- Unsupported kinds are skipped in this M219 slice.
- The helper returns `0` after successful source-order key processing.
- No float, FloatOrPercent, bool old-value merge branches, `update_values_from_multi_to_multi_2`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
