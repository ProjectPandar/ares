# M28: PrintConfig FFF travel avoidance option registry

## Goal
Port the first FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:897-921` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp` fields, `PrintConfigConstants.hpp` defaults, and the listed `PrintConfig.cpp` lines; no new Ares pipeline, travel-planning, crate, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `reduce_crossing_wall` and `max_travel_detour_distance` with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Typed accessors and travel-planning behavior for the new options remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
