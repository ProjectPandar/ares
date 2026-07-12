# M73 Spec: PrintConfig gyroid optimization and sparse infill pattern option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` gyroid optimization and sparse infill pattern option-definition slice into `ares-core` option registry metadata by adding registry coverage for `gyroid_optimized` and `sparse_infill_pattern`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1136`: `gyroid_optimized` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2915-2926`: `gyroid_optimized` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1102`: `sparse_infill_pattern` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2985`: `sparse_infill_pattern` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip metadata and enum values/labels beyond the current registry boundary.
- Gyroid optimization runtime behavior and sparse infill pattern selection behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2987+`: `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted `gyroid_optimized`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `sparse_infill_pattern`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/strength.rs`: extend source metadata assertions for the two options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_strength.rs`: extend public lookup coverage for the two options.
- `docs/roadmap.md` and `docs/milestones/m73-print-config-gyroid-sparse-pattern-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `gyroid_optimized` (`coBool`, default `false`, field at `PrintConfig.hpp:1136`, definition lines 2915-2926, Ares kind `Bool`)
- `sparse_infill_pattern` (`coEnum`, default `crosshatch`, field at `PrintConfig.hpp:1102`, definition lines 2928-2985, Ares kind `Enum`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Bool` and `Enum`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, gyroid optimization behavior, sparse infill pattern runtime behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and enum value/label metadata from `PrintConfig.cpp:2915-2985` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Gyroid optimization, sparse infill pattern selection/runtime behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, and following options from `PrintConfig.cpp:2987+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for both new keys.
- Plan/spec explicitly account for deferred UI/enum metadata, gyroid optimization/sparse pattern behavior, slicing/extrusion/G-code behavior, and following `lateral_lattice_angle_1` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
