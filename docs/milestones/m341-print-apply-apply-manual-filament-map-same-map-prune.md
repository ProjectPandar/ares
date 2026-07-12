# M341: PrintApply apply manual filament_map same-map prune

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1210-1224`: after M340 has prepared `old_filament_map` and `new_filament_map`, `Print::apply(...)` compares maps only when their sizes match, initializes `same_map = true`, iterates indices in order, continues when values match or the index is not in `used_filament_set`, sets `same_map = false` and breaks on the first used differing index, and erases `filament_map` from `print_diff_set` when `same_map` remains true.

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

Supporting context is M340 manual branch setup and `used_filament_set` creation from `PrintApply.cpp:1121-1122`. This milestone models only the same-size comparison, loop decision, and conditional `filament_map` erase as private staged data.

## Exit criteria

- Preserve same-size guard: no same-map loop and no `filament_map` erase when old/new map lengths differ.
- Preserve initializing `same_map` to true when lengths match.
- Preserve source-order index iteration from `0` to `old_filament_map.size() - 1`.
- Preserve continuing when old and new values at an index are equal.
- Preserve continuing when the index is absent from `used_filament_set`, even if values differ.
- Preserve setting `same_map` false and stopping at the first used index with differing values.
- Preserve erasing `filament_map` from `print_diff_set` only when `same_map` is true after the loop.
- Preserve duplicate-suppressed diff-set behavior after optional erasure.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
