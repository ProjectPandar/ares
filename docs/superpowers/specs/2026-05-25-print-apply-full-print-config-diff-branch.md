# PrintApply full print-config diff branch Spec

## Goal
Port OrcaSlicer's `full_print_config_diffs(...)` key-loop behavior into `ares-core` as a private JSON-map helper for later full print config diff wiring.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:269-294`: `new_full_config.keys()` iteration, old/new option lookup, missing-old changed-key insertion, inequality changed-key insertion, `wipe_tower_x` / `wipe_tower_y` plate-index special case, and return of collected `full_config_diff`.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:267-268`: purpose comment for storing full print config into `new_full_config` for G-code export and PlaceholderParser.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6708`: `wipe_tower_x` / `wipe_tower_y` option definitions as float vectors.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:245-258`: M262 matching wipe-tower plate-index comparison logic in `print_config_diffs(...)`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M263 until this M263 plan/spec review returns `APPROVE`.

## Requirements
- Add a private helper in `crates/ares-core/src/options/filament_override.rs` or a split child module if needed for LOC.
- Suggested signature: `fn collect_full_print_config_diff_updates(current_full_config_values: &serde_json::Map<String, Value>, new_full_config_values: &serde_json::Map<String, Value>, new_full_keys: &[String], plate_index: usize, diff_keys: &mut Vec<String>) -> Result<(), SliceError>`.
- Iterate `new_full_keys` in order to mirror `new_full_config.keys()`.
- For each key, require a new value from `new_full_config_values[key]`; this private helper may return `SliceError::InvalidInput` if a provided key is absent from the new map.
- If the old value is absent from `current_full_config_values`, append the key to `diff_keys`.
- If the old value exists and equals the new value, leave outputs untouched.
- If the old value exists and differs from the new value:
  - for `wipe_tower_x` and `wipe_tower_y`, apply Orca's plate-index logic from `PrintApply.cpp:276-287`: if both arrays have `plate_index`, append only when indexed values differ; if exactly one array has `plate_index`, append; if neither array has `plate_index`, suppress;
  - for all other keys, append the key.
- Preserve existing `diff_keys` mutation behavior: append only, do not clear or deduplicate.
- Do not implement public `full_print_config_diffs` wiring, `PrintApply::print_config_diffs` public wiring, print config mutation, config apply/apply_only, placeholder parser updates, profile loading, public APIs, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No public full print config diff API.
- No PlaceholderParser updates or G-code export behavior.
- No print config mutation, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
