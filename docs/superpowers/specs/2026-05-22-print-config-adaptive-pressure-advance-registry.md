# M57 Spec: PrintConfig adaptive pressure advance option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` adaptive pressure advance option-definition slice into `ares-core` option registry metadata by adding registry coverage for `adaptive_pressure_advance`, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs`, and `adaptive_pressure_advance_bridges`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1305`: `adaptive_pressure_advance` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1306`: `adaptive_pressure_advance_overhangs` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1307`: `adaptive_pressure_advance_model` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1308`: `adaptive_pressure_advance_bridges` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2264-2278`: `adaptive_pressure_advance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2280-2303`: `adaptive_pressure_advance_model` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2305-2311`: `adaptive_pressure_advance_overhangs` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2313-2320`: `adaptive_pressure_advance_bridges` option definition.

Related upstream behavior explicitly deferred:

- Runtime adaptive pressure advance and firmware-specific pressure advance behavior.
- Calibration-model parsing, fitting, validation, and G-code emission behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/mode/multiline/full-width/height/max metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2322+`: `line_width` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for all four adaptive pressure advance options.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/pressure.rs`: metadata assertions for adaptive pressure advance options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_pressure.rs`: public lookup coverage for adaptive pressure advance options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `adaptive_pressure_advance` (`coBools`, default `false`, field at `PrintConfig.hpp:1305`, definition lines 2264-2278)
- `adaptive_pressure_advance_model` (`coStrings`, default `0,0,0\n0,0,0`, field at `PrintConfig.hpp:1307`, definition lines 2280-2303)
- `adaptive_pressure_advance_overhangs` (`coBools`, default `false`, field at `PrintConfig.hpp:1306`, definition lines 2305-2311)
- `adaptive_pressure_advance_bridges` (`coFloats`, default `0`, field at `PrintConfig.hpp:1308`, definition lines 2313-2320)

## Functional requirements

1. Add the included missing options to sorted definition shards using `OptionValueKind::Bools`, `OptionValueKind::Strings`, and `OptionValueKind::Floats`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, runtime adaptive pressure advance behavior, calibration-model parsing/fitting, firmware-specific behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `line_width` or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M58.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2264-2320` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime adaptive pressure advance, calibration-model parsing/fitting, firmware-specific behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `line_width` and following options from `PrintConfig.cpp:2322+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime adaptive pressure advance behavior, calibration-model behavior, slicing/extrusion/G-code behavior, and following `line_width` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
