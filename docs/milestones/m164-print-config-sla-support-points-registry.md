# M164: PrintConfig SLA support points registry

## Goal
Port the automatic SLA support-points settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7696-7710` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1729-1731`, `PrintConfig.cpp:7696-7710`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, automatic support-point placement behavior, SLA support geometry behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_points_density_relative` and `support_points_minimal_distance` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for automatic support-point placement, SLA support generation, support geometry, pad generation, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `pad_enable`, pad settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7712+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
