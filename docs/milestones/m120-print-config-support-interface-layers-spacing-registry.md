# M120: PrintConfig support interface loop, filament, layers, and spacing registry

## Goal
Port the adjacent support interface loop-pattern, interface filament, top/bottom interface layers, and top interface spacing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6055-6112` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:962-967`, `PrintConfig.cpp:6055-6112`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, support interface loop generation, support interface filament routing, support interface layer-count behavior, interface spacing behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_interface_loop_pattern`, `support_interface_filament`, `support_interface_top_layers`, `support_interface_bottom_layers`, and `support_interface_spacing` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support interface loop generation, filament routing, layer-count handling, spacing, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_bottom_interface_spacing`, `support_interface_speed`, and following support options from `PrintConfig.cpp:6114+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
