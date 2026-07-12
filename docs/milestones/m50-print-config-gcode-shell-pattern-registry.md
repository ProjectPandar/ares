# M50: PrintConfig G-code and shell pattern option registry

## Goal
Port the FFF machine/filament end G-code, object-by-object G-code, vertical shell thickness, and solid shell pattern option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1940-2025` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:87-98`, `PrintConfig.hpp:223-228`, `PrintConfig.hpp:1087`, `PrintConfig.hpp:1090-1092`, `PrintConfig.hpp:1294-1300`, `PrintConfig.cpp:225-255`, `PrintConfig.cpp:368-374`, and `PrintConfig.cpp:1940-2025`; no new Ares pipeline, crate, dependency, custom G-code execution, object-by-object scheduling, vertical shell generation, infill pattern generation, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `machine_end_gcode`, `printing_by_object_gcode`, `filament_end_gcode`, `ensure_vertical_shell_thickness`, `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` with exact kinds, defaults, and source line ranges.
- `early.rs` remains under 400 LOC by moving existing `is_infill_first`, `layer_height`, and `line_width` definitions to the start of `late.rs`, preserving exact metadata and sorted merged order.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/enum label/multiline/full-width/height/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Custom G-code execution, object-by-object scheduling, vertical shell generation, top/bottom/internal solid infill pattern generation, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `outer_wall_line_width`, `outer_wall_speed`, and following options from `PrintConfig.cpp:2027+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
