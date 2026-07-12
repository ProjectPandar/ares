# M158: PrintConfig SLA material identity and cost registry

## Goal
Port the first SLA material settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7370-7423` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1811-1814`, `PrintConfig.cpp:7370-7423`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA material behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `material_colour`, `material_type`, `bottle_volume`, `bottle_weight`, `material_density`, and `bottle_cost` with exact kinds, defaults, and source line ranges.
- Existing `initial_layer_height` remains unchanged because it is already present from the SLA material definition at `PrintConfig.cpp:7390-7395`.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA material identity, cost, density, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `faded_layers` and later SLA exposure/support/material settings from `PrintConfig.cpp:7425+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
