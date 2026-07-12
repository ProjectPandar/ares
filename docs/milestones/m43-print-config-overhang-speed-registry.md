# M43: PrintConfig overhang speed option registry

## Goal
Port the FFF `enable_overhang_speed`, `slowdown_for_curled_perimeters`, `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, and `overhang_4_4_speed` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1500-1570` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1171-1175,1201` and `PrintConfig.cpp:1500-1570`; no new Ares pipeline, crate, overhang speed behavior, curled-perimeter slowdown behavior, speed planning, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `enable_overhang_speed`, `slowdown_for_curled_perimeters`, and `overhang_1_4_speed` through `overhang_4_4_speed` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/full-label/tooltip/sidetext/min/ratio-over/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Overhang speed classification, curled-perimeter slowdown, speed assignment behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- Existing `bridge_speed` and `internal_bridge_speed` registry entries remain unchanged.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
