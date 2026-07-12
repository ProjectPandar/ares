# M104: PrintConfig Z-hop, lift-boundary, and extruder/nozzle type registry

## Goal
Port the adjacent Z-hop, Z-hop boundary/type, travel-slope, lift-enforcement, extruder type, nozzle-volume type, and default nozzle-volume type option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5237` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:382-394`, `PrintConfig.hpp:412-421`, `PrintConfig.hpp:1375-1381`, `PrintConfig.hpp:1408-1409`, `PrintConfig.cpp:526-540`, `PrintConfig.cpp:565-575`, `PrintConfig.cpp:5122-5237`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, Z-hop movement behavior, lift enforcement behavior, extruder/nozzle variant resolution behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `z_hop`, `retract_lift_above`, `retract_lift_below`, `z_hop_types`, `travel_slope`, `retract_lift_enforce`, `extruder_type`, `nozzle_volume_type`, and `default_nozzle_volume_type` with exact kinds, defaults, and source line ranges.
- Enum defaults and source citations reference the upstream enum maps for Z-hop type, retract-lift enforcement type, extruder type, and nozzle-volume type.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for Z-hop travel, slope/spiral lift, surface enforcement, extruder/nozzle variant resolution, preset synchronization, slicing, extrusion, and downstream G-code behavior remains deferred.
- Following extruder variant / AMS options from `PrintConfig.cpp:5239+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
