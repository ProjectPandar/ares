# M225: DynamicPrintConfig update_values_from_multi_to_multi_2 bool nullable merge

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the third `coBools` branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9272`, with `PrintConfig.hpp:676` declaration context, existing setup from `PrintConfig.cpp:9172-9197`, the `coBools` branch from `PrintConfig.cpp:9247-9272`, and nullable bool storage/nil semantics from `Config.hpp:1857-1967`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the M223/M224 `update_values_from_multi_to_multi_2` API to handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable` in addition to existing float and FloatOrPercent branches.
- Preserve M223 float/nullable-float and M224 nullable FloatOrPercent behavior unchanged.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_sets`.
- Unknown option definitions and unsupported kinds are skipped without error.
- Compute `same_variant_indices` for every destination variant by collecting source variant indices equal to it; missing source variants produce an empty list with no fallback.
- For each handled bool key, start from the destination config's nullable bool vector and overwrite each destination index only when one or more non-nil source bool values exist at same-variant source indices.
- Source nil values are ignored. Ares represents nullable bool nil as JSON string `"nil"`.
- If multiple non-nil source bools match a destination variant, write the first non-nil source bool in source-index order and stop scanning that destination variant.
- If no matching non-nil source bool exists for a destination variant, keep the destination config value unchanged, including destination nil values.
- Missing destination config values, source/destination length mismatches, and invalid bool arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
