# M143 Spec: PrintConfig wipe-tower extra and idle-temperature registry slice

## Goal
Port the adjacent wipe-tower bridging/extra purge-line and idle-temperature option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1588`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6872-6877`: `wipe_tower_bridging` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1595`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6879-6886`: `wipe_tower_extra_spacing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1589`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6888-6896`: `wipe_tower_extra_flow` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1603`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6898-6905`: `idle_temperature` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/max/mode metadata beyond the current registry metadata boundary.
- Runtime wipe-tower bridging distance behavior, wipe-tower purge-line spacing/flow application, idle-temperature handling for multi-tool ooze prevention, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6907+`: `xy_hole_compensation`, `xy_contour_compensation`, `hole_to_polyhole`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle_independent.rs`: add `idle_temperature` before `independent_support_layer_height`.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs`: add `wipe_tower_bridging`, `wipe_tower_extra_flow`, and `wipe_tower_extra_spacing` in lexicographic order among existing `wipe_tower_*` definitions.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add `idle_temperature` after `hot_plate_temp_initial_layer` and before `independent_support_layer_height`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the three `wipe_tower_*` keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_extra_idle_temperature.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_extra_idle_temperature.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all four covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by four.
- `docs/roadmap.md` and `docs/milestones/m143-print-config-wipe-tower-extra-idle-temperature-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_bridging` (`coFloat`, default `10`, field at `PrintConfig.hpp:1588`, definition lines 6872-6877, Ares kind `Float`)
- `wipe_tower_extra_spacing` (`coPercent`, default `100`, field at `PrintConfig.hpp:1595`, definition lines 6879-6886, Ares kind `Percent`)
- `wipe_tower_extra_flow` (`coPercent`, default `100`, field at `PrintConfig.hpp:1589`, definition lines 6888-6896, Ares kind `Percent`)
- `idle_temperature` (`coInts`, default `0`, field at `PrintConfig.hpp:1603`, definition lines 6898-6905, Ares kind `Ints`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe-tower bridging behavior, purge-line spacing/flow behavior, idle-temperature behavior, ooze-prevention behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `xy_hole_compensation`, `xy_contour_compensation`, `hole_to_polyhole`, or following options from `PrintConfig.cpp:6907+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6907+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
