# M150: PrintConfig bead-width registry

## Goal
Port the adjacent first-layer/minimum bead-width option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099-7119` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1026-1027`, `PrintConfig.cpp:7099-7119`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, first-layer bead-width behavior, minimum wall-width behavior, thin-feature widening behavior, Arachne/classic perimeter behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `initial_layer_min_bead_width` and `min_bead_width` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for bead-width selection, minimum wall-width replacement, thin-feature widening, Arachne/classic perimeter generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- Filament extruder override nullable options and following behavior from `PrintConfig.cpp:7121+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
