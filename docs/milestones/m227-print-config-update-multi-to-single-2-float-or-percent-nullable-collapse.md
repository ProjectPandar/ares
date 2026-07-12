# M227: DynamicPrintConfig update_values_from_multi_to_single_2 FloatOrPercent nullable collapse

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the commented `update_values_from_multi_to_single_2` helper's second `coFloatsOrPercents` branch in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9344`, with declaration/comment context from `PrintConfig.hpp:673-674`, setup/key filtering from `PrintConfig.cpp:9290-9304`, existing M226 float branch context from `PrintConfig.cpp:9307-9325`, the `coFloatsOrPercents` branch from `PrintConfig.cpp:9326-9344`, FloatOrPercent data/order context from `Config.hpp:31-42`, and nullable FloatOrPercent nil semantics from `Config.hpp:1344-1345` and `Config.hpp:1450`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Extend the M226 `update_values_from_multi_to_single_2` API to handle `OptionValueKind::FloatOrPercent` in addition to floats/nullable-floats.
- Preserve M226 float/nullable-float behavior unchanged.
- Process only keys already present in `self`, matching source `t_config_option_keys keys = this->keys()`, and filter them through `key_set`.
- Unknown option definitions and unsupported kinds are skipped without error.
- For each handled FloatOrPercent key, scan all source entries in order, ignore nil entries, and select the first value whose numeric `.value` is strictly less than the upstream initial `9999%` sentinel candidate.
- Use the branch's raw `.value < min.value` comparison and do not apply `FloatOrPercent::operator<` equal-value percent ordering.
- Collapse each handled vector to one element by erasing entries after index `0`.
- If a selected value exists, write the selected FloatOrPercent value to index `0` while preserving its percent flag.
- If no non-nil value below `9999` exists, keep the original first entry after collapsing to one element.
- Invalid FloatOrPercent arrays and empty handled arrays return `SliceError::InvalidInput` with no partial mutation.
- Defer the bool branch, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, and dependency changes.
