# M26: PrintConfig common params option registry

## Goal
Port the first common `libslic3r::PrintConfigDef::init_common_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:672-782` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp` plus the listed `PrintConfig.cpp` lines; no new Ares pipeline, crate, or slicing behavior is added.

## Exit checklist
- The upstream `layer_height` definition inside `PrintConfig.cpp:672-782` is accounted for as already covered by prior registry/accessor work.
- `OptionValueKind` can represent the included upstream config kinds: string, points, and point groups.
- `OPTION_DEFINITIONS` includes the twelve M26 options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Typed accessors/behavior for the new options remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
