# M147 Spec: PrintConfig wall-transition registry slice

## Goal
Port the adjacent wall-transition option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1021`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003-7012`: `wall_transition_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1022`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7014-7027`: `wall_transition_filter_deviation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1023`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7029-7040`: `wall_transition_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1024`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7042-7049`: `wall_distribution_count` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max metadata beyond the current registry metadata boundary.
- Arachne/classic perimeter generation and wall-transition geometry/planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051+`: `min_feature_size`, `min_length_factor`, `wall_maximum_resolution`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wall_distribution_count` after `wall_direction` and before `wall_filament`; add the three `wall_transition_...` definitions after `wall_sequence` and before `wipe`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the same keys in matching sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wall_transition.rs`: add metadata assertions for the four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wall_transition.rs`: add public lookup assertions for the four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all four covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by four.
- `docs/roadmap.md` and `docs/milestones/m147-print-config-wall-transition-registry.md`: milestone sequencing docs.

## Included option definitions

- `wall_transition_length` (`coPercent`, default `100`, field at `PrintConfig.hpp:1021`, definition lines 7003-7012, Ares kind `Percent`)
- `wall_transition_filter_deviation` (`coPercent`, default `25`, field at `PrintConfig.hpp:1022`, definition lines 7014-7027, Ares kind `Percent`)
- `wall_transition_angle` (`coFloat`, default `10`, field at `PrintConfig.hpp:1023`, definition lines 7029-7040, Ares kind `Float`)
- `wall_distribution_count` (`coInt`, default `1`, field at `PrintConfig.hpp:1024`, definition lines 7042-7049, Ares kind `Int`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, Arachne/classic perimeter behavior, wall-transition planning/geometry behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `min_feature_size`, `min_length_factor`, `wall_maximum_resolution`, or following options from `PrintConfig.cpp:7051+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split a registry test module if implementation evidence shows a file would reach 400 LOC.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, wall-transition/perimeter behavior, and following `PrintConfig.cpp:7051+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
