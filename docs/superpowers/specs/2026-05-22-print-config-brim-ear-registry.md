# M45 Spec: PrintConfig brim ear option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` brim-ear option-definition slice into `ares-core` option registry metadata, covering brim-ear enablement, detection length, and max angle without changing brim-ear geometry detection, brim generation, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:925-926`: fields for `brim_ears_detection_length` and `brim_ears_max_angle`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1665-1693`: `PrintConfigDef::init_fff_params()` option definitions for this slice, including the `brim_ears` key.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/Brim.*` and `SkirtBrim.*`: brim-ear path generation and brim placement.
- `OrcaSlicer/src/libslic3r/Geometry.*`: sharp-angle detection and geometry decimation for brim ears.
- `OrcaSlicer/src/libslic3r/GCode.cpp`: downstream extrusion/G-code effects.
- `OrcaSlicer/src/libslic3r/Preset.cpp`: preset option-list behavior and UI visibility rules.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `brim_ears`, `brim_ears_detection_length`, and `brim_ears_max_angle`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/brim.rs`: focused brim metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `brim_ears` (`coBool`, default `false`, lines 1665-1670)
- `brim_ears_max_angle` (`coFloat`, default `125`, lines 1672-1682)
- `brim_ears_detection_length` (`coFloat`, default `1`, lines 1684-1693)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Bool` and `OptionValueKind::Float`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, brim-ear sharp-edge detection, detection-radius decimation, max-angle behavior, brim generation behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter compatible-printer/profile options from `PrintConfig.cpp:1695+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M46, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/sidetext/min/max/mode metadata from `PrintConfig.cpp:1665-1693` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Brim-ear sharp-edge detection, geometry decimation, max-angle filtering, and brim path generation are deferred to later source-cited brim/skirt geometry milestones.
- Extrusion behavior and G-code behavior are deferred.
- Preset option-list behavior in `Preset.cpp` and object override handling are deferred.
- Compatible-printer/profile options from `PrintConfig.cpp:1695+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, brim-ear behavior, geometry behavior, extrusion/G-code behavior, preset-list behavior, object override behavior, and following compatible-profile options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
