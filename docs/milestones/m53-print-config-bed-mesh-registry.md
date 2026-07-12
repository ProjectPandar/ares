# M53: PrintConfig bed mesh option registry

## Goal
Port the FFF bed mesh and adaptive mesh margin option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2162-2200` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1641-1644` and `PrintConfig.cpp:2162-2200`; no new Ares pipeline, crate, dependency, adaptive bed mesh behavior, probe constraint behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes a `Point` variant for upstream `coPoint` registry metadata.
- `OPTION_DEFINITIONS` includes `bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, and `adaptive_bed_mesh_margin` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/mode behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Adaptive bed mesh behavior, probe area clamping, G-code generation, slicing behavior, and extrusion behavior remain deferred.
- `grab_length`, `extruder_colour`, `extruder_offset`, and following options from `PrintConfig.cpp:2202+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
