# M160: PrintConfig SLA material correction registry

## Goal
Port the next SLA material correction settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7479-7505` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1817-1820`, `PrintConfig.cpp:7479-7505`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA material correction behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `material_correction`, `material_correction_x`, `material_correction_y`, and `material_correction_z` with exact kinds, defaults, and source line ranges.
- Existing `material_colour`, `material_density`, and `material_type` metadata remains unchanged if mechanically moved into a smaller shard.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA material correction/scaling, material profiles, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `material_vendor`, SLA profile identifiers, `material_print_speed`, and later SLA settings from `PrintConfig.cpp:7507+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
