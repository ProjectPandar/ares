# M58: PrintConfig cooling slowdown option registry

## Goal
Port the adjacent FFF cooling-slowdown option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2334-2347` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1519-1520`, `PrintConfig.cpp:2334-2347`, and the current option registry metadata boundary. Because current registry shards are near the 400 LOC limit, the milestone also splits the sorted definition table without changing existing metadata; no new Ares pipeline, crate, dependency, fan runtime behavior, layer-time slowdown behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `reduce_fan_stop_start_freq` and `dont_slow_down_outer_wall` with exact kinds, defaults, and source line ranges.
- The registry definition table is split into smaller sorted shards so every modified Rust file remains under 400 LOC.
- Existing option metadata moved during the split remains unchanged, including `filament_flow_ratio`; all 173 existing definitions are preserved before adding the 2 new keys.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Fan runtime behavior, layer-time slowdown behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `fan_cooling_layer_time`, filament color/options, and following options from `PrintConfig.cpp:2349+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
