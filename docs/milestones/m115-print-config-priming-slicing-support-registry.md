# M115: PrintConfig priming, slicing mode, Z offset, and support-enable registry

## Goal
Port the adjacent single-extruder priming, slice gap closing, slicing mode, Z offset, and support-enable option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5863-5908` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:162-170`, `PrintConfig.cpp:305-310`, `PrintConfig.hpp:946-948`, `PrintConfig.hpp:1390`, `PrintConfig.hpp:1609`, `PrintConfig.cpp:5863-5908`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, single-extruder priming behavior, mesh gap-closing behavior, slicing mode behavior, Z-offset application, support generation behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `single_extruder_multi_material_priming`, `slice_closing_radius`, `slicing_mode`, `z_offset`, and `enable_support` with exact kinds, defaults, and source line ranges.
- `slicing_mode` cites the upstream `SlicingMode` enum map and uses default `regular`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for extruder priming, mesh gap closing, slicing-mode polygon rules, Z-offset application, support generation, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_type` and following support options from `PrintConfig.cpp:5910+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
