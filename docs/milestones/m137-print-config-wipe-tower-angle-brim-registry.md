# M137: PrintConfig wipe-tower angle and brim registry

## Goal
Port the adjacent wipe-tower rotation, prime-tower brim width, and wipe-tower cone angle option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6718-6744` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1581-1582`, `PrintConfig.hpp:1594`, `PrintConfig.cpp:6718-6744`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower rotation behavior, prime-tower brim behavior, cone stabilization behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `prime_tower_brim_width`, `wipe_tower_cone_angle`, and `wipe_tower_rotation_angle` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wipe-tower rotation, prime-tower brim width/auto calculation, cone stabilization, prime tower generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wipe_tower_max_purge_speed`, `wipe_tower_wall_type`, and following options from `PrintConfig.cpp:6746+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
