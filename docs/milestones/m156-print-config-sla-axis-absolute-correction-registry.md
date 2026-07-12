# M156: PrintConfig SLA axis and absolute correction registry

## Goal
Port the next SLA printer correction options from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7320-7349` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1838-1841`, `PrintConfig.cpp:7320-7349`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA correction behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `relative_correction_x`, `relative_correction_y`, `relative_correction_z`, and `absolute_correction` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above; split registry and metadata fixtures as needed.
- Runtime behavior for SLA scaling/correction, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `elefant_foot_min_width` and later SLA settings from `PrintConfig.cpp:7351+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
