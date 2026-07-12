# M89: PrintConfig wrapping detection and sparse-infill utility option registry

## Goal
Port the adjacent FFF clumping/wrapping detection, sparse-infill filament, sparse-infill line width, infill/wall overlap, top/bottom infill/wall overlap, and sparse-infill speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3987-4061` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1121-1125,1348-1350`, `PrintConfig.cpp:3987-4061`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wrapping/clumping detection behavior, sparse-infill filament routing, line-width resolution, overlap behavior, speed behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes or refreshes source-cited metadata for `enable_wrapping_detection`, `wrapping_detection_layers`, `wrapping_exclude_area`, `sparse_infill_filament`, `sparse_infill_line_width`, `infill_wall_overlap`, `top_bottom_infill_wall_overlap`, and `sparse_infill_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/category/sidetext/min/max/ratio/gui-type/mode metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for wrapping/clumping detection, wrapping exclude areas, sparse-infill extruder selection, sparse-infill line-width resolution, infill/wall overlap, top/bottom overlap, sparse-infill speed, slicing, extrusion, and downstream G-code behavior remains deferred.
- `inherits`, `inherits_group`, and following options from `PrintConfig.cpp:4063+` remain unchanged/deferred.
- `pre_middle_process` remains below the 400 LOC threshold by splitting its filament-tail registry definitions into a focused `pre_middle_filament` shard without changing registry behavior.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
