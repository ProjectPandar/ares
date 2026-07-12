# M113: PrintConfig start G-code and filament-change registry

## Goal
Port the adjacent file/machine/filament start G-code and single-extruder manual filament-change option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5777-5819` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1385-1389`, `PrintConfig.cpp:5777-5819`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, file/machine/filament start G-code execution behavior, single-extruder multi-material behavior, manual filament-change behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `file_start_gcode`, `machine_start_gcode`, `filament_start_gcode`, `single_extruder_multi_material`, and `manual_filament_change` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for start G-code injection/execution, single-extruder multi-material handling, manual filament-change command omission, slicing, extrusion, and downstream G-code behavior remains unchanged/deferred.
- Following wipe-tower type/ramming/tool-change and later options from `PrintConfig.cpp:5821+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
