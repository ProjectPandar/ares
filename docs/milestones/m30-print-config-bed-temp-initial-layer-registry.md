# M30: PrintConfig bed temperature initial-layer option registry

## Goal
Port the FFF first-layer bed temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1496-1501` `ConfigOptionInts` fields and `PrintConfig.cpp:984-1041`; no new Ares pipeline, crate, temperature G-code, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes the six first-layer bed-temperature options with exact defaults and `PrintConfig.hpp:1496-1501` / `PrintConfig.cpp:984-1041` source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `curr_bed_type`, fan/overhang options, and temperature G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC; near-limit registry definitions are first split into `definitions/table.rs` and stored through a compact local macro rather than pushing any file past the split threshold.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
