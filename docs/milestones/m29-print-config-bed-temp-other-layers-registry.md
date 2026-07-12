# M29: PrintConfig bed temperature other-layers option registry

## Goal
Port the FFF other-layer bed temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:923-982` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp` `ConfigOptionInts` fields and the listed `PrintConfig.cpp` lines; no new Ares pipeline, crate, temperature G-code, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OptionValueKind` can represent upstream `coInts`.
- `OPTION_DEFINITIONS` includes the six other-layer bed-temperature options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Initial-layer temperature options and temperature G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
