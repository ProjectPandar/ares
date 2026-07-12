# M148 Spec: PrintConfig minimum feature and wall-length registry slice

## Goal
Port the adjacent minimum feature and minimum wall-length option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1025`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051-7060`: `min_feature_size` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1039`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7062-7074`: `min_length_factor` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max metadata beyond the current registry metadata boundary.
- Minimum feature filtering/widening, short wall pruning, Arachne/classic perimeter generation, and wall-transition geometry/planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076+`: `wall_maximum_resolution`, `wall_maximum_deviation`, `initial_layer_min_bead_width`, `min_bead_width`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `min_feature_size` before `min_layer_height`; add `min_length_factor` after `min_layer_height` and before `min_resonance_avoidance_speed`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `min_feature_size` before `min_layer_height`; add `min_length_factor` after `min_layer_height` and before `min_resonance_avoidance_speed`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/min_feature_length.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_min_feature_length.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/min_feature_length_values.rs`: move the existing adjacent max-volumetric fixture rows into the helper and add known-count fixture values for both covered keys there, keeping `values.rs` below 400 LOC.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by two.
- `docs/roadmap.md` and `docs/milestones/m148-print-config-min-feature-length-registry.md`: milestone sequencing docs.

## Included option definitions

- `min_feature_size` (`coPercent`, default `25`, field at `PrintConfig.hpp:1025`, definition lines 7051-7060, Ares kind `Percent`)
- `min_length_factor` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1039`, definition lines 7062-7074, Ares kind `Float`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, minimum feature filtering/widening, short wall pruning, Arachne/classic perimeter behavior, wall-transition planning/geometry behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wall_maximum_resolution`, `wall_maximum_deviation`, `initial_layer_min_bead_width`, `min_bead_width`, or following options from `PrintConfig.cpp:7076+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, minimum-feature/short-wall behavior, and following `PrintConfig.cpp:7076+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
