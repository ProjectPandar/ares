# M340: PrintApply apply manual filament_map setup

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1205-1208`: inside the manual-mode `else` branch from M338, `Print::apply(...)` erases `extruder_ams_count` from `print_diff_set`, copies `m_config.filament_map.values` into `old_filament_map`, and reads required `ConfigOptionInts` `filament_map` values from `new_full_config` into `new_filament_map`.

```cpp
else {
    print_diff_set.erase("extruder_ams_count");
    std::vector<int> old_filament_map = m_config.filament_map.values;
    std::vector<int> new_filament_map = new_full_config.option<ConfigOptionInts>("filament_map", true)->values;
```

Supporting context is M337 `print_diff_set` setup and M338's non-auto/manual branch selection. This milestone models only the manual-branch setup actions and copied vectors as private staged data.

## Exit criteria

- Preserve manual-branch entry as a caller-provided gate.
- Preserve active `print_diff_set.erase("extruder_ams_count")` action.
- Preserve duplicate-suppressed diff-set membership after erasing `extruder_ams_count`.
- Preserve old map source identity `m_config.filament_map.values` and result identity `old_filament_map`.
- Preserve new map lookup receiver `new_full_config`, option type `ConfigOptionInts`, key `filament_map`, required flag `true`, value source `values`, and result identity `new_filament_map`.
- Preserve old/new integer values in source order, including duplicates and negative values.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer same-size comparison and loop from `PrintApply.cpp:1210-1224`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup/mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
