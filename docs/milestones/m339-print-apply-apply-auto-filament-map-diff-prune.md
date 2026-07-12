# M339: PrintApply apply auto filament_map diff prune

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1196-1203`: inside the auto-mode branch from M338, `Print::apply(...)` checks whether `print_diff_set` contains `filament_map`; when present, it erases `filament_map`, leaves the commented-out `full_config_diff.erase("filament_map")` as a non-action, looks up old and new `filament_map` options as required `ConfigOptionInts`, copies the new option into the old full config option, and assigns `m_config.filament_map = *new_opt`.

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

Supporting context is M337 `print_diff_set` membership and M338 auto-mode entry. This milestone models only the staged actions and resulting key-set membership for this auto-mode inner branch; it does not mutate real configs.

## Exit criteria

- Preserve entering the branch only when `filament_map` is present in `print_diff_set`.
- Preserve `print_diff_set.erase("filament_map")` as the only active diff-set mutation in this branch.
- Preserve duplicate suppression through set membership.
- Preserve the commented `full_config_diff.erase("filament_map")` as a recorded non-action, not an active mutation.
- Preserve old option lookup receiver `m_full_print_config`, key `filament_map`, required flag `true`, and type `ConfigOptionInts`.
- Preserve new option lookup receiver `new_full_config`, key `filament_map`, required flag `true`, and type `ConfigOptionInts`.
- Preserve `old_opt->set(new_opt)` as a staged set action.
- Preserve `m_config.filament_map = *new_opt` as a staged assignment action.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer manual branch from `PrintApply.cpp:1205-1226`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup/mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
