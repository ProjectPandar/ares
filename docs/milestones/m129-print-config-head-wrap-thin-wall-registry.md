# M129: PrintConfig head-wrap detect zone and thin-wall registry

## Goal
Port the adjacent head-wrap detect zone and thin-wall detection option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6503-6514` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1485`, `PrintConfig.hpp:1165`, `PrintConfig.cpp:6503-6514`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, head-wrap detection behavior, thin-wall slicing behavior, geometry behavior, UI behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `head_wrap_detect_zone` and `detect_thin_wall` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for head-wrap detection, thin-wall detection, slicing geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `change_filament_gcode`, `change_extrusion_role_gcode`, `filament_change_extrusion_role_gcode`, top-surface options, and following options from `PrintConfig.cpp:6516+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
