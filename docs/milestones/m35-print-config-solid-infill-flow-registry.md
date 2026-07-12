# M35: PrintConfig solid infill flow ratio option registry

## Goal
Port the FFF top and bottom solid infill flow ratio `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1286-1305` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1193-1194` and `PrintConfig.cpp:1286-1305`; no new Ares pipeline, crate, flow planning, extrusion behavior, G-code behavior, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes the two solid infill flow ratio options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- The `initial_layer_flow_ratio` legacy remap in `PrintConfig.cpp:8003-8005` is not implemented because Ares has no legacy fallback and M35 is registry-only.
- `set_other_flow_ratios`, per-role flow-ratio options, flow planning, extrusion behavior, and G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
