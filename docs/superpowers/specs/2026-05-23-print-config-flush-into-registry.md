# M142 Spec: PrintConfig flush-into registry slice

## Goal
Port the adjacent flush-into option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1005`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6847-6854`: `flush_into_infill` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1006`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6856-6862`: `flush_into_support` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1003`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6864-6870`: `flush_into_objects` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category metadata beyond the current registry metadata boundary.
- Runtime purging into object infill, support, or selected objects; prime-tower enablement dependency handling; color-mixing/object assignment behavior; slicing behavior; geometry behavior; extrusion behavior; and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6872+`: `wipe_tower_bridging`, `wipe_tower_extra_spacing`, `wipe_tower_extra_flow`, `idle_temperature`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add the three `flush_into_*` definitions in lexicographic order after `first_x_layer_fan_speed` and before `flush_multiplier`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add the three `flush_into_*` expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/flush_into.rs`: add metadata assertions for all three definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_flush_into.rs`: add public lookup assertions for all three definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all three covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by three.
- `docs/roadmap.md` and `docs/milestones/m142-print-config-flush-into-registry.md`: milestone sequencing docs.

## Included option definitions

- `flush_into_infill` (`coBool`, default `false`, field at `PrintConfig.hpp:1005`, definition lines 6847-6854, Ares kind `Bool`)
- `flush_into_support` (`coBool`, default `true`, field at `PrintConfig.hpp:1006`, definition lines 6856-6862, Ares kind `Bool`)
- `flush_into_objects` (`coBool`, default `false`, field at `PrintConfig.hpp:1003`, definition lines 6864-6870, Ares kind `Bool`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, purge-routing behavior, object/infill/support flush behavior, prime-tower dependency behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wipe_tower_bridging`, `wipe_tower_extra_spacing`, `wipe_tower_extra_flow`, `idle_temperature`, or following options from `PrintConfig.cpp:6872+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove all three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6872+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
