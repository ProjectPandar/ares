# M88: PrintConfig skin, skeleton, and combined-infill option registry

## Goal
Port the adjacent FFF skin/skeleton infill density, depth, line-width, symmetric-infill, and combined-infill max-layer-height option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3898-3984` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1098,1126-1131,1134`, `PrintConfig.cpp:3898-3984`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, skin/skeleton infill behavior, symmetric-infill behavior, combined-infill layer-height behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `skeleton_infill_density`, `skin_infill_density`, `skin_infill_depth`, `infill_lock_depth`, `skin_infill_line_width`, `skeleton_infill_line_width`, `symmetric_infill_y_axis`, and `infill_combination_max_layer_height` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/category/sidetext/min/max/ratio/mode metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for skin/skeleton infill regions, infill lock depth, skin/skeleton line-width resolution, symmetric Y-axis infill, combined-infill max-layer-height, slicing, extrusion, and downstream G-code behavior remains deferred.
- BBS clumping/wrapping detection options from `PrintConfig.cpp:3986+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
