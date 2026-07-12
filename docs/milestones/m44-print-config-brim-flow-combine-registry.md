# M44: PrintConfig brim flow and combine option registry

## Goal
Port the FFF `brim_flow_ratio`, `brim_use_efc_outline`, and `combine_brims` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1637-1663` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:921-922,1619` and `PrintConfig.cpp:1637-1663`; at original registry completion it added no new Ares pipeline, crate, brim flow behavior, elephant-foot-compensated brim outline behavior, brim combining behavior, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior.

Later source-cited runtime slices now consume `brim_flow_ratio`, `combine_brims`, and the rectangle-scaffold outer `brim_use_efc_outline` path. The EFC-outline slice follows `OrcaSlicer/src/libslic3r/Brim.cpp:55-62` gating and applies the active elephant-foot compensation offset to current Ares rectangular outer brim bounds. Full Orca EFC surface generation, polygon/ex-polygon outline selection, support brim EFC behavior, and painted/ear EFC snapping remain deferred.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `brim_flow_ratio`, `brim_use_efc_outline`, and `combine_brims` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Original M44 registry-only behavior deferred brim flow calculation, EFC outline alignment, brim combining, extrusion behavior, and downstream G-code behavior.
- Later runtime slices consume brim flow calculation, brim combining, and rectangle-scaffold outer EFC outline alignment; full Orca EFC polygon/surface behavior remains deferred.
- Existing `brim_width`, `brim_type`, and `brim_object_gap` registry entries remain unchanged.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
