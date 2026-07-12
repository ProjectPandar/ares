# Spec: M337 PrintApply apply filament_map_mode guard

## Goal

Port the filament-map processing set setup and `filament_map_mode` guard from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1190-1192` into `ares-core` private staged state.

## Upstream source mapping

```cpp
//BBS: process the filament_map related logic
std::unordered_set<std::string> print_diff_set(print_diff.begin(), print_diff.end());
if (print_diff_set.find("filament_map_mode") == print_diff_set.end())
```

The Rust staging must model:

- the upstream comment intent,
- `print_diff_set` as the set created from `print_diff`,
- guard key `filament_map_mode`,
- whether the downstream block is entered based on absence of that key.

## Non-goals / deferred behavior

- Do not implement the `FilamentMapMode map_mode = ...` lookup from `PrintApply.cpp:1194`.
- Do not implement the `map_mode < fmmManual` branch from `PrintApply.cpp:1195-1204`.
- Do not implement the manual-mode `else` branch from `PrintApply.cpp:1205-1226`.
- Do not implement reassignment of `print_diff` from `PrintApply.cpp:1227-1228`.
- Do not mutate real `print_diff`, `m_config`, `m_full_print_config`, or `new_full_config`.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Staged guard records comment intent `BBS: process the filament_map related logic`.
- Staged guard records source identity `print_diff` and set identity `print_diff_set`.
- Input diff keys are represented as set membership, so duplicate keys are counted once.
- Staged guard records guard key `filament_map_mode`.
- When input diff keys do not contain `filament_map_mode`, `enter_filament_map_processing` is true.
- When input diff keys contain `filament_map_mode`, `enter_filament_map_processing` is false.
- Non-guard keys do not prevent entry.
- All new symbols stay private to `ares-core` staged modules.
