# Spec: M340 PrintApply apply manual filament_map setup

## Goal

Port the manual-mode branch setup from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1205-1208` into `ares-core` private staged state.

## Upstream source mapping

```cpp
else {
    print_diff_set.erase("extruder_ams_count");
    std::vector<int> old_filament_map = m_config.filament_map.values;
    std::vector<int> new_filament_map = new_full_config.option<ConfigOptionInts>("filament_map", true)->values;
```

The Rust staging must model:

- manual-branch entry,
- erasing `extruder_ams_count` from `print_diff_set`,
- old map copy from `m_config.filament_map.values`,
- new required `ConfigOptionInts` lookup from `new_full_config` key `filament_map`,
- copied old/new map values.

## Non-goals / deferred behavior

- Do not implement the equal-size check from `PrintApply.cpp:1210`.
- Do not implement the `same_map` loop from `PrintApply.cpp:1212-1222`.
- Do not implement erasing `filament_map` from `print_diff_set` at `PrintApply.cpp:1223-1224`.
- Do not implement reassignment of `print_diff` from `PrintApply.cpp:1227-1228`.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- When manual branch is not entered, staged setup emits no actions and leaves the duplicate-suppressed input set unchanged.
- When manual branch is entered, staged setup records active `print_diff_set.erase("extruder_ams_count")`.
- Resulting staged diff set omits `extruder_ams_count` when present.
- Duplicate diff keys are suppressed in resulting staged diff set.
- Staged old map copy records result `old_filament_map` and source `m_config.filament_map.values`.
- Staged new map lookup records result `new_filament_map`, receiver `new_full_config`, option type `ConfigOptionInts`, key `filament_map`, required `true`, and value source `values`.
- Old and new integer map values are preserved in source order.
- Duplicate and negative integer map values are preserved without validation or deduplication.
- All new symbols stay private to `ares-core` staged modules.
