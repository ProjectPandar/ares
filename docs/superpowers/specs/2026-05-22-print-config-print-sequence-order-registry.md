# M47 Spec: PrintConfig print sequence and order option registry slice

## Goal
Port the adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` print sequence/order option-definition slice into `ares-core` option registry metadata, covering `print_sequence` and `print_order` without changing print scheduling, object ordering, slicing, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:148-159`: `PrintSequence` and `PrintOrder` enum definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1505-1506`: config fields for `print_sequence` and `print_order`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:293-303`: enum key maps for `PrintSequence` and `PrintOrder`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1750-1770`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8318-8467`: downstream validation/use of `print_sequence`.
- Print-object scheduling, object-by-object constraints, intra-layer object ordering, slicing, extrusion, and G-code behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1772+`: cooling, acceleration, default profile, and following options.
- UI labels, enum labels, mode behavior, and profile/preset behavior beyond registry metadata.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definitions for `print_order` and `print_sequence`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/profile.rs`: focused metadata assertions for print sequence/order.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `print_sequence` (`coEnum`, default `by layer`, lines 1750-1759; enum keys at lines 293-297; field at `PrintConfig.hpp:1505`)
- `print_order` (`coEnum`, default `default`, lines 1761-1770; enum keys at lines 299-303; field at `PrintConfig.hpp:1506`)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Enum`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, print scheduling behavior, object-by-object constraints, intra-layer object ordering, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `slow_down_for_layer_cooling`, `default_acceleration`, `default_filament_profile`, `default_print_profile`, air-filtration, fan-speed, or following options from `PrintConfig.cpp:1772+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M48, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/tooltip/enum-label/mode metadata from `PrintConfig.cpp:1750-1770` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Print-sequence scheduling, object-by-object validation, intra-layer object ordering, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited print lifecycle milestones.
- Preset/profile behavior and UI behavior are deferred.
- Cooling, acceleration, default-profile, air-filtration, fan-speed, and following options from `PrintConfig.cpp:1772+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, print scheduling, object ordering, slicing/extrusion/G-code behavior, preset/profile behavior, and following cooling/acceleration/profile options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
