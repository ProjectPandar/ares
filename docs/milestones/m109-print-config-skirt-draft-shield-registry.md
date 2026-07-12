# M109: PrintConfig skirt and draft-shield registry

## Goal
Port the adjacent skirt and draft-shield option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5540-5627` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:286-292`, `PrintConfig.hpp:927`, `PrintConfig.hpp:1512`, `PrintConfig.hpp:1552-1558`, `PrintConfig.cpp:437-447`, `PrintConfig.cpp:5540-5627`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, skirt generation behavior, draft-shield geometry, skirt-per-object behavior, minimum-skirt-length loop calculation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `skirt_distance`, `skirt_start_angle`, `skirt_height`, `single_loop_draft_shield`, `draft_shield`, `skirt_type`, `skirt_loops`, `skirt_speed`, and `min_skirt_length` with exact kinds, defaults, and source line ranges.
- Existing `skirt_distance`, `skirt_height`, `skirt_loops`, and `skirt_speed` registry entries are updated only to complete source citations; their kinds and defaults remain unchanged.
- `draft_shield` and `skirt_type` use the current registry enum metadata boundary with upstream enum-map citations and defaults `disabled` and `combined`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for skirt generation, draft-shield geometry, per-object skirt behavior, minimum-skirt-length loop calculation, slicing, extrusion, and downstream G-code behavior remains unchanged/deferred.
- Following slowdown / minimum sparse infill options from `PrintConfig.cpp:5629+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
