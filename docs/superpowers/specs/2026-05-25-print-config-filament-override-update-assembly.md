# compute_filament_override_value update assembly Spec

## Goal
Port OrcaSlicer's `compute_filament_override_value(...)` clone/apply/change/output suffix into `ares-core` as a private JSON-vector helper for later profile and filament override wiring.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10073-10082`: clone new machine option, apply prepared filament override, compare against old machine option, append changed key, and set the computed override value.

Context only:
- `PrintConfig.cpp:10051-10071`: M258 long-retraction input substitution context.
- `PrintConfig.hpp:690-691`: function declaration context.
- `Config.hpp:713-753`: M259 vector `ConfigOptionVector<T>::apply_override(...)` semantics.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M260 until this M260 plan/spec review returns `APPROVE`.

## Requirements
- Add private helpers in `crates/ares-core/src/options/filament_override.rs`.
- Add `fn compute_filament_override_value(key: &str, old_machine_value: &Value, new_machine_value: &Value, new_filament_value: &Value, enable_long_retraction_when_cut: Option<&Value>, default_index: &[isize], nullable_override: bool, diff_keys: &mut Vec<String>, filament_overrides: &mut serde_json::Map<String, Value>) -> Result<bool, SliceError>`.
- Treat `new_machine_value` and the M258-prepared filament value as JSON arrays for this staged vector helper. Return `SliceError::InvalidInput` at this private helper boundary if either staged vector input is not an array.
- Call the existing M258 `prepared_filament_override_value(...)` before applying override semantics.
- Clone `new_machine_value` into a mutable computed vector, then call the existing M259 `apply_vector_override(...)` helper with `default_index` and `nullable_override`.
- Compare the computed vector value with `old_machine_value` after applying the override; do not use the M259 modified flag as the changed-output predicate.
- If changed, append `key.to_owned()` to `diff_keys`, insert `Value::Array(computed_values)` into `filament_overrides` under `key.to_owned()`, and return `Ok(true)`.
- If unchanged, leave `diff_keys` and `filament_overrides` untouched and return `Ok(false)`.
- Preserve zero-overlap no-op behavior inherited from `apply_vector_override(...)`: if nullable override has no overlap, the computed value remains the cloned new machine value and the final changed predicate is still `old_machine_value != computed_value`.
- Do not implement concrete Orca `ConfigOption` hierarchy dispatch, scalar option override dispatch, public APIs, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
- Add tests for changed insertion, unchanged no-op, M258 long-retraction nil preparation flowing into the update assembly, non-nullable replacement flowing through the final changed predicate, and nullable zero-overlap using the final old-vs-computed comparison.

## Non-goals
- Do not implement full profile/preset filament override wiring, `DynamicPrintConfig` storage, concrete nullable option classes, scalar option override dispatch, public APIs, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
