# M159: PrintConfig SLA exposure time registry

## Goal
Port the next SLA printer/material exposure settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7425-7477` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1815-1816`, `PrintConfig.hpp:1848-1851`, `PrintConfig.cpp:7425-7477`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA exposure behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `faded_layers`, `min_exposure_time`, `max_exposure_time`, `exposure_time`, `min_initial_exposure_time`, `max_initial_exposure_time`, and `initial_exposure_time` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA exposure timing, faded layers, material correction, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `material_correction`, `material_correction_x`, `material_correction_y`, `material_correction_z`, and later SLA settings from `PrintConfig.cpp:7479+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
