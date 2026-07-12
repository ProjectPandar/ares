# M139: PrintConfig wipe-tower wall type registry

## Goal
Port the wipe-tower wall type enum option definition from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6759-6773` plus its `WipeTowerWallType` enum map into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:405-408`, `PrintConfig.hpp:1597`, `PrintConfig.cpp:558-563`, `PrintConfig.cpp:6759-6773`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower wall-shape behavior, cone/rib tower geometry, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wipe_tower_wall_type` with exact kind, default, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for rectangle/cone/rib wall selection, cone/fillet/rib geometry, prime tower generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wipe_tower_extra_rib_length`, `wipe_tower_rib_width`, `wipe_tower_fillet_wall`, `wipe_tower_filament`, and following options from `PrintConfig.cpp:6775+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
