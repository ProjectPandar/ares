# M112: PrintConfig timelapse and preheat registry

## Goal
Port the adjacent timelapse, standby temperature, and preheat option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5728-5774` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:281-284`, `PrintConfig.hpp:1565-1567`, `PrintConfig.hpp:1615`, `PrintConfig.cpp:431-435`, `PrintConfig.cpp:5728-5774`, and the current option registry metadata boundary. A mechanical registry-table shard split is allowed only to keep modified Rust files under 400 LOC; no new Ares pipeline, crate, dependency, timelapse capture behavior, ooze-prevention temperature behavior, preheat G-code generation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `timelapse_type`, `standby_temperature_delta`, `preheat_time`, and `preheat_steps` with exact kinds, defaults, and source line ranges.
- `timelapse_type` uses the current registry enum metadata boundary with upstream enum-map citations and default `0`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup after the mechanical shard split.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for timelapse capture, ooze-prevention standby temperature application, preheat command insertion, slicing, extrusion, and downstream G-code behavior remains unchanged/deferred.
- Following file/machine/filament start G-code and later options from `PrintConfig.cpp:5777+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
