# M54: PrintConfig extruder visual and offset option registry

## Goal
Port the FFF grab length, extruder color, and extruder offset option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2202-2225` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1517-1518`, `PrintConfig.hpp:1625`, and `PrintConfig.cpp:2202-2225`; no new Ares pipeline, crate, dependency, UI color behavior, extruder offset behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `grab_length`, `extruder_colour`, and `extruder_offset` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/gui_type/sidetext/min/mode behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- UI color behavior, firmware/tool offset behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_flow_ratio`, `print_flow_ratio`, and following options from `PrintConfig.cpp:2227+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
