# M72: PrintConfig infill direction and extra solid option registry

## Goal
Port the adjacent FFF infill direction, sparse density, model-aligned infill direction, extra solid infill, and multiline infill option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2861-2913` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1095-1096,1101,1106-1107,1135`, `PrintConfig.cpp:2861-2913`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, infill-angle behavior, model-aligned direction behavior, extra solid infill insertion, multiline infill runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes or updates source-cited metadata for `infill_direction`, `solid_infill_direction`, `sparse_infill_density`, `align_infill_direction_to_model`, `extra_solid_infills`, and `fill_multiline` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- Existing `infill_direction` and `sparse_infill_density` definitions preserve kind/default while adding the upstream field citations for this source slice.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Infill-angle behavior, model-aligned direction behavior, extra solid layer insertion, multiline infill behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `gyroid_optimized`, `sparse_infill_pattern`, and following options from `PrintConfig.cpp:2915+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
