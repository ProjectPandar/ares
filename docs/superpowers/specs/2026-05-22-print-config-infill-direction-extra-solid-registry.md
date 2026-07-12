# M72 Spec: PrintConfig infill direction and extra solid option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` infill direction / extra solid option-definition slice into `ares-core` option registry metadata by adding registry coverage for `solid_infill_direction`, `align_infill_direction_to_model`, `extra_solid_infills`, and `fill_multiline`, and by source-citation-aligning existing `infill_direction` and `sparse_infill_density` definitions.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1095`: `infill_direction` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2861-2869`: `infill_direction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1096`: `solid_infill_direction` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2871-2879`: `solid_infill_direction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1101`: `sparse_infill_density` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2881-2889`: `sparse_infill_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1106`: `align_infill_direction_to_model` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2891-2896`: `align_infill_direction_to_model` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1107`: `extra_solid_infills` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2898-2903`: `extra_solid_infills` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1135`: `fill_multiline` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2906-2913`: `fill_multiline` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Infill angle application, solid/sparse infill direction behavior, model-aligned fill direction behavior, extra solid infill insertion, and multiline infill runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2915+`: `gyroid_optimized`, `sparse_infill_pattern`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted `align_infill_direction_to_model`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `extra_solid_infills`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted `fill_multiline` and update existing `infill_direction` source citation to include `PrintConfig.hpp:1095`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `solid_infill_direction` and update existing `sparse_infill_density` source citation to include `PrintConfig.hpp:1101`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/strength.rs`: source metadata assertions for the six options.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: register the strength metadata test module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_strength.rs`: public lookup coverage for the six options.
- `crates/ares-core/src/options/tests.rs`: register the public strength lookup test module.
- `docs/roadmap.md` and `docs/milestones/m72-print-config-infill-direction-extra-solid-registry.md`: milestone sequencing docs.

## Included option definitions

Add or align registry metadata for these exact upstream options and default values:

- `infill_direction` (`coFloat`, default `45`, field at `PrintConfig.hpp:1095`, definition lines 2861-2869, Ares kind `Float`)
- `solid_infill_direction` (`coFloat`, default `45`, field at `PrintConfig.hpp:1096`, definition lines 2871-2879, Ares kind `Float`)
- `sparse_infill_density` (`coPercent`, default `20`, field at `PrintConfig.hpp:1101`, definition lines 2881-2889, Ares kind `Percent`)
- `align_infill_direction_to_model` (`coBool`, default `false`, field at `PrintConfig.hpp:1106`, definition lines 2891-2896, Ares kind `Bool`)
- `extra_solid_infills` (`coString`, default empty string, field at `PrintConfig.hpp:1107`, definition lines 2898-2903, Ares kind `String`)
- `fill_multiline` (`coInt`, default `1`, field at `PrintConfig.hpp:1135`, definition lines 2906-2913, Ares kind `Int`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Bool`, `String`, `Int`, and `Float` as appropriate.
2. Preserve kind/default for existing `infill_direction` and `sparse_infill_density` while updating their source citations to include the matching `PrintConfig.hpp` fields.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, infill-angle behavior, model-aligned direction behavior, extra solid infill behavior, multiline infill behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `gyroid_optimized`, `sparse_infill_pattern`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, and bounds metadata from `PrintConfig.cpp:2861-2913` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Infill direction behavior, model-aligned fill direction, extra solid infill insertion, multiline infill, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `gyroid_optimized`, `sparse_infill_pattern`, and following options from `PrintConfig.cpp:2915+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six included keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all six included keys.
- Plan/spec explicitly account for deferred UI/bounds metadata, infill direction/extra solid/multiline behavior, slicing/extrusion/G-code behavior, and following `gyroid_optimized` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
