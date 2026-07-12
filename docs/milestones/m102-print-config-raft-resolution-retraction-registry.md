# M102: PrintConfig raft, resolution, and retraction trigger registry

## Goal
Port the adjacent raft support, path resolution, and initial retraction trigger option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4988-5066` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:939-943,1367,1549-1551`, `PrintConfig.cpp:4988-5066`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, raft-generation behavior, contour simplification behavior, retraction planning behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `raft_contact_distance`, `raft_expansion`, `raft_first_layer_density`, `raft_first_layer_expansion`, `raft_layers`, `resolution`, `retraction_minimum_travel`, `retract_before_wipe`, and `retract_when_changing_layer` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for raft generation, path simplification, retraction/wipe planning, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following retraction length / cut / extruder-change / Z-hop options from `PrintConfig.cpp:5068+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
