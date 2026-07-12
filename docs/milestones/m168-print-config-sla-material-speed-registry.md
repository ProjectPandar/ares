# M168: PrintConfig SLA material speed registry

## Goal
Port the SLA material print speed setting from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7855-7864` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1805`, `PrintConfig.hpp:1821`, `PrintConfig.cpp:413-417`, `PrintConfig.cpp:7855-7864`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, SLA exposure/material-speed behavior, SL1 export behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `material_print_speed` with exact kind, default, enum/source citations, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- Expected-key shards remain below 400 LOC and preserve sorted concatenated order.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Runtime behavior for SLA material speed, SL1 export profile selection, slicing, extrusion planning, and downstream G-code remains unchanged/deferred.
- Later non-`init_sla_params` legacy handling from `PrintConfig.cpp:7867+` remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
