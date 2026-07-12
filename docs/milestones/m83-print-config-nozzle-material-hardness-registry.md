# M83: PrintConfig nozzle material and hardness option registry

## Goal
Port the adjacent FFF nozzle material/hardness option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3652-3679` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `CommonDefs.hpp:12-20`, `PrintConfig.hpp:338-353,1402-1403`, `PrintConfig.cpp:485-492,3652-3679`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, nozzle material compatibility behavior, nozzle hardness validation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes metadata-only `EnumsNullable` for Orca `coEnums` plus nullable/generic nullable registry metadata.
- `OPTION_DEFINITIONS` includes `nozzle_type` and `nozzle_hrc` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream enum labels, UI label/tooltip/sidetext/min/max/mode/nullable metadata remains deferred beyond the current metadata boundary, except preserving nullable enum-vector type identity through `EnumsNullable`.
- Runtime behavior for nozzle material compatibility, nozzle hardness checking, slicing, extrusion, and downstream G-code behavior remains deferred.
- `printer_structure` and following options from `PrintConfig.cpp:3681+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
