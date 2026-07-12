# M86: PrintConfig G-code flavor and object-label option registry

## Goal
Port the adjacent FFF G-code flavor, pellet-printer, multi-bed, object-label, exclude-object, and verbose-G-code option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785-3851` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:33-46,1355,1461,1623-1624,1626`, `PrintConfig.cpp:161-176,3785-3851`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, G-code flavor behavior, pellet-printer behavior, multi-bed behavior, object-label/exclude-object command behavior, verbose comment emission, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `gcode_flavor`, `pellet_modded_printer`, `support_multi_bed_types`, `gcode_label_objects`, `exclude_object`, and `gcode_comments` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/mode/readonly/enum-label metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for G-code flavor selection, pellet-printer behavior, multi-bed UI/runtime behavior, object labeling, exclude-object commands, verbose comments, slicing, extrusion, and downstream G-code behavior remains deferred.
- `infill_combination` and following options from `PrintConfig.cpp:3853+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
