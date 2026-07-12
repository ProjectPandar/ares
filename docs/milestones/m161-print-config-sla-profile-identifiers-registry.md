# M161: PrintConfig SLA profile identifiers registry

## Goal
Port the next SLA material/print profile identifier settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7507-7535` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:7507-7535` and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA profile-selection behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `material_vendor`, `default_sla_material_profile`, `sla_material_settings_id`, `default_sla_print_profile`, and `sla_print_settings_id` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA profile selection, material vendor resolution, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `supports_enable`, SLA support settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7537+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
