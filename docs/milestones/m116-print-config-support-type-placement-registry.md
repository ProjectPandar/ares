# M116: PrintConfig support type and support placement registry

## Goal
Port the adjacent support type, support/object placement gap, support pattern angle, and support placement filter option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5910-5979` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:195-209`, `PrintConfig.cpp:342-348`, `PrintConfig.hpp:950-955`, `PrintConfig.cpp:5910-5979`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support generation behavior, tree/normal support algorithm behavior, manual support enforcer/blocker behavior, support geometry, support pattern generation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_type`, `support_object_xy_distance`, `support_object_first_layer_gap`, `support_angle`, `support_on_build_plate_only`, `support_critical_regions_only`, and `support_remove_small_overhang` with exact kinds, defaults, and source line ranges.
- `support_type` cites the upstream `SupportType` enum map and uses default `normal(auto)`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support generation, tree/normal/manual support selection, support enforcer/blocker handling, support geometry, support pattern placement, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_top_z_distance` and following support options from `PrintConfig.cpp:5981+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
