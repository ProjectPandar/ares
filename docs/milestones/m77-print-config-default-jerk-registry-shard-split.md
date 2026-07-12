# M77: PrintConfig default jerk registry with pre-middle shard split

## Goal
Port the adjacent FFF `default_jerk` and `default_junction_deviation` option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169-3186` into `ares-core` registry metadata while splitting the oversized pre-middle registry shard so modified Rust files remain below 400 LOC.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1052,1060`, `PrintConfig.cpp:3169-3186`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, jerk runtime behavior, junction-deviation runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `default_jerk` and `default_junction_deviation` with exact kinds, defaults, and source line ranges.
- `pre_middle.rs` is split into smaller sorted shards without changing existing option metadata or public APIs.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Default jerk runtime behavior, junction-deviation runtime behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `PrintConfig.cpp:3188-3249` jerk options remain owned by M76; `initial_layer_line_width`, `initial_layer_print_height`, and following options from `PrintConfig.cpp:3251+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
