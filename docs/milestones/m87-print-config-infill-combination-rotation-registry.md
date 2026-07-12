# M87: PrintConfig infill combination and rotation-template option registry

## Goal
Port the adjacent FFF infill-combination, infill-shift, sparse-infill rotation-template, and solid-infill rotation-template option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3853-3896` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1097,1099-1100,1132`, `PrintConfig.cpp:3853-3896`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, infill-combination behavior, infill shift behavior, infill rotation-template parsing/runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `infill_combination`, `infill_shift_step`, `sparse_infill_rotate_template`, and `solid_infill_rotate_template` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/category/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for infill combination, infill shifting, sparse/solid infill rotation-template parsing, slicing, extrusion, and downstream G-code behavior remains deferred.
- `skeleton_infill_density` and following options from `PrintConfig.cpp:3898+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
