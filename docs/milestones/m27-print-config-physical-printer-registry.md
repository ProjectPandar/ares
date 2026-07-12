# M27: PrintConfig physical printer option registry

## Goal
Port the physical-printer common `libslic3r::PrintConfigDef::init_common_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:786-894` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp` `AuthorizationType` plus the listed `PrintConfig.cpp` lines; no new Ares pipeline, crate, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- Registry definitions are split so modified Rust files remain under 400 LOC.
- `OptionValueKind` can represent upstream `coStrings`.
- `OPTION_DEFINITIONS` includes the thirteen M27 options with exact defaults and source line ranges.
- `preset_names` duplicate upstream definitions are accounted for in the source reference.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Typed accessors/behavior for the new options remain deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
