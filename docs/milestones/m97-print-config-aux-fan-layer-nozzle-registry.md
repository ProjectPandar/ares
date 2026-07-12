# M97: PrintConfig auxiliary fan, min layer, and nozzle option registry

## Goal
Port the adjacent fan-minimum, auxiliary fan, minimum layer height citation, slow-down minimum speed, and nozzle diameter citation option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4651-4721` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1475-1478,1537-1538,1542-1543`, `PrintConfig.cpp:4651-4721`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, cooling behavior, auxiliary fan G-code behavior, adaptive layer-height behavior, nozzle behavior, speed planning behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `fan_min_speed`, `additional_cooling_fan_speed`, `close_additional_fan_first_x_layers`, `additional_fan_full_speed_layer`, `first_x_layer_fan_speed`, and `slow_down_min_speed` with exact kinds, defaults, and source line ranges.
- Existing `min_layer_height` and `nozzle_diameter` source metadata includes `PrintConfig.hpp:1538` and `PrintConfig.hpp:1543` respectively while preserving current kind/default and typed behavior.
- `crates/ares-core/src/options/tests/registry_helpers.rs` is split before adding more fixture entries so modified Rust files remain under 400 LOC.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for part cooling, auxiliary fan commands, adaptive layer-height limits, nozzle-specific behavior, speed planning, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following `notes`, `host_type`, and printer-host options from `PrintConfig.cpp:4723+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
