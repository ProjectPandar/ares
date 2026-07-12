# ConfigOptionVector apply_override mapping Spec

## Goal
Port OrcaSlicer's vector `apply_override(...)` semantics into `ares-core` as a private helper for later `compute_filament_override_value(...)` assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/Config.hpp:713-753`: `ConfigOptionVector<T>::apply_override(...)`.

Context only:
- `PrintConfig.cpp:10073-10076`: caller clone/apply/changed comparison context.
- `PrintConfig.cpp:10051-10071`: M258 input substitution context.
- `PrintConfig.cpp:10077-10082`: later changed-key insertion and override config mutation, deferred.

## Requirements
- Add private helpers in `crates/ares-core/src/options/filament_override.rs`.
- Add `fn apply_vector_override(machine_values: &mut Vec<Value>, override_values: &[Value], default_index: &[isize], nullable_override: bool) -> Result<bool, SliceError>`.
- For `nullable_override == false`, replace `machine_values` with `override_values` only when they differ, returning `true` if replaced and `false` when equal.
- For `nullable_override == true`, treat `Value::String("nil")` as nil.
- Nullable override behavior must return `false` and leave values unchanged when `min(machine_values.len(), override_values.len()) < 1`.
- Before applying nullable entries, preserve a copy of the original machine values as defaults.
- Resize `machine_values` to `override_values.len()`; when non-empty, use the first original machine value as fill for growth.
- For each nullable override entry after the zero-overlap guard: copy non-nil values into the same index and set `modified = true`; for nil values, restore from `default_index[i] - 1` when `default_index[i] > 0` and that zero-based index is within the original default vector, otherwise restore from the first original machine value.
- Do not implement type hierarchy dispatch, scalar options, full `compute_filament_override_value`, changed-key insertion, public APIs, preset/profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
- Add tests for non-nullable replacement and unchanged detection, nullable non-nil copy, nullable nil restore from `default_index`, nullable fallback to first default value, nullable zero-overlap no-op, and zero-overlap no-op when either side is empty.

## Non-goals
- Do not implement `diff_keys.emplace_back`, `filament_overrides.set_key_value`, full `compute_filament_override_value`, public APIs, concrete Orca `ConfigOption` class hierarchy, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
