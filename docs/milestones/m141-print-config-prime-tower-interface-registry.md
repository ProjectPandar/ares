# M141: PrintConfig prime-tower interface registry

## Goal
Port the adjacent wiping-volume and prime-tower interface option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6810-6845` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1583-1587`, `PrintConfig.hpp:1602`, `PrintConfig.cpp:6810-6845`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, wiping-volume behavior, prime-tower skip/flat-ironing behavior, interface temperature behavior, infill-gap behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `enable_tower_interface_cooldown_during_tower`, `enable_tower_interface_features`, `prime_tower_flat_ironing`, `prime_tower_infill_gap`, `prime_tower_skip_points`, and `wiping_volumes_extruders` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wiping-volume interpretation, prime-tower skip points, flat ironing, interface-feature handling, interface cooldown, infill-gap application, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `flush_into_infill`, `flush_into_support`, `flush_into_objects`, and following options from `PrintConfig.cpp:6847+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
