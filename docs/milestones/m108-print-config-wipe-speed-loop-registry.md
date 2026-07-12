# M108: PrintConfig wipe speed and loop registry

## Goal
Port the adjacent wipe-speed and loop-wipe option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5502-5538` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1183-1186`, `PrintConfig.cpp:5502-5538`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, role-based wipe speed selection, loop-wipe movement, external-loop wipe placement, wipe speed calculation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `role_based_wipe_speed`, `wipe_on_loops`, `wipe_before_external_loop`, and `wipe_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for role-based wipe speed selection, wipe speed calculation, loop-wipe movement, external-loop wipe placement, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following skirt/draft-shield options from `PrintConfig.cpp:5540+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
