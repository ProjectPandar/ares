# M150 Spec: PrintConfig bead-width registry slice

## Goal
Port the adjacent first-layer minimum bead-width and minimum bead-width option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1026`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099-7107`: `initial_layer_min_bead_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1027`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7109-7119`: `min_bead_width` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min metadata beyond the current registry metadata boundary.
- First-layer bead-width selection, minimum wall-width replacement, thin-feature widening, Arachne/classic perimeter generation, and wall-transition geometry/planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7121+`: filament extruder override nullable option generation and following behavior.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle_independent.rs`: add `initial_layer_min_bead_width` after `initial_layer_line_width` and before `initial_layer_print_height`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `min_bead_width` after `max_volumetric_extrusion_rate_slope_segment_length` and before `min_feature_size`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add `initial_layer_min_bead_width` after `initial_layer_line_width` and before `initial_layer_print_height`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `min_bead_width` after `max_volumetric_extrusion_rate_slope_segment_length` and before `min_feature_size`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/bead_width.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_bead_width.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs`: add `initial_layer_min_bead_width` fixture near adjacent initial-layer values.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/min_feature_length_values.rs`: add `min_bead_width` fixture near adjacent min-feature values.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by two.
- `docs/roadmap.md` and `docs/milestones/m150-print-config-bead-width-registry.md`: milestone sequencing docs.

## Included option definitions

- `initial_layer_min_bead_width` (`coPercent`, default `85`, field at `PrintConfig.hpp:1026`, definition lines 7099-7107, Ares kind `Percent`)
- `min_bead_width` (`coPercent`, default `85`, field at `PrintConfig.hpp:1027`, definition lines 7109-7119, Ares kind `Percent`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, first-layer bead-width selection, minimum wall-width replacement, thin-feature widening, Arachne/classic perimeter behavior, wall-transition planning/geometry behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add filament extruder override nullable option generation or following behavior from `PrintConfig.cpp:7121+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, bead-width/minimum-wall behavior, and following `PrintConfig.cpp:7121+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
