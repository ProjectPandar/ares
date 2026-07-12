# M157: PrintConfig SLA foot and gamma correction registry

## Goal
Port the next SLA printer correction options from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7351-7367` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1843-1844`, `PrintConfig.cpp:7351-7367`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA foot/gamma correction behavior, UI behavior, geometry behavior, extrusion behavior, slicing behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `elefant_foot_min_width` and `gamma_correction` with exact kinds, defaults, and source line ranges.
- Existing `elefant_foot_compensation` remains unchanged because it is already present from the earlier shared PrintConfig option definition.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for SLA foot/gamma correction, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- SLA material settings from `PrintConfig.cpp:7370+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
