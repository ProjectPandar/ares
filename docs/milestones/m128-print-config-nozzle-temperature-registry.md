# M128: PrintConfig nozzle temperature registry

## Goal
Port the adjacent nozzle-temperature option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478-6501` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1568,1571-1572`, `PrintConfig.cpp:6478-6501`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, nozzle-temperature behavior, temperature range validation behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `nozzle_temperature`, `nozzle_temperature_range_high`, and `nozzle_temperature_range_low` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for nozzle temperature, temperature-range validation, start-G-code variables, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `head_wrap_detect_zone`, `detect_thin_wall`, G-code option definitions, and following options from `PrintConfig.cpp:6503+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
