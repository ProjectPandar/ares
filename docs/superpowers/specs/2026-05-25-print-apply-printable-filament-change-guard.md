# PrintApply printable-filament change guard Spec

## Goal
Port OrcaSlicer's `is_printable_filament_changed(...)` entry guard into `ares-core` as a private staged helper for later printable-area/extruder-area diff wiring.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-303`: `old_poly != new_poly` guard and manual `filament_map_mode` early return.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:304-340`: deferred printable-area/extruder-area polygon construction, diff/intersection, and intersection-id comparison branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:577-582`, `PrintConfig.cpp:2414-2428`, and `PrintConfig.hpp:424-428` / `PrintConfig.hpp:1335`: `filament_map_mode` enum option context.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M264 until this M264 plan/spec review returns `APPROVE`.

## Requirements
- Add a private helper in a new internal module `crates/ares-core/src/print_apply.rs` and register it from `crates/ares-core/src/lib.rs` with `mod print_apply;`.
- Suggested signature: `fn printable_filament_change_guard(new_full_config_values: &serde_json::Map<String, serde_json::Value>, old_poly: &[Point2], new_poly: &[Point2]) -> Result<bool, SliceError>`.
- Compare `old_poly` and `new_poly` by exact point equality, matching the upstream `old_poly != new_poly` guard at this staged boundary.
- If polygons are equal, return `Ok(false)` without reading `filament_map_mode`.
- If polygons differ, inspect optional `new_full_config_values["filament_map_mode"]`.
- Treat the string value `"fmmManual"` and the legacy/UI value `"Manual"` as manual for this private staged helper; no other enum parsing is added in this milestone.
- If the mode is manual, return `Ok(false)`.
- If the mode is absent or non-manual, return `Ok(true)` as a staged sentinel for entering the deferred geometry-comparison branch from `PrintApply.cpp:304-340`.
- If `filament_map_mode` is present but not a string, return `SliceError::InvalidInput("filament_map_mode must be a string")`.
- Do not implement printable-area/extruder-area polygon construction, Clipper diff/intersection behavior, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual printable-area / extruder-area geometry calculation.
- No public API or UI-facing API.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
