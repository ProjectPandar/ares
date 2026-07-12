# M114: PrintConfig wipe-tower and ramming registry

## Goal
Port the adjacent wipe-tower type, purge/ramming, tool-change-on-wipe-tower, and sparse-layer wipe-tower option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5821-5861` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:74-77`, `PrintConfig.cpp:212-216`, `PrintConfig.hpp:1391`, `PrintConfig.hpp:1457-1460`, `PrintConfig.cpp:5821-5861`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower generation behavior, ramming behavior, tool-change travel behavior, sparse-layer wipe-tower behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wipe_tower_type`, `purge_in_prime_tower`, `enable_filament_ramming`, `tool_change_on_wipe_tower`, and `wipe_tower_no_sparse_layers` with exact kinds, defaults, and source line ranges.
- `wipe_tower_type` cites the upstream `WipeTowerType` enum map and uses default `type2`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- Any mechanical registry-table shard split only preserves sorted order and keeps modified Rust files below 400 LOC.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for wipe-tower implementation selection, purge-in-prime-tower handling, filament ramming, tool-change travel, sparse-layer wipe-tower suppression, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `single_extruder_multi_material_priming` and following options from `PrintConfig.cpp:5863+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
