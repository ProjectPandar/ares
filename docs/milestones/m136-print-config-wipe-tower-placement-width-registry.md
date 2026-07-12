# M136: PrintConfig wipe-tower placement and width registry

## Goal
Port the adjacent wipe-tower placement and prime-tower width option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6716` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1577-1579`, `PrintConfig.cpp:6694-6716`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower placement behavior, prime-tower sizing behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wipe_tower_x`, `wipe_tower_y`, and `prime_tower_width` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wipe-tower X/Y placement, prime-tower width, partplate placement logic, prime tower generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wipe_tower_rotation_angle`, `prime_tower_brim_width`, `wipe_tower_cone_angle`, and following options from `PrintConfig.cpp:6718+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
