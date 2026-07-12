# M38: PrintConfig overhang and sparse infill flow ratio option registry

## Goal
Port the FFF `overhang_flow_ratio` and `sparse_infill_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1344-1363` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1214-1219` and `PrintConfig.cpp:1344-1363`; no new Ares pipeline, crate, flow planning, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `overhang_flow_ratio` and `sparse_infill_flow_ratio` with exact defaults and source line ranges.
- Flow-ratio metadata assertions are split out of `metadata.rs` so modified Rust files remain under 400 LOC.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Runtime flow scaling, preset option-list behavior, and object override handling remain deferred.
- Following per-role flow-ratio options, flow planning, extrusion behavior, and G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
