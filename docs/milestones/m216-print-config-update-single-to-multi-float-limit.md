# M216: DynamicPrintConfig update_values_from_single_to_multi float limit

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coFloats` branch of `DynamicPrintConfig::update_values_from_single_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8880`, with `PrintConfig.hpp:670` declaration context, `PrintConfig.cpp:8826-8831` required `multi_config.option(variant_name)` guard, `PrintConfig.cpp:8833-8843` config-definition/key lookup skip behavior, `PrintConfig.cpp:8866-8880` `coFloats` resize-and-limit behavior, `Config.hpp:635-662` vector resize semantics, `Config.hpp:812-870` `ConfigOptionFloats` vector context, and representative float-vector option-definition context from `PrintConfig.cpp:766-773`, `2349-2357`, and `4591-4599`. It builds directly on M215's string/int copy helper and remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Missing `variant_name` in `multi_config` returns `-1` and leaves `self` unchanged before validating float keys.
- Present `variant_name` must be a string array; its length is the `variant_count` used by the float branch, and empty arrays are accepted only when copied float source arrays are also empty.
- Keys are processed in source-equivalent sorted/unique order, matching `std::set<std::string>& key_set`; unknown keys are skipped without error.
- M215 `Strings` and `Ints` behavior remains covered and unchanged after the helper is extended or renamed for the float slice.
- Keys whose registry kind is `OptionValueKind::Floats` and whose source key exists in `multi_config` validate the source as a finite float array with length exactly `variant_count`; a mismatched present source length returns `SliceError::InvalidInput` instead of relying on C++ `assert`.
- Existing target float arrays are resized to `variant_count` using Orca `ConfigOptionVector::resize` semantics: truncate when too long, duplicate the first existing value when extending, and use the registry default first numeric value when creating an absent target value.
- For each index, the target value is replaced with the source value only when `target[index] > source[index]`; otherwise the existing/resized target value is preserved.
- Invalid present target/source float values return `SliceError::InvalidInput` at the Ares JSON boundary, with no partial `self` mutation on error.
- The helper returns `0` after successful source-order key processing.
- No `coFloatsOrPercents`, `coBools`, `update_values_from_multi_to_multi`, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
