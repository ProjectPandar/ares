# M218: DynamicPrintConfig update_values_from_single_to_multi bool resize

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coBools` branch of `DynamicPrintConfig::update_values_from_single_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8908`, with `PrintConfig.hpp:670` declaration context, `PrintConfig.cpp:8826-8831` required `multi_config.option(variant_name)` guard, `PrintConfig.cpp:8833-8843` config-definition/key lookup skip behavior, `PrintConfig.cpp:8897-8908` `coBools` resize-only behavior, `Config.hpp:635-662` vector resize semantics, and representative `coBools` option-definition context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`. It completes the currently scoped `update_values_from_single_to_multi` branch sequence from M215-M217 and remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Missing `variant_name` in `multi_config` returns `-1` and leaves `self` unchanged before validating bool-vector keys.
- Present `variant_name` must be a string array; its length is the `variant_count` used by this branch, and empty arrays are accepted only when present bool source arrays are also empty.
- Keys are processed in source-equivalent sorted/unique order, matching `std::set<std::string>& key_set`; unknown keys are skipped without error.
- M215 string/int, M216 float, and M217 FloatOrPercent behavior remain covered and unchanged after the helper is extended or renamed for the bool slice.
- Keys whose registry kind is `OptionValueKind::Bools` and whose source key exists in `multi_config` validate the source as an array of booleans.
- Present bool source arrays must have length exactly `variant_count`; a mismatched present source length returns `SliceError::InvalidInput` instead of relying on C++ `assert`.
- Existing target bool arrays are resized to `variant_count` using Orca `ConfigOptionVector::resize` semantics: truncate when too long, duplicate the first existing value when extending, and use the registry default first bool value when creating an absent target value.
- The branch is resize-only: unlike strings/ints it does not copy source values, and unlike floats/FloatOrPercent it does not limit by source values after resize.
- Invalid present target/source bool values return `SliceError::InvalidInput` at the Ares JSON boundary, with no partial `self` mutation on error.
- The helper returns `0` after successful source-order key processing.
- No `update_values_from_multi_to_multi`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
