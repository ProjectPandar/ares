# M47: PrintConfig print sequence and order option registry

## Goal
Port the FFF `print_sequence` and `print_order` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1750-1770` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:148-159`, `PrintConfig.hpp:1505-1506`, `PrintConfig.cpp:293-303`, and `PrintConfig.cpp:1750-1770`; no new Ares pipeline, crate, dependency, print-order scheduling behavior, object-by-object validation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `OPTION_DEFINITIONS` includes `print_sequence` and `print_order` with exact enum kind, default values, and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/tooltip/enum labels/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Print-sequence scheduling, object-by-object constraints, intra-layer object ordering, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- Following cooling/acceleration/filament-profile options from `PrintConfig.cpp:1772+` remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
