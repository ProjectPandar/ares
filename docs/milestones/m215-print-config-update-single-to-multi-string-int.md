# M215: DynamicPrintConfig update_values_from_single_to_multi string/int copy

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `coStrings` and `coInts` copy branches of `DynamicPrintConfig::update_values_from_single_to_multi` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8865`, with `PrintConfig.hpp:670` declaration context, `PrintConfig.cpp:8826-8831` required `multi_config.option(variant_name)` guard, `PrintConfig.cpp:8833-8843` config-definition/key lookup skip behavior, and representative option-definition context from `PrintConfig.cpp:5252-5264`, `5272-5284`, `5292-5304`. It adds only an explicit `SliceOptions::update_values_from_single_to_multi_string_int_keys(...)` helper; the C++ `id_name` parameter is intentionally omitted because it is unused in this selected source slice for source string-vector and int-vector copy behavior. It does not port float, FloatOrPercent, bool resizing, `update_values_from_multi_to_multi`, preset/profile materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit criteria

- Missing `variant_name` in `multi_config` returns `-1` and leaves `self` unchanged.
- Present `variant_name` must be a string array, including empty arrays; returning `InvalidInput` for a present non-string-array JSON value is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
- Keys are processed in source-equivalent sorted/unique order, matching `std::set<std::string>& key_set`; unknown keys are skipped without error, matching source missing `ConfigOptionDef` warning behavior.
- Keys whose registry kind is `OptionValueKind::Strings` copy the source value from `multi_config` to `self` when the source key exists.
- Keys whose registry kind is `OptionValueKind::Ints` copy the source value from `multi_config` to `self` when the source key exists.
- Missing source key values are skipped without creating or changing `self` values.
- Unsupported option kinds are skipped in this M215 slice.
- Invalid present source values for copied string/int vector keys return `SliceError::InvalidInput` at the Ares boundary instead of panicking.
- The helper returns `0` after successful source-order key processing.
- No float, FloatOrPercent, bool resize, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
