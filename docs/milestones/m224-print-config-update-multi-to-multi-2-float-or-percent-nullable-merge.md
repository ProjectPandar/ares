# M224: DynamicPrintConfig update_values_from_multi_to_multi_2 FloatOrPercent nullable merge

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the second `coFloatsOrPercents` branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9246`, with `PrintConfig.hpp:676` declaration context, existing M223 setup from `PrintConfig.cpp:9172-9197`, the `coFloatsOrPercents` branch from `PrintConfig.cpp:9223-9246`, FloatOrPercent data/order context from `Config.hpp:31-42`, and nullable FloatOrPercent nil semantics from `Config.hpp:1344-1345` and `Config.hpp:1450`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the M223 `update_values_from_multi_to_multi_2` API to handle `OptionValueKind::FloatOrPercent` in addition to floats/nullable-floats.
- Preserve M223 float/nullable-float behavior unchanged.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_sets`.
- Unknown option definitions and unsupported kinds are skipped without error.
- Compute `same_variant_indices` for every destination variant by collecting source variant indices equal to it; missing source variants produce an empty list with no fallback.
- For each handled FloatOrPercent key, start from the destination config's FloatOrPercent vector and overwrite each destination index only when one or more non-nil source values exist at same-variant source indices.
- Source nil values are ignored. Ares represents nullable FloatOrPercent nil as JSON string `"nil"`.
- If multiple non-nil source values match a destination variant, initialize a candidate to `9999%`, replace it only when `src_values[idx].value < candidate.value`, and write the final candidate. This branch follows the upstream `src_values[idx].value < target_value.value` comparison and does not apply `FloatOrPercent::operator<` equal-value percent ordering.
- If at least one non-nil source value matches but all values are equal to or greater than `9999`, write `9999%`, matching the upstream sentinel behavior.
- If no matching non-nil source value exists for a destination variant, keep the destination config value unchanged.
- Missing destination config values, source/destination length mismatches, and invalid FloatOrPercent arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer `coBools`, `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
