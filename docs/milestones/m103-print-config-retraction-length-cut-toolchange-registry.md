# M103: PrintConfig retraction length, cut, and toolchange registry

## Goal
Port the adjacent base retraction length, long retraction when cut/extruder-change, retraction distance, and toolchange retraction option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5068-5120` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1368-1374`, `PrintConfig.cpp:5068-5120`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, retraction planning behavior, filament-cut behavior, extruder-change behavior, toolchange behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `retraction_length`, `enable_long_retraction_when_cut`, `long_retractions_when_cut`, `retraction_distances_when_cut`, `long_retractions_when_ec`, `retraction_distances_when_ec`, and `retract_length_toolchange` with exact kinds, defaults, and source line ranges.
- Nullable cut/extruder-change options use the existing nullable value kinds when upstream default objects are nullable.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for retraction planning, filament cutting, extruder-change retraction, toolchange retraction, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following Z-hop options from `PrintConfig.cpp:5122+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
