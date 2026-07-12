# M102 Spec: PrintConfig raft, resolution, and retraction trigger registry slice

## Goal
Port the adjacent raft support, path resolution, and initial retraction trigger option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:939`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4988-4997`: `raft_contact_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:940`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4999-5006`: `raft_expansion` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:941`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5008-5016`: `raft_first_layer_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:942`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5018-5026`: `raft_first_layer_expansion` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:943`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5028-5037`: `raft_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1549`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5039-5046`: `resolution` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1550`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5048-5053`: `retraction_minimum_travel` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1367`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5055-5060`: `retract_before_wipe` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1551`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5062-5066`: `retract_when_changing_layer` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/mode metadata beyond the current registry boundary.
- Raft generation, support raft geometry, path simplification/contour resolution behavior, retraction trigger planning, wipe planning, and layer-change retraction behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5068+`: `retraction_length`, long retraction when cut/extruder change, toolchange retraction, Z-hop, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for the nine covered options.
- Registry key, metadata, fixture-count, and public lookup tests cover all nine definitions.
- `docs/roadmap.md` and `docs/milestones/m102-print-config-raft-resolution-retraction-registry.md`: milestone sequencing docs.

## Included option definitions

- `raft_contact_distance` (`coFloat`, default `0.1`, field at `PrintConfig.hpp:939`, definition lines 4988-4997, Ares kind `Float`)
- `raft_expansion` (`coFloat`, default `1.5`, field at `PrintConfig.hpp:940`, definition lines 4999-5006, Ares kind `Float`)
- `raft_first_layer_density` (`coPercent`, default `90`, field at `PrintConfig.hpp:941`, definition lines 5008-5016, Ares kind `Percent`)
- `raft_first_layer_expansion` (`coFloat`, default `2.0`, field at `PrintConfig.hpp:942`, definition lines 5018-5026, Ares kind `Float`)
- `raft_layers` (`coInt`, default `INITIAL_RAFT_LAYERS`, currently `0` in OrcaSlicer defaults, field at `PrintConfig.hpp:943`, definition lines 5028-5037, Ares kind `Int`)
- `resolution` (`coFloat`, default `0.01`, field at `PrintConfig.hpp:1549`, definition lines 5039-5046, Ares kind `Float`)
- `retraction_minimum_travel` (`coFloats`, default `2`, field at `PrintConfig.hpp:1550`, definition lines 5048-5053, Ares kind `Floats`)
- `retract_before_wipe` (`coPercents`, default `100`, field at `PrintConfig.hpp:1367`, definition lines 5055-5060, Ares kind `Percents`)
- `retract_when_changing_layer` (`coBools`, default `false`, field at `PrintConfig.hpp:1551`, definition lines 5062-5066, Ares kind `Bools`)

## Functional requirements

1. Add the nine missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, raft generation, resolution/simplification behavior, retraction planning, wipe behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following `retraction_length`, long retraction, toolchange retraction, Z-hop, or later options from `PrintConfig.cpp:5068+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, raft/resolution/retraction runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5068+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
