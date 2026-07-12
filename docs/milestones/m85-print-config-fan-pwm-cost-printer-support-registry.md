# M85: PrintConfig fan PWM, cost, and printer support option registry

## Goal
Port the adjacent FFF part-cooling PWM clamp, time-cost, chamber-temperature support, and air-filtration support option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3740-3783` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1316,1357,1405,1407`, `PrintConfig.cpp:3740-3783`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, fan PWM clamp behavior, printer-cost behavior, chamber-temperature control behavior, air-filtration control behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `part_cooling_fan_min_pwm`, `time_cost`, `support_chamber_temp_control`, and `support_air_filtration` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/sidetext/min/max/mode/readonly metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for part-cooling fan PWM clamping, time-cost calculation, chamber-temperature control, air-filtration fan commands, slicing, extrusion, and downstream G-code behavior remains deferred.
- `gcode_flavor` and following options from `PrintConfig.cpp:3785+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
