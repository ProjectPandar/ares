# Spec: M341 PrintApply apply manual filament_map same-map prune

## Goal

Port the manual-mode same-map comparison and conditional `filament_map` diff prune from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1210-1224` into `ares-core` private staged state.

## Upstream source mapping

```cpp
if (old_filament_map.size() == new_filament_map.size())
{
    bool same_map = true;
    for (size_t index = 0; index < old_filament_map.size(); index++)
    {
        if ((old_filament_map[index] == new_filament_map[index])
            || (used_filament_set.find(index) == used_filament_set.end()))
            continue;
        else {
            same_map = false;
            break;
        }
    }
    if (same_map)
        print_diff_set.erase("filament_map");
}
```

The Rust staging must model:

- size equality gate,
- `same_map` initialization and final value,
- ordered index visits,
- the equal-value continue condition,
- the unused-index continue condition,
- first used differing index break,
- conditional `print_diff_set.erase("filament_map")`.

## Non-goals / deferred behavior

- Do not implement reassignment of `print_diff` from `PrintApply.cpp:1227-1228`.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Different old/new map lengths skip the loop, leave `same_map` unset/absent, and do not erase `filament_map`.
- Equal old/new maps mark `same_map` true and erase `filament_map` from the staged diff set.
- Differing values only at unused indices still mark `same_map` true and erase `filament_map`.
- A differing value at a used index marks `same_map` false and does not erase `filament_map`.
- The first used differing index is recorded and later indices are not visited.
- Visited indices are recorded in source order.
- Duplicate diff keys are suppressed in the resulting staged diff set.
- All new symbols stay private to `ares-core` staged modules.
