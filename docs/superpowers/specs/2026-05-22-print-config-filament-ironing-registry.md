# M80 Spec: PrintConfig filament ironing override registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament-specific ironing override option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_ironing_flow`, `filament_ironing_spacing`, `filament_ironing_inset`, and `filament_ironing_speed`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1148`: `filament_ironing_flow` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3372-3383`: `filament_ironing_flow` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1149`: `filament_ironing_spacing` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3385-3395`: `filament_ironing_spacing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1150`: `filament_ironing_inset` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3397-3407`: `filament_ironing_inset` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1151`: `filament_ironing_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3409-3418`: beginning of `filament_ironing_speed` option definition, continuing immediately through its nullable default before `fuzzy_skin` starts at `PrintConfig.cpp:3420`.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary, except nullable type identity.
- Ironing runtime behavior and filament override resolution.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3420+`: `fuzzy_skin` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add metadata-only `OptionValueKind::PercentsNullable` for upstream `coPercents` plus `nullable = true` metadata.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definitions for the four `filament_ironing_*` keys.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend source metadata assertions for the four options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the four options.
- `docs/roadmap.md` and `docs/milestones/m80-print-config-filament-ironing-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_ironing_flow` (`coPercents`, `nullable = true`, `ConfigOptionPercentsNullable::nil_value()` default, field at `PrintConfig.hpp:1148`, definition lines 3372-3383, Ares kind `PercentsNullable`, default string `nil`)
- `filament_ironing_spacing` (`coFloats`, `nullable = true`, `ConfigOptionFloatsNullable::nil_value()` default, field at `PrintConfig.hpp:1149`, definition lines 3385-3395, Ares kind `FloatsNullable`, default string `nil`)
- `filament_ironing_inset` (`coFloats`, `nullable = true`, `ConfigOptionFloatsNullable::nil_value()` default, field at `PrintConfig.hpp:1150`, definition lines 3397-3407, Ares kind `FloatsNullable`, default string `nil`)
- `filament_ironing_speed` (`coFloats`, `nullable = true`, `ConfigOptionFloatsNullable::nil_value()` default, field at `PrintConfig.hpp:1151`, definition lines 3409-3418, Ares kind `FloatsNullable`, default string `nil`)

## Functional requirements

1. Add metadata-only `OptionValueKind::PercentsNullable`; do not add parsing/runtime behavior for it in this milestone.
2. Add the included missing options to existing sorted definition shards using `PercentsNullable` and `FloatsNullable`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, ironing behavior, filament override behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `fuzzy_skin` or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3372-3418` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable identity is preserved through `PercentsNullable`/`FloatsNullable` only.
- Ironing runtime behavior, filament override resolution, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `fuzzy_skin` and following options from `PrintConfig.cpp:3420+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, nil default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for `PercentsNullable`, deferred UI/bounds metadata, ironing runtime behavior, slicing/extrusion/G-code behavior, and following fuzzy-skin scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
