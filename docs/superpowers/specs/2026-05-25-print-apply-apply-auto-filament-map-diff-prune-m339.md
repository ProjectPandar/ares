# Spec: M339 PrintApply apply auto filament_map diff prune

## Goal

Port the auto-mode inner `filament_map` diff-prune and staged config-copy actions from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1196-1203` into `ares-core` private staged state.

## Upstream source mapping

```cpp
if (print_diff_set.find("filament_map") != print_diff_set.end()) {
    print_diff_set.erase("filament_map");
    //full_config_diff.erase("filament_map");
    ConfigOptionInts* old_opt = m_full_print_config.option<ConfigOptionInts>("filament_map", true);
    ConfigOptionInts* new_opt = new_full_config.option<ConfigOptionInts>("filament_map", true);
    old_opt->set(new_opt);
    m_config.filament_map = *new_opt;
}
```

The Rust staging must model:

- branch entry based on `filament_map` membership in `print_diff_set`,
- active erasure of `filament_map` from `print_diff_set`,
- commented full-config diff erase as a non-action,
- old/new required `ConfigOptionInts` lookups,
- `old_opt->set(new_opt)`,
- `m_config.filament_map = *new_opt`.

## Non-goals / deferred behavior

- Do not implement the manual-mode `else` branch from `PrintApply.cpp:1205-1226`.
- Do not implement reassignment of `print_diff` from `PrintApply.cpp:1227-1228`.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not actively erase `filament_map` from `full_config_diff`; upstream has that line commented out.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Without `filament_map` in input diff keys, staged branch is not entered and no actions are emitted.
- With `filament_map` in input diff keys, staged branch is entered.
- With duplicate `filament_map` input keys, set membership suppresses duplicates and resulting set omits `filament_map` once.
- Active staged actions include `print_diff_set.erase("filament_map")`.
- Staged non-action records the commented `full_config_diff.erase("filament_map")` without applying it.
- Old option lookup records receiver `m_full_print_config`, type `ConfigOptionInts`, key `filament_map`, required `true`, and result `old_opt`.
- New option lookup records receiver `new_full_config`, type `ConfigOptionInts`, key `filament_map`, required `true`, and result `new_opt`.
- Staged set action records `old_opt->set(new_opt)`.
- Staged assignment records destination `m_config.filament_map` and source `*new_opt`.
- All new symbols stay private to `ares-core` staged modules.
