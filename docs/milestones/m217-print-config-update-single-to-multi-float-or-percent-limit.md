# M217: DynamicPrintConfig update_values_from_single_to_multi FloatOrPercent limit

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloatsOrPercents` branch of `DynamicPrintConfig::update_values_from_single_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8896`, with `PrintConfig.hpp:670` declaration context, `PrintConfig.cpp:8826-8831` required `multi_config.option(variant_name)` guard, `PrintConfig.cpp:8833-8843` config-definition/key lookup skip behavior, `PrintConfig.cpp:8881-8896` `coFloatsOrPercents` resize-and-limit behavior, `Config.hpp:31-39` `FloatOrPercent` data shape, `Config.hpp:635-662` vector resize semantics, `Config.hpp:1318-1448` `ConfigOptionFloatsOrPercents` vector parse/serialize context, and representative option-definition context from `PrintConfig.cpp:2027-2037`, `2322-2332`, and `3104-3112`. It builds directly on M215/M216 and remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Missing `variant_name` in `multi_config` returns `-1` and leaves `self` unchanged before validating FloatOrPercent keys.
- Present `variant_name` must be a string array; its length is the `variant_count` used by this branch, and empty arrays are accepted only when copied FloatOrPercent source arrays are also empty.
- Keys are processed in source-equivalent sorted/unique order, matching `std::set<std::string>& key_set`; unknown keys are skipped without error.
- M215 string/int behavior and M216 float behavior remain covered and unchanged after the helper is extended or renamed for the FloatOrPercent slice.
- Keys whose registry kind is `OptionValueKind::FloatOrPercent` and whose source key exists in `multi_config` validate the source as an array of finite number/string values. Strings ending in `%` preserve percent-flag semantics; numeric values and numeric strings without `%` are absolute values.
- Present FloatOrPercent source arrays must have length exactly `variant_count`; a mismatched present source length returns `SliceError::InvalidInput` instead of relying on C++ `assert`.
- Existing target FloatOrPercent arrays are resized to `variant_count` using Orca `ConfigOptionVector::resize` semantics: truncate when too long, duplicate the first existing value when extending, and use the registry default first FloatOrPercent value when creating an absent target value.
- For each index, the target entry is replaced with the complete source entry only when `target[index].value > source[index].value`; otherwise the existing/resized target entry, including percent flag, is preserved.
- Invalid present target/source FloatOrPercent values return `SliceError::InvalidInput` at the Ares JSON boundary, with no partial `self` mutation on error.
- The helper returns `0` after successful source-order key processing.
- No `coBools`, `update_values_from_multi_to_multi`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
