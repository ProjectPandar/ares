# M94: PrintConfig machine jerk, min-rate, and acceleration PRT option registry

## Goal
Port the adjacent machine XYZE jerk, junction deviation, minimum feedrate, and M204 P/R/T acceleration option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4429-4514` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1259-1274`, `PrintConfig.cpp:4377-4390,4429-4514`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, machine-limit emission behavior, firmware G-code behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `machine_max_jerk_e`, `machine_max_jerk_x`, `machine_max_jerk_y`, `machine_max_jerk_z`, `machine_max_junction_deviation`, `machine_min_extruding_rate`, `machine_min_travel_rate`, `machine_max_acceleration_extruding`, `machine_max_acceleration_retracting`, and `machine_max_acceleration_travel` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Registry key expectation tests remain below 400 LOC by splitting the large expected-key list into focused submodules without changing test behavior.
- Upstream UI full-label/tooltip/category/sidetext/min/max/mode/readonly metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for machine-limit emission, M204/M205 G-code emission, firmware profile behavior, slicing, extrusion, and downstream G-code behavior remains deferred.
- Resonance avoidance, input shaping, and following options from `PrintConfig.cpp:4516+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
