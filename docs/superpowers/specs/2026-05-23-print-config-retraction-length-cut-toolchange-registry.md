# M103 Spec: PrintConfig retraction length, cut, and toolchange registry slice

## Goal
Port the adjacent base retraction length, long retraction when cut/extruder-change, retraction distance, and toolchange retraction option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1368`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5068-5075`: `retraction_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1370`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5077-5079`: `enable_long_retraction_when_cut` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1372`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5081-5086`: `long_retractions_when_cut` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1371`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5088-5094`: `retraction_distances_when_cut` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1374`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5096-5100`: `long_retractions_when_ec` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1373`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5102-5109`: `retraction_distances_when_ec` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1369`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5111-5120`: `retract_length_toolchange` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/nullable metadata beyond the current registry boundary.
- Retraction planning, filament-cut long retraction behavior, extruder-change long retraction behavior, toolchange retraction behavior, and emitted G-code movement changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122+`: `z_hop`, Z-hop boundaries/types, travel slope, lift enforcement, extruder/nozzle volume type, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `enable_long_retraction_when_cut` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add `long_retractions_when_cut` and `long_retractions_when_ec` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `retract_length_toolchange`, `retraction_distances_when_cut`, `retraction_distances_when_ec`, and `retraction_length` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all seven definitions.
- `docs/roadmap.md` and `docs/milestones/m103-print-config-retraction-length-cut-toolchange-registry.md`: milestone sequencing docs.

## Included option definitions

- `retraction_length` (`coFloats`, default `0.8`, field at `PrintConfig.hpp:1368`, definition lines 5068-5075, Ares kind `Floats`)
- `enable_long_retraction_when_cut` (`coInt`, default `0`, field at `PrintConfig.hpp:1370`, definition lines 5077-5079, Ares kind `Int`)
- `long_retractions_when_cut` (`coBools`, default `false`, field at `PrintConfig.hpp:1372`, definition lines 5081-5086, Ares kind `Bools`)
- `retraction_distances_when_cut` (`coFloats`, default `18`, field at `PrintConfig.hpp:1371`, definition lines 5088-5094, Ares kind `Floats`)
- `long_retractions_when_ec` (`coBools` with nullable metadata and `ConfigOptionBoolsNullable`, default `false`, field at `PrintConfig.hpp:1374`, definition lines 5096-5100, Ares kind `BoolsNullable`)
- `retraction_distances_when_ec` (`coFloats` with nullable metadata and `ConfigOptionFloatsNullable`, default `10`, field at `PrintConfig.hpp:1373`, definition lines 5102-5109, Ares kind `FloatsNullable`)
- `retract_length_toolchange` (`coFloats`, default `10`, field at `PrintConfig.hpp:1369`, definition lines 5111-5120, Ares kind `Floats`)

## Functional requirements

1. Add the seven missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, retraction planning, filament-cut behavior, extruder-change behavior, toolchange retraction behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following `z_hop`, Z-hop boundary/type, travel-slope, lift-enforcement, extruder/nozzle-volume type, or later options from `PrintConfig.cpp:5122+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the seven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all seven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, retraction/cut/toolchange runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5122+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
