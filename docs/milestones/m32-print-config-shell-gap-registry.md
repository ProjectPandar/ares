# M32: PrintConfig shell and gap-fill option registry

## Goal
Port the FFF before-layer-change G-code, bottom shell, and gap-fill target `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1110-1168` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:241-243,1038,1079-1080,1294`, `PrintConfig.cpp:393-398`, and `PrintConfig.cpp:1110-1168`; no new Ares pipeline, crate, G-code hook execution, bottom-shell behavior, gap-fill behavior, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes the four shell/gap-fill options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Overhang fan options, bridge angle options, G-code hook execution, bottom-shell behavior, and gap-fill behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
