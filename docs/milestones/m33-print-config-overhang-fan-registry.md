# M33: PrintConfig overhang fan option registry

## Goal
Port the FFF overhang/bridge fan `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1170-1211` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:304-310,1502-1504`, `PrintConfig.cpp:456-464`, and `PrintConfig.cpp:1170-1211`; no new Ares pipeline, crate, cooling behavior, bridge-detection behavior, fan G-code behavior, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OptionValueKind` can represent upstream `coBools` and `coEnums`.
- `OPTION_DEFINITIONS` includes the three overhang fan options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Bridge angle options, cooling decisions, bridge-detection integration, fan speed planning, and fan G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
