# M163: PrintConfig SLA support base and placement registry

## Goal
Port the next SLA support base/placement settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7613-7694` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1698-1727`, `PrintConfig.cpp:7613-7694`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA support generation behavior, support base geometry behavior, support placement behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_buildplate_only`, `support_pillar_widening_factor`, `support_base_diameter`, `support_base_height`, `support_base_safety_distance`, `support_critical_angle`, `support_max_bridge_length`, `support_max_pillar_link_distance`, and `support_object_elevation` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA support creation, support base geometry, support placement, automatic support points, pad generation, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `support_points_density_relative`, `support_points_minimal_distance`, pad settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7696+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
