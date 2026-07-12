# M117: PrintConfig support Z-distance and enforced layers registry

## Goal
Port the adjacent support top/bottom Z-distance and enforced-support-layers option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6025` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:956-958`, `PrintConfig.cpp:5981-6025`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, Z-gap behavior, independent support layer-height rounding behavior, enforced support generation behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_top_z_distance`, `support_bottom_z_distance`, and `enforce_support_layers` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for support Z-gap application, layer-height rounding, enforced support generation, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_filament` and following support options from `PrintConfig.cpp:6027+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
