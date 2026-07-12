# Spec: M351 PrintApply extruder count change handling

## Goal

Port the extruder-count change handling slice from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1276-1279` into `ares-core` private staged state.

## Upstream source mapping

```cpp
if (num_extruders  != m_config.filament_diameter.size()) {
    num_extruders  = m_config.filament_diameter.size();
    num_extruders_changed  = true;
}
```

The Rust staging must model:

- saved previous count identity `num_extruders`,
- current count source `m_config.filament_diameter.size()`,
- inequality branch condition,
- no staged assignment when counts match,
- staged assignment to current count and changed flag true when counts differ.

## Non-goals / deferred behavior

- Do not implement the full-config branch exit from `PrintApply.cpp:1280`.
- Do not implement `ModelObjectStatusDB model_object_status_db` from `PrintApply.cpp:1282`.
- Do not implement model-object synchronization from `PrintApply.cpp:1284+`.
- Do not perform real config mutation, real vector storage, or option lookup.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Staged output records previous-count name `num_extruders` and current-count source `m_config.filament_diameter.size()`.
- Equal previous/current counts produce `branch_taken = false`, no new count assignment, and `num_extruders_changed = false`.
- Different previous/current counts produce `branch_taken = true`, `assigned_num_extruders = current count`, and `num_extruders_changed = true`.
- Zero counts are handled by the same equality/inequality rules without special cases.
- All new symbols stay private to `ares-core` staged modules.
