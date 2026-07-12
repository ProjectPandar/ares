# M90 Spec: PrintConfig inheritance, MMU interlocking, and calibration flag registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` profile inheritance, interface-shell, MMU segmented-region, interlocking, and flowrate-calibration flag option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4063-4069`: `inherits` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4071-4075`: `inherits_group` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:935`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4077-4084`: `interface_shells` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:937`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4086-4093`: `mmu_segmented_region_max_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:938`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4095-4104`: `mmu_segmented_region_interlocking_depth` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1062`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4106-4111`: `interlocking_beam` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1063`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4113-4120`: `interlocking_beam_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1064`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4122-4130`: `interlocking_orientation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1065`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4132-4138`: `interlocking_beam_layer_count` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1066`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4140-4146`: `interlocking_depth` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1067`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4148-4154`: `interlocking_boundary_avoidance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1070`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4156-4159`: `calib_flowrate_topinfill_special_order` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/mode/full-width/height/cli metadata beyond the current registry boundary.
- Profile inheritance and inheritance-group resolution.
- Interface-shell generation behavior.
- MMU segmented-region and interlocking geometry behavior.
- Flowrate calibration special-order behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4161+`: `ironing_type` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definition for `calib_flowrate_topinfill_special_order`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `inherits` and `inherits_group`, then keep the shard below the 400 LOC threshold.
- `crates/ares-core/src/options/registry/definitions/table/middle_tail.rs`: split the existing internal-tail definitions out of `middle.rs` and add `interface_shells`, `interlocking_beam`, `interlocking_beam_layer_count`, `interlocking_beam_width`, `interlocking_boundary_avoidance`, `interlocking_depth`, and `interlocking_orientation` in sorted order without changing unrelated moved metadata.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merge the new `middle_tail` shard between `middle` and `late` to preserve sorted lookup order.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for `mmu_segmented_region_interlocking_depth` and `mmu_segmented_region_max_width`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod interlocking;`.
- `crates/ares-core/src/options/registry/tests/metadata/interlocking.rs`: source metadata assertions for all twelve options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_interlocking;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_interlocking.rs`: public lookup coverage for all twelve options.
- `docs/roadmap.md` and `docs/milestones/m90-print-config-inheritance-interlocking-registry.md`: milestone sequencing docs.

## Included option definitions

- `inherits` (`coString`, default `""`, definition lines 4063-4069, Ares kind `String`)
- `inherits_group` (`coStrings`, default `""`, definition lines 4071-4075, Ares kind `Strings`)
- `interface_shells` (`coBool`, default `false`, field at `PrintConfig.hpp:935`, definition lines 4077-4084, Ares kind `Bool`)
- `mmu_segmented_region_max_width` (`coFloat`, default `0`, field at `PrintConfig.hpp:937`, definition lines 4086-4093, Ares kind `Float`)
- `mmu_segmented_region_interlocking_depth` (`coFloat`, default `0`, field at `PrintConfig.hpp:938`, definition lines 4095-4104, Ares kind `Float`)
- `interlocking_beam` (`coBool`, default `false`, field at `PrintConfig.hpp:1062`, definition lines 4106-4111, Ares kind `Bool`)
- `interlocking_beam_width` (`coFloat`, default `0.8`, field at `PrintConfig.hpp:1063`, definition lines 4113-4120, Ares kind `Float`)
- `interlocking_orientation` (`coFloat`, default `22.5`, field at `PrintConfig.hpp:1064`, definition lines 4122-4130, Ares kind `Float`)
- `interlocking_beam_layer_count` (`coInt`, default `2`, field at `PrintConfig.hpp:1065`, definition lines 4132-4138, Ares kind `Int`)
- `interlocking_depth` (`coInt`, default `2`, field at `PrintConfig.hpp:1066`, definition lines 4140-4146, Ares kind `Int`)
- `interlocking_boundary_avoidance` (`coInt`, default `2`, field at `PrintConfig.hpp:1067`, definition lines 4148-4154, Ares kind `Int`)
- `calib_flowrate_topinfill_special_order` (`coBool`, default `false`, field at `PrintConfig.hpp:1070`, definition lines 4156-4159, Ares kind `Bool`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, inheritance behavior, interface-shell behavior, MMU interlocking behavior, calibration behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `ironing_type` or following options from `PrintConfig.cpp:4161+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC by splitting `middle` into a focused `middle_tail` shard when M90 pushes the existing file over the limit; create focused interlocking tests instead of growing unrelated near-limit files.

## Acceptance checks

- Registry tests prove all twelve new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all twelve new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `ironing_type` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
