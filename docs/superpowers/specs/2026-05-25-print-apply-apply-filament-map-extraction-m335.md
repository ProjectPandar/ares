# Spec: M335 PrintApply apply filament_map extraction

## Goal

Port the local `filament_map` extraction from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1178-1179` into `ares-core` private staged state.

## Upstream source mapping

```cpp
auto opt_filament_map = new_full_config.option<ConfigOptionInts>("filament_map");
std::vector<int> filament_maps = opt_filament_map ? opt_filament_map->values : std::vector<int>();
```

The Rust staging must model:

- source config identity `new_full_config`,
- option key `filament_map`,
- absent option maps to an empty vector,
- present option values are copied into the staged `filament_maps` vector in order.

## Non-goals / deferred behavior

- Do not implement the commented else branch from `PrintApply.cpp:1168-1176`.
- Do not call or port `print_config_diffs(...)` from `PrintApply.cpp:1184`.
- Do not call or port `full_print_config_diffs(...)`, object diffs, region diffs, or filament-map mode mutation logic.
- Do not mutate a real `DynamicPrintConfig` or real `m_config.filament_map`.
- Do not add validation, deduplication, or normalization for `filament_map` values.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Missing `filament_map` produces staged values `[]`.
- Present empty `filament_map` produces staged values `[]`.
- Present `filament_map` values are preserved in source order.
- Duplicate `filament_map` values are preserved.
- Negative `filament_map` values are preserved because this source boundary only copies `ConfigOptionInts` values.
- The staged extraction records source config identity `new_full_config`.
- The staged extraction records key `filament_map`.
- All new symbols stay private to `ares-core` staged modules.
