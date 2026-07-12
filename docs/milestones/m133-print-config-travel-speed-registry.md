# M133: PrintConfig travel-speed registry

## Goal
Port the adjacent travel-speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6610-6626` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1396-1397`, `PrintConfig.cpp:6610-6626`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, travel-planning behavior, Z-travel behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` keeps `travel_speed` with exact kind/default and adds the missing `PrintConfig.hpp:1396` source line.
- `OPTION_DEFINITIONS` includes `travel_speed_z` with exact kind, default, and source line range.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the newly covered key while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for travel speed, Z travel speed, speed planning, slicing, geometry, extrusion, and downstream G-code remains unchanged/deferred.
- `wipe`, `wipe_distance`, `enable_prime_tower`, and following options from `PrintConfig.cpp:6628+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
