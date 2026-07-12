# M164 Spec: PrintConfig SLA support points registry slice

## Goal
Port the automatic SLA support-points settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1729-1731`, `PrintConfig.cpp:7696-7710`: automatic SLA support-points density and minimal-distance option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/min metadata beyond the current registry metadata boundary.
- Automatic support-point placement, SLA support generation, support geometry, support placement, and runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7712+`: `pad_enable`, pad settings, and later SLA settings.
- `material_print_speed` from `PrintConfig.cpp:7855-7864`, which is outside this source slice.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add sorted `support_points_density_relative` and `support_points_minimal_distance` to the existing support shard.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_support_points.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_support_points.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 2.
- `docs/roadmap.md` and `docs/milestones/m164-print-config-sla-support-points-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_points_density_relative` (`coInt`, default `100`, field at `PrintConfig.hpp:1730`, definition lines 7696-7702, Ares kind `Int`)
- `support_points_minimal_distance` (`coFloat`, default `1`, field at `PrintConfig.hpp:1731`, definition lines 7704-7710, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `pad_enable` beginning at `PrintConfig.cpp:7712` is deferred to a later source-cited SLA pad milestone.
- Pad wall/brim/merge/slope/around-object settings and later SLA settings following `pad_enable` are deferred.
- `material_print_speed` from `PrintConfig.cpp:7855-7864` remains deferred.
- Runtime automatic support-point placement, SLA support generation, support geometry, and pad generation behavior is deferred.

## Functional requirements

1. Add the 2 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, automatic support-point placement behavior, SLA support-generation behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `pad_enable` or later SLA settings from `PrintConfig.cpp:7712+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA support runtime behavior and `PrintConfig.cpp:7712+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
