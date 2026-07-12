# M48: PrintConfig cooling and default profile option registry

## Goal
Port the FFF cooling, acceleration, default profile, air-filtration, and exhaust-fan option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1772-1845` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.cpp:1772-1845`; no new Ares pipeline, crate, dependency, fan-control behavior, acceleration planning, default-profile selection, cooling slowdown behavior, filesystem, network, UI, slicing, extrusion, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `slow_down_for_layer_cooling`, `default_acceleration`, `default_filament_profile`, `default_print_profile`, `activate_air_filtration`, `activate_air_filtration_during_print`, `activate_air_filtration_on_completion`, `during_print_exhaust_fan_speed`, `complete_print_exhaust_fan_speed`, and `close_fan_the_first_x_layers` with exact kinds, defaults, and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `registry_helpers.rs` remains under 400 LOC by moving public lookup coverage to a focused test module before adding more cases.
- Upstream label/category/tooltip/sidetext/min/max/mode/CLI metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Cooling slowdown, acceleration planning, default profile selection, air-filtration fan control, exhaust fan G-code behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `bridge_no_support` and `thick_bridges` remain already-registered existing options; `thick_internal_bridges` and following options from `PrintConfig.cpp:1863+` remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
