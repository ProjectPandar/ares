# M167: PrintConfig SLA hollowing registry

## Goal
Port the SLA model-hollowing settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7819-7853` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1791-1802`, `PrintConfig.cpp:7819-7853`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, hollowing geometry, cavity detection, drain-hole behavior, OpenVDB behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `hollowing_enable`, `hollowing_min_thickness`, `hollowing_quality`, and `hollowing_closing_distance` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- Expected-key shards remain below 400 LOC and preserve sorted concatenated order.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Runtime behavior for SLA hollowing, cavity creation, drain holes, material print speed, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `material_print_speed` and later SLA material settings from `PrintConfig.cpp:7855+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
