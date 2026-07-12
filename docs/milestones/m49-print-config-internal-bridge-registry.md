# M49: PrintConfig internal bridge option registry

## Goal
Port the FFF bridge/internal-bridge option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1938` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:231-238`, `PrintConfig.hpp:928`, `PrintConfig.hpp:986-990`, `PrintConfig.hpp:932`, `PrintConfig.cpp:377-390`, and `PrintConfig.cpp:1847-1938`; no new Ares pipeline, crate, dependency, bridge detection/filtering behavior, extra bridge layer generation, support decision behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- Existing `bridge_no_support` and `thick_bridges` metadata remains registered and gains source coverage for the upstream `PrintConfig.hpp` fields.
- `OPTION_DEFINITIONS` includes `thick_internal_bridges`, `enable_extra_bridge_layer`, `dont_filter_internal_bridges`, and `max_bridge_length` with exact kinds, defaults, and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `early.rs` remains under 400 LOC by moving existing `max_layer_height` and `max_travel_detour_distance` definitions to the start of `late.rs` together with new `max_bridge_length`, preserving sorted merged order.
- Upstream label/category/tooltip/enum label/sidetext/min/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Bridge support decisions, internal bridge filtering, extra bridge layer generation, max-bridge support behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `machine_end_gcode` and following options from `PrintConfig.cpp:1940+` remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
