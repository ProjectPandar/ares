# M220: DynamicPrintConfig update_values_from_multi_to_multi float merge

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the existing guard, variant-index preparation, key lookup, and `coFloats` old-value merge branch of `DynamicPrintConfig::update_values_from_multi_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9093`, with `PrintConfig.hpp:671` declaration context, `PrintConfig.cpp:8984-9017` new-extruder variant-index preparation, `PrintConfig.cpp:8988-8993` required current/new variant and new-id guards, `PrintConfig.cpp:9019-9031` config-definition/key lookup skip behavior, `PrintConfig.cpp:9065-9093` `coFloats` branch, `Config.hpp:635-662` vector option resize/indexing context, and representative float option-definition context from `PrintConfig.cpp:766-773`, `2349-2357`, and `4591-4599`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Keep M219 string/int behavior intact for the same helper path.
- Missing current `variant_name`, new `variant_name`, or new `id_name` returns `-1` and leaves `self` unchanged before validating copied or merged key values.
- Present current/new variant arrays must be string arrays and present new id array must be an i32 array. Their lengths provide `variant_count` and `new_variant_count` for source assertion checks.
- The Rust API computes source-equivalent `extruder_variant_indices` and `new_variant_indices` from `new_extruder_variants` and uses them for float merging.
- Keys are processed in source-equivalent sorted/unique order; unknown keys are skipped without error.
- String and int keys still copy full new-config source arrays.
- Float keys whose source key exists in `new_config` first copy the full new source float array, then for each new extruder variant with a matching new variant index, merge old current values from all same-variant old indices by keeping the lower old value when it is less than the copied new value.
- If no old variant matches a requested new extruder variant, all old indices are considered, matching the source fallback.
- New extruder variants with no matching new variant index remain as copied source values.
- Missing source values for copied/merged kinds are skipped without creating or changing `self` values.
- Invalid present copied/merged values return `SliceError::InvalidInput`, with no partial `self` mutation on error.
- Float old/source lengths must match source assertions (`variant_count == old_count`, `new_variant_count == new_count`) and return `SliceError::InvalidInput` on mismatch.
- Unsupported kinds other than strings, ints, and floats are skipped in this M220 slice.
- The helper returns `0` after successful source-order key processing.
- No FloatOrPercent or bool old-value merge branches, `update_values_from_multi_to_multi_2`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
