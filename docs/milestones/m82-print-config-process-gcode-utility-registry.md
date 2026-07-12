# M82: PrintConfig process and G-code utility option registry

## Goal
Port the adjacent FFF process/G-code utility option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3578-3643` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:125-129,1059,1120,1190,1298,1346-1347,1353`, `PrintConfig.cpp:185-190,3578-3643`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, gap-fill filtering behavior, speed behavior, precise-Z behavior, arc-fitting behavior, G-code line-number behavior, first-layer scan behavior, power-loss recovery G-code behavior, slicing behavior, extrusion behavior, or G-code output behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filter_out_gap_fill`, `gap_infill_speed`, `precise_z_height`, `enable_arc_fitting`, `gcode_add_line_number`, `scan_first_layer`, and `enable_power_loss_recovery` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/category/tooltip/sidetext/min/mode/enum label metadata remains deferred beyond the current metadata boundary, while enum key/default string identity is source-cited.
- Runtime behavior for gap filtering, gap speed application, precise-Z layer adjustment, arc fitting, line numbering, first-layer camera scan, power-loss recovery commands, slicing, extrusion, and G-code output remains deferred.
- `nozzle_type` and following options from `PrintConfig.cpp:3652+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
