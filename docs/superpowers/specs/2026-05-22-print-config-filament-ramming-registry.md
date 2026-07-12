# M68 Spec: PrintConfig filament ramming option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament ramming parameters and multitool ramming option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_ramming_parameters`, `filament_multitool_ramming`, `filament_multitool_ramming_volume`, and `filament_multitool_ramming_flow`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1451`: `filament_ramming_parameters` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2745-2750`: `filament_ramming_parameters` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1452`: `filament_multitool_ramming` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2752-2758`: `filament_multitool_ramming` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1453`: `filament_multitool_ramming_volume` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2760-2766`: `filament_multitool_ramming_volume` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1454`: `filament_multitool_ramming_flow` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2768-2774`: `filament_multitool_ramming_flow` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Ramming parameter parsing, validation, editing through `RammingDialog`, and runtime ramming behavior.
- Multitool ramming volume/flow application and wipe-tower behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2776+`: `filament_density`, `filament_type`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `filament_multitool_ramming`, `filament_multitool_ramming_flow`, `filament_multitool_ramming_volume`, and `filament_ramming_parameters` definitions.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the four options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the four options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_ramming_parameters` (`coStrings`, default `120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0| 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6`, field at `PrintConfig.hpp:1451`, definition lines 2745-2750, Ares kind `Strings`)
- `filament_multitool_ramming` (`coBools`, default `false`, field at `PrintConfig.hpp:1452`, definition lines 2752-2758, Ares kind `Bools`)
- `filament_multitool_ramming_volume` (`coFloats`, default `10`, field at `PrintConfig.hpp:1453`, definition lines 2760-2766, Ares kind `Floats`)
- `filament_multitool_ramming_flow` (`coFloats`, default `10`, field at `PrintConfig.hpp:1454`, definition lines 2768-2774, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to the sorted `pre_middle` definition shard using `Strings`, `Bools`, and `Floats`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, ramming parser/editor/runtime behavior, multitool ramming behavior, wipe-tower behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_density`, `filament_type`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2745-2774` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Ramming parameter parsing/editing/runtime, multitool ramming volume/flow application, wipe-tower behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_density`, `filament_type`, and following options from `PrintConfig.cpp:2776+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for deferred UI metadata, ramming parser/editor/runtime behavior, wipe-tower behavior, slicing/extrusion/G-code behavior, and following density/type scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
