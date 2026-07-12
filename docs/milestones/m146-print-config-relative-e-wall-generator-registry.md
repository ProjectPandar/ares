# M146: PrintConfig relative E and wall-generator registry

## Goal
Port the adjacent relative extrusion and wall-generator option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6980-7001` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:294-300`, `PrintConfig.hpp:1020`, `PrintConfig.hpp:1418`, `PrintConfig.cpp:520-524`, `PrintConfig.cpp:6980-7001`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, relative-E G-code behavior, wall generator/perimeter behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `use_relative_e_distances` and `wall_generator` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for relative extrusion addressing, wipe-tower relative-E enforcement, classic/Arachne perimeter generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wall_transition_length`, `wall_transition_filter_deviation`, and following options from `PrintConfig.cpp:7003+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
