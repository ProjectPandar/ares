# M130: PrintConfig change G-code registry

## Goal
Port the adjacent filament-change and extrusion-role-change G-code option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6516-6541` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1392-1395`, `PrintConfig.cpp:6516-6541`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, G-code insertion behavior, tool-change behavior, extrusion-role-change behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `change_extrusion_role_gcode`, `change_filament_gcode`, and `filament_change_extrusion_role_gcode` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for filament-change G-code, extrusion-role-change G-code, tool changes, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- `top_surface_line_width`, `top_surface_speed`, `top_shell_layers`, and following options from `PrintConfig.cpp:6543+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
