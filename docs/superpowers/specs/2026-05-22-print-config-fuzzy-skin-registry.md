# M81 Spec: PrintConfig fuzzy-skin registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` fuzzy-skin option-definition slice into `ares-core` option registry metadata by adding registry coverage for the twelve `fuzzy_skin*` options from `PrintConfig.cpp:3420-3576`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`, `PrintConfig.cpp:192-200`, `PrintConfig.hpp:1108`, `PrintConfig.cpp:3420-3439`: `fuzzy_skin` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1109`, `PrintConfig.cpp:3441-3449`: `fuzzy_skin_thickness` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1110`, `PrintConfig.cpp:3451-3459`: `fuzzy_skin_point_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1111`, `PrintConfig.cpp:3461-3466`: `fuzzy_skin_first_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:59-63`, `PrintConfig.cpp:218-223`, `PrintConfig.hpp:1113`, `PrintConfig.cpp:3468-3489`: `fuzzy_skin_mode` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:65-72`, `PrintConfig.cpp:202-210`, `PrintConfig.hpp:1112`, `PrintConfig.cpp:3491-3515`: `fuzzy_skin_noise_type` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1114`, `PrintConfig.cpp:3517-3525`: `fuzzy_skin_scale` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1115`, `PrintConfig.cpp:3527-3534`: `fuzzy_skin_octaves` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1116`, `PrintConfig.cpp:3536-3543`: `fuzzy_skin_persistence` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1117`, `PrintConfig.cpp:3545-3551`: `fuzzy_skin_ripples_per_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1118`, `PrintConfig.cpp:3553-3565`: `fuzzy_skin_ripple_offset` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1119`, `PrintConfig.cpp:3567-3576`: `fuzzy_skin_layers_between_ripple_offset` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode/enum label metadata beyond the current registry boundary.
- Fuzzy-skin geometry generation, random/noise displacement, ripple behavior, validation, first-layer filtering, and runtime option interpretation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3578+`: `filter_out_gap_fill` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for all twelve `fuzzy_skin*` keys after `full_fan_speed_layer` and before `gap_fill_flow_ratio`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod fuzzy;`.
- `crates/ares-core/src/options/registry/tests/metadata/fuzzy.rs`: source metadata assertions for all twelve options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_fuzzy;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_fuzzy.rs`: public lookup coverage for all twelve options.
- `docs/roadmap.md` and `docs/milestones/m81-print-config-fuzzy-skin-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `fuzzy_skin` (`coEnum`, default `disabled_fuzzy`, field at `PrintConfig.hpp:1108`, enum at `PrintConfig.hpp:50-57`, enum map at `PrintConfig.cpp:192-200`, definition lines 3420-3439, Ares kind `Enum`)
- `fuzzy_skin_thickness` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:1109`, definition lines 3441-3449, Ares kind `Float`)
- `fuzzy_skin_point_distance` (`coFloat`, default `0.3`, field at `PrintConfig.hpp:1110`, definition lines 3451-3459, Ares kind `Float`)
- `fuzzy_skin_first_layer` (`coBool`, default `false`, field at `PrintConfig.hpp:1111`, definition lines 3461-3466, Ares kind `Bool`)
- `fuzzy_skin_mode` (`coEnum`, default `displacement`, field at `PrintConfig.hpp:1113`, enum at `PrintConfig.hpp:59-63`, enum map at `PrintConfig.cpp:218-223`, definition lines 3468-3489, Ares kind `Enum`)
- `fuzzy_skin_noise_type` (`coEnum`, default `classic`, field at `PrintConfig.hpp:1112`, enum at `PrintConfig.hpp:65-72`, enum map at `PrintConfig.cpp:202-210`, definition lines 3491-3515, Ares kind `Enum`)
- `fuzzy_skin_scale` (`coFloat`, default `1`, field at `PrintConfig.hpp:1114`, definition lines 3517-3525, Ares kind `Float`)
- `fuzzy_skin_octaves` (`coInt`, default `4`, field at `PrintConfig.hpp:1115`, definition lines 3527-3534, Ares kind `Int`)
- `fuzzy_skin_persistence` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1116`, definition lines 3536-3543, Ares kind `Float`)
- `fuzzy_skin_ripples_per_layer` (`coInt`, default `15`, field at `PrintConfig.hpp:1117`, definition lines 3545-3551, Ares kind `Int`)
- `fuzzy_skin_ripple_offset` (`coPercent`, default `50`, field at `PrintConfig.hpp:1118`, definition lines 3553-3565, Ares kind `Percent`)
- `fuzzy_skin_layers_between_ripple_offset` (`coInt`, default `1`, field at `PrintConfig.hpp:1119`, definition lines 3567-3576, Ares kind `Int`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using `Enum`, `Float`, `Bool`, `Int`, and `Percent`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, fuzzy-skin runtime behavior, validation, random/noise displacement, ripple behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `filter_out_gap_fill` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create new focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3420-3576` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; enum key/default identity is preserved through string defaults and source citations only.
- Fuzzy-skin geometry generation, random/noise displacement, ripple behavior, validation, first-layer filtering, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filter_out_gap_fill` and following options from `PrintConfig.cpp:3578+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all twelve new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all twelve new keys.
- Plan/spec explicitly account for deferred UI/bounds/enum-label metadata, fuzzy-skin runtime behavior, slicing/extrusion/G-code behavior, and following `filter_out_gap_fill` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
