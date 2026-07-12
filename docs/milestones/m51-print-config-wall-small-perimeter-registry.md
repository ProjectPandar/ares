# M51: PrintConfig wall ordering and small perimeter option registry

## Goal
Port the adjacent FFF outer-wall, small-perimeter, wall-order, infill-first, and wall-direction option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2110` into `ares-core` registry metadata, adding only the missing registry keys for small-perimeter and wall ordering/direction options.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:132-144`, `PrintConfig.hpp:1191-1192`, `PrintConfig.hpp:1209-1212`, `PrintConfig.cpp:277-290`, and `PrintConfig.cpp:2027-2110`; no new Ares pipeline, crate, dependency, small-perimeter speed behavior, wall order behavior, wall direction path generation, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- Existing registry definitions for `outer_wall_line_width`, `outer_wall_speed`, and `is_infill_first` remain present and unchanged.
- `OPTION_DEFINITIONS` includes `small_perimeter_speed`, `small_perimeter_threshold`, `wall_sequence`, and `wall_direction` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/enum label/mode/sidetext/ratio/min/max metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Wall ordering, wall direction, small-perimeter speed application, typed accessors, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
