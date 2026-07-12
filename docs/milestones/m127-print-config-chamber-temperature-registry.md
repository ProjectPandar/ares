# M127: PrintConfig chamber temperature registry

## Goal
Port the adjacent chamber-temperature control option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448-6476` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1636-1637`, `PrintConfig.cpp:6448-6476`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, chamber-temperature behavior, G-code emission behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `activate_chamber_temp_control` and `chamber_temperature` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for chamber temperature control, M191/M141 emission, start-G-code variable handling, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `nozzle_temperature`, `nozzle_temperature_range_low`, `nozzle_temperature_range_high`, and following nozzle-temperature options from `PrintConfig.cpp:6478+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
