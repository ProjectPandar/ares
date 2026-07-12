# M162: PrintConfig SLA support head and pillar registry

## Goal
Port the first SLA support head/pillar settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7537-7611` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:265-269`, `PrintConfig.hpp:1674-1696`, `PrintConfig.cpp:406-411`, `PrintConfig.cpp:7537-7611`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA support generation behavior, pillar/head geometry behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `supports_enable`, `support_head_front_diameter`, `support_head_penetration`, `support_head_width`, `support_pillar_diameter`, `support_small_pillar_diameter_percent`, `support_max_bridges_on_pillar`, and `support_pillar_connection_mode` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA support creation, support-head geometry, pillar geometry, bridge planning, support placement, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `support_buildplate_only`, support base settings, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7613+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
