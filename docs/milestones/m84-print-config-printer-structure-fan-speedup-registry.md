# M84: PrintConfig printer structure and fan speed-up option registry

## Goal
Port the adjacent FFF printer-structure, best-object-position, auxiliary-fan, and fan speed-up/kick-start option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3681-3738` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:357-363,1310-1312,1404,1406,1541`, `PrintConfig.cpp:494-501,3681-3738`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, printer-structure behavior, auto-arrange behavior, fan command scheduling/kick-start behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `printer_structure`, `best_object_pos`, `auxiliary_fan`, `fan_speedup_time`, `fan_speedup_overhangs`, and `fan_kickstart` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/sidetext/min/mode/enum-label metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for printer structure, best object positioning, fan speed-up scheduling, fan kick-start commands, slicing, extrusion, and downstream G-code behavior remains deferred.
- `part_cooling_fan_min_pwm` and following options from `PrintConfig.cpp:3740+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
