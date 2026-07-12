# M351: PrintApply extruder count change handling

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1276-1279`: after `m_full_print_config = new_full_config`, `Print::apply(...)` compares the saved `num_extruders` value to the current `m_config.filament_diameter.size()`. When they differ, it updates `num_extruders` to the current filament-diameter count and sets `num_extruders_changed = true`.

```cpp
if (num_extruders  != m_config.filament_diameter.size()) {
    num_extruders  = m_config.filament_diameter.size();
    num_extruders_changed  = true;
}
```

Supporting context is the pre-branch `num_extruders` capture and `num_extruders_changed = false` initialization at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1248-1250`, plus the prior full print config assignment at `PrintApply.cpp:1274-1275`. This milestone models only the staged comparison and conditional assignment result as private staged data.

## Exit criteria

- Preserve previous-count identity `num_extruders` and current-count source `m_config.filament_diameter.size()`.
- Preserve equality behavior: when previous and current counts match, no assignment is staged and `num_extruders_changed` remains false.
- Preserve change behavior: when counts differ, stage assignment of `num_extruders` to the current count and `num_extruders_changed = true`.
- Preserve exact branch condition semantics as previous count not equal to current filament-diameter count.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer full-config branch exit from `PrintApply.cpp:1280`, `ModelObjectStatusDB` construction from `PrintApply.cpp:1282`, model-object synchronization from `PrintApply.cpp:1284+`, real config mutation, real vector storage, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
