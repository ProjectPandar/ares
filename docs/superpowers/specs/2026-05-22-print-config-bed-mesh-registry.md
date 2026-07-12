# M53 Spec: PrintConfig bed mesh option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` bed mesh option-definition slice into `ares-core` option registry metadata by adding registry coverage for `bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, and `adaptive_bed_mesh_margin`, including the missing `OptionValueKind::Point` representation for upstream `coPoint` options.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1641`: `bed_mesh_min` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1642`: `bed_mesh_max` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1643`: `bed_mesh_probe_distance` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1644`: `adaptive_bed_mesh_margin` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2162-2172`: `bed_mesh_min` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2174-2184`: `bed_mesh_max` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2186-2193`: `bed_mesh_probe_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2195-2200`: `adaptive_bed_mesh_margin` option definition.

Related upstream behavior explicitly deferred:

- Adaptive bed mesh probing, min/max clamping, and probe-distance behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/sidetext/min/mode metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2202+`: `grab_length`, `extruder_colour`, `extruder_offset`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add `OptionValueKind::Point` for upstream `coPoint` metadata.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `adaptive_bed_mesh_margin`, `bed_mesh_min`, `bed_mesh_max`, and `bed_mesh_probe_distance`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/hardware.rs`: metadata assertions for bed mesh options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_hardware.rs`: public lookup coverage for Point and Float kinds.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `bed_mesh_min` (`coPoint`, default `-99999x-99999`, field at `PrintConfig.hpp:1641`, definition lines 2162-2172)
- `bed_mesh_max` (`coPoint`, default `99999x99999`, field at `PrintConfig.hpp:1642`, definition lines 2174-2184)
- `bed_mesh_probe_distance` (`coPoint`, default `50x50`, field at `PrintConfig.hpp:1643`, definition lines 2186-2193)
- `adaptive_bed_mesh_margin` (`coFloat`, default `0`, field at `PrintConfig.hpp:1644`, definition lines 2195-2200)

Default string format uses the existing registry point-list convention (`x` between coordinates, comma between multiple points); `Point` stores a single point with the same coordinate separator.

## Functional requirements

1. Add `OptionValueKind::Point` for single upstream `coPoint` option metadata.
2. Add the included missing options to sorted definition shards using `OptionValueKind::Point` and `Float`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, adaptive bed mesh behavior, probe clamping behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `grab_length`, `extruder_colour`, `extruder_offset`, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M54.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2162-2200` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Adaptive bed mesh probing, clamping, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `grab_length`, `extruder_colour`, `extruder_offset`, and following options from `PrintConfig.cpp:2202+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for the new `Point` kind, deferred upstream UI metadata, adaptive bed mesh behavior, slicing/extrusion/G-code behavior, and following `grab_length` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
