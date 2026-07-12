# M149 Spec: PrintConfig wall maximum resolution registry slice

## Goal
Port the adjacent wall maximum resolution/deviation option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1030`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076-7085`: `wall_maximum_resolution` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1031`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7087-7097`: `wall_maximum_deviation` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max metadata beyond the current registry metadata boundary.
- Wall path simplification, maximum deviation enforcement, Arachne/classic perimeter generation, and wall-transition geometry/planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099+`: `initial_layer_min_bead_width`, `min_bead_width`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wall_maximum_deviation` and `wall_maximum_resolution` after `wall_loops` and before `wall_sequence`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the same keys in matching sorted positions after `wall_loops` and before `wall_sequence`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wall_maximum.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wall_maximum.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for both covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by two.
- `docs/roadmap.md` and `docs/milestones/m149-print-config-wall-maximum-registry.md`: milestone sequencing docs.

## Included option definitions

- `wall_maximum_resolution` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1030`, definition lines 7076-7085, Ares kind `Float`)
- `wall_maximum_deviation` (`coFloat`, default `0.025`, field at `PrintConfig.hpp:1031`, definition lines 7087-7097, Ares kind `Float`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wall simplification, maximum deviation enforcement, Arachne/classic perimeter behavior, wall-transition planning/geometry behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `initial_layer_min_bead_width`, `min_bead_width`, or following options from `PrintConfig.cpp:7099+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, wall maximum behavior, and following `PrintConfig.cpp:7099+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
