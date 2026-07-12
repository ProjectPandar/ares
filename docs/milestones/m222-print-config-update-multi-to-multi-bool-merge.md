# M222: DynamicPrintConfig update_values_from_multi_to_multi bool merge

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the existing guard, variant-index preparation, key lookup, string/int copy, float merge, FloatOrPercent merge, and `coBools` old-value merge branch of `DynamicPrintConfig::update_values_from_multi_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9155`, with `PrintConfig.hpp:671` declaration context, `PrintConfig.cpp:8984-9017` new-extruder variant-index preparation, `PrintConfig.cpp:8988-8993` required current/new variant and new-id guards, `PrintConfig.cpp:9019-9031` config-definition/key lookup skip behavior, `PrintConfig.cpp:9032-9123` existing string/int/float/FloatOrPercent branches, `PrintConfig.cpp:9125-9155` `coBools` branch, `Config.hpp:635-662` vector option context, and representative bool option-definition context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Keep M219 string/int behavior, M220 float behavior, and M221 FloatOrPercent behavior intact for the same helper path.
- Rename the helper to `update_values_from_multi_to_multi_string_int_float_percent_bool_keys` without keeping a legacy alias.
- Missing current `variant_name`, new `variant_name`, or new `id_name` returns `-1` and leaves `self` unchanged before validating copied or merged key values.
- Present current/new variant arrays must be string arrays and present new id array must be an i32 array. Their lengths provide `variant_count` and `new_variant_count` for source assertion checks.
- The Rust API computes source-equivalent `extruder_variant_indices` and `new_variant_indices` from `new_extruder_variants` and uses them for bool merging.
- Keys are processed in source-equivalent sorted/unique order; unknown keys are skipped without error.
- String, int, float, and FloatOrPercent behavior from previous milestones remains unchanged.
- Bool keys whose source key exists in `new_config` first copy the full new source bool array, then for each new extruder variant with a matching new variant index, set the copied new value to `true` if any old same-variant old index is `true`.
- If no old variant matches a requested new extruder variant, all old indices are considered, matching the source fallback.
- New extruder variants with no matching new variant index remain as copied source values.
- Missing current bool values use the registry default first value repeated to `variant_count`, matching the source `this->option<ConfigOptionBools>(key, true)` materialization effect.
- Missing source values for copied/merged kinds are skipped without creating or changing `self` values.
- Invalid present copied/merged values return `SliceError::InvalidInput`, with no partial `self` mutation on error.
- Bool old value length must equal `variant_count`; source bool length must equal `new_variant_count`; mismatches return `SliceError::InvalidInput`.
- Unsupported kinds other than strings, ints, floats, FloatOrPercent, and bools are skipped in this M222 slice.
- The helper returns `0` after successful source-order key processing.
- No `update_values_from_multi_to_multi_2`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
