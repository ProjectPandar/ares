# M140: PrintConfig wipe-tower rib and filament registry

## Goal
Port the adjacent wipe-tower rib, fillet, and perimeter-filament option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6775-6808` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1598-1601`, `PrintConfig.cpp:6775-6808`, and the current option registry metadata boundary. A mechanical registry-table shard split is included only to keep Rust files under 400 LOC; no new Ares pipeline, crate, dependency, rib geometry behavior, fillet behavior, wipe-tower filament selection behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wipe_tower_extra_rib_length`, `wipe_tower_filament`, `wipe_tower_fillet_wall`, and `wipe_tower_rib_width` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- The tail registry table is split mechanically so modified Rust files remain below 400 LOC.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for rib sizing, rib-width constraints, fillet walls, wipe-tower perimeter filament selection, prime tower generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wiping_volumes_extruders`, `prime_tower_skip_points`, and following options from `PrintConfig.cpp:6810+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
