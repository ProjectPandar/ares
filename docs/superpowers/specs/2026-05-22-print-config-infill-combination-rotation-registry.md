# M87 Spec: PrintConfig infill combination and rotation-template registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` infill-combination, infill-shift, sparse-infill rotation-template, and solid-infill rotation-template option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1132`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3853-3860`: `infill_combination` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1099`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3862-3870`: `infill_shift_step` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1100`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3872-3884`: `sparse_infill_rotate_template` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1097`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3886-3896`: `solid_infill_rotate_template` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/mode metadata beyond the current registry boundary.
- Infill-combination layer merging behavior.
- Infill shift runtime geometry/path behavior.
- Sparse/solid infill rotation-template parsing and per-layer application.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3898+`: `skeleton_infill_density` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `infill_combination` and `infill_shift_step`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `sparse_infill_rotate_template` and `solid_infill_rotate_template`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod infill;`.
- `crates/ares-core/src/options/registry/tests/metadata/infill.rs`: source metadata assertions for all four options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_infill;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_infill.rs`: public lookup coverage for all four options.
- `docs/roadmap.md` and `docs/milestones/m87-print-config-infill-combination-rotation-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `infill_combination` (`coBool`, default `false`, field at `PrintConfig.hpp:1132`, definition lines 3853-3860, Ares kind `Bool`)
- `infill_shift_step` (`coFloat`, default `0.4`, field at `PrintConfig.hpp:1099`, definition lines 3862-3870, Ares kind `Float`)
- `sparse_infill_rotate_template` (`coString`, default `""`, field at `PrintConfig.hpp:1100`, definition lines 3872-3884, Ares kind `String`)
- `solid_infill_rotate_template` (`coString`, default `""`, field at `PrintConfig.hpp:1097`, definition lines 3886-3896, Ares kind `String`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, infill-combination behavior, infill-shift behavior, rotation-template parsing/application, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `skeleton_infill_density` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3853-3896` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Infill combination, infill shift, rotation-template parsing/application, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `skeleton_infill_density` and following options from `PrintConfig.cpp:3898+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `skeleton_infill_density` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
