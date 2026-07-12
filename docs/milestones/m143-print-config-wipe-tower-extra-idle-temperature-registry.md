# M143: PrintConfig wipe-tower extra and idle-temperature registry

## Goal
Port the adjacent wipe-tower bridging/extra purge-line and idle-temperature option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6872-6905` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1588-1589`, `PrintConfig.hpp:1595`, `PrintConfig.hpp:1603`, `PrintConfig.cpp:6872-6905`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, wipe-tower bridging behavior, purge-line spacing/flow behavior, idle-temperature/ooze-prevention behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `idle_temperature`, `wipe_tower_bridging`, `wipe_tower_extra_flow`, and `wipe_tower_extra_spacing` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wipe-tower bridging distance, purge-line spacing/flow, idle temperature, ooze prevention, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `xy_hole_compensation`, `xy_contour_compensation`, `hole_to_polyhole`, and following options from `PrintConfig.cpp:6907+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
