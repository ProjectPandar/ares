# M228: DynamicPrintConfig update_values_from_multi_to_single_2 bool nullable collapse

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the commented `update_values_from_multi_to_single_2` helper's third `coBools` branch in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9371`, with declaration/comment context from `PrintConfig.hpp:673-674`, setup/key filtering from `PrintConfig.cpp:9290-9304`, existing M226/M227 branch context from `PrintConfig.cpp:9307-9344`, the `coBools` branch from `PrintConfig.cpp:9345-9363`, and nullable bool storage/nil semantics from `Config.hpp:1857-1967`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the M226/M227 `update_values_from_multi_to_single_2` API to handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable` in addition to existing float and FloatOrPercent branches.
- Preserve M226 float/nullable-float and M227 nullable FloatOrPercent behavior unchanged.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_set`.
- Unknown option definitions and unsupported kinds are skipped without error.
- For each handled bool key, scan source entries in order, ignore nil entries, and select the first non-nil bool value.
- Collapse each handled vector to one element by erasing entries after index `0`.
- If a selected value exists, write the selected bool value to index `0`; `false` is a valid non-nil selected value.
- If no non-nil source value exists, keep the original first entry after collapsing to one element.
- Invalid bool arrays and empty handled arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
