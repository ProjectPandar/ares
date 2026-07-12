# M152: PrintConfig narrow internal solid infill registry

## Goal
Port the `detect_narrow_internal_solid_infill` option definition from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7154-7161` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1017`, `PrintConfig.cpp:7154-7161`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, narrow internal solid infill detection behavior, infill pattern selection behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `detect_narrow_internal_solid_infill` with exact kind, default, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for narrow internal solid infill detection, concentric/rectilinear pattern selection, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `PrintConfigDef::init_extruder_option_keys` from `PrintConfig.cpp:7164+` remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
