# M41: PrintConfig one-wall quality option registry

## Goal
Port the FFF `precise_outer_wall`, `only_one_wall_top`, `min_width_top_surface`, `only_one_wall_first_layer`, and `extra_perimeters_on_overhangs` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1404-1444` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1176-1188,1200` and `PrintConfig.cpp:1404-1444`; no new Ares pipeline, crate, wall-spacing behavior, one-wall surface detection, overhang perimeter behavior, flow planning, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `precise_outer_wall`, `only_one_wall_top`, `min_width_top_surface`, `only_one_wall_first_layer`, and `extra_perimeters_on_overhangs` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/sidetext/min/max/ratio-over/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Wall-spacing precision, one-wall top/first-layer behavior, top-surface threshold behavior, overhang extra perimeter generation, and downstream print-planning behavior remain deferred.
- Following overhang-reversal options and later quality/wall options remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
