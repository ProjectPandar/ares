# M100: PrintConfig make-overhang and wall option registry

## Goal
Port the adjacent make-overhang-printable, overhang-wall detection, wall-filament, and inner-wall width/speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850-4916` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1032-1033,1153-1156,1199`, `PrintConfig.cpp:4850-4916`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, make-overhang geometry behavior, overhang-wall detection behavior, wall-filament routing behavior, line-width resolution behavior, speed-planning behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `make_overhang_printable`, `make_overhang_printable_angle`, `make_overhang_printable_hole_size`, `detect_overhang_wall`, `wall_filament`, `inner_wall_line_width`, and `inner_wall_speed` with exact kinds, defaults, and source line ranges.
- `inner_wall_line_width` uses existing float-or-percent registry metadata without adding typed behavior.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for make-overhang geometry, overhang-wall detection, wall-filament routing, wall line-width resolution, wall speed planning, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following `wall_loops`, `alternate_extra_wall`, `post_process`, and later options from `PrintConfig.cpp:4918+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
