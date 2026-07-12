# M226: DynamicPrintConfig update_values_from_multi_to_single_2 float nullable collapse

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the commented `update_values_from_multi_to_single_2` helper's first `coFloats` branch in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9321`, with declaration/comment context from `PrintConfig.hpp:673-674`, setup/key filtering from `PrintConfig.cpp:9290-9304`, branch behavior from `PrintConfig.cpp:9306-9321`, and nullable float nil semantics from `Config.hpp:837-838` and `Config.hpp:952`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an `ares-core` API for the M226 slice of `update_values_from_multi_to_single_2`, limited to `OptionValueKind::Floats` and `OptionValueKind::FloatsNullable`.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_set`.
- Unknown option definitions and unsupported kinds are skipped without error.
- For each handled float key, scan all source entries in order, ignore nil entries, and select the minimum non-nil value using the upstream initial `9999.0` sentinel and strict `<` comparison.
- Collapse each handled vector to one element by erasing entries after index `0`.
- If at least one non-nil value smaller than `9999.0` exists, write that selected minimum to index `0`.
- If no non-nil value exists, or all non-nil values are equal to or greater than `9999.0`, keep the original first entry after collapsing to one element.
- Invalid float arrays and empty handled arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer FloatOrPercent and bool branches, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
