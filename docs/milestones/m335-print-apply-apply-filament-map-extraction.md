# M335: PrintApply apply filament_map extraction

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1178-1179`: immediately after the gated extruder-variant update block, `Print::apply(...)` reads `filament_map` from `new_full_config` as `ConfigOptionInts` and initializes local `filament_maps` to the option values when present or to an empty `std::vector<int>` when absent.

```cpp
auto opt_filament_map = new_full_config.option<ConfigOptionInts>("filament_map");
std::vector<int> filament_maps = opt_filament_map ? opt_filament_map->values : std::vector<int>();
```

Supporting context is the downstream `print_config_diffs(...)` call at `PrintApply.cpp:1184`, which receives `filament_maps`, and the `filament_map` option definition already ported from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2401-2405` / `PrintConfig.hpp:1336`.

This milestone models only the local extraction result as private staged data. It does not compute print diffs or mutate real configs.

## Exit criteria

- Preserve the exact source option key `filament_map`.
- Preserve source identity `new_full_config`.
- Preserve absent-option behavior as an empty vector.
- Preserve present empty `ConfigOptionInts` behavior as an empty vector.
- Preserve present integer values in source order.
- Preserve duplicate and negative integer values without validation or deduplication.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer the commented else branch from `PrintApply.cpp:1168-1176`, the `print_config_diffs(...)` call from `PrintApply.cpp:1184`, full/full-object/region diff computation, filament-map mode mutation logic from `PrintApply.cpp:1190+`, real `DynamicPrintConfig`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
