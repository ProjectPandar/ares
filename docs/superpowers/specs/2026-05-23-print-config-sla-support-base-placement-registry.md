# M163 Spec: PrintConfig SLA support base and placement registry slice

## Goal
Port the next SLA support base/placement settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1698-1727`, `PrintConfig.cpp:7613-7694`: SLA support buildplate-only, pillar widening, base diameter/height/safety-distance, critical angle, bridge/link distance, and object elevation option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max metadata beyond the current registry metadata boundary.
- SLA support creation, support base geometry, support placement, pillar/link planning, and runtime support generation behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7696+`: automatic support point placement settings, pad settings, and later SLA settings.
- `material_print_speed` from `PrintConfig.cpp:7855-7864`, which is outside this source slice.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add sorted `support_base_diameter`, `support_base_height`, `support_base_safety_distance`, `support_buildplate_only`, `support_critical_angle`, `support_max_bridge_length`, `support_max_pillar_link_distance`, `support_object_elevation`, and `support_pillar_widening_factor` to the existing support shard.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_support_base_placement.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_support_base_placement.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 9.
- `docs/roadmap.md` and `docs/milestones/m163-print-config-sla-support-base-placement-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_buildplate_only` (`coBool`, default `false`, field at `PrintConfig.hpp:1699`, definition lines 7613-7618, Ares kind `Bool`)
- `support_pillar_widening_factor` (`coFloat`, default `0`, field at `PrintConfig.hpp:1705`, definition lines 7620-7627, Ares kind `Float`)
- `support_base_diameter` (`coFloat`, default `4`, field at `PrintConfig.hpp:1708`, definition lines 7629-7637, Ares kind `Float`)
- `support_base_height` (`coFloat`, default `1`, field at `PrintConfig.hpp:1711`, definition lines 7639-7646, Ares kind `Float`)
- `support_base_safety_distance` (`coFloat`, default `1`, field at `PrintConfig.hpp:1714`, definition lines 7648-7656, Ares kind `Float`)
- `support_critical_angle` (`coFloat`, default `45`, field at `PrintConfig.hpp:1717`, definition lines 7658-7666, Ares kind `Float`)
- `support_max_bridge_length` (`coFloat`, default `15`, field at `PrintConfig.hpp:1720`, definition lines 7668-7675, Ares kind `Float`)
- `support_max_pillar_link_distance` (`coFloat`, default `10`, field at `PrintConfig.hpp:1723`, definition lines 7677-7684, Ares kind `Float`)
- `support_object_elevation` (`coFloat`, default `5`, field at `PrintConfig.hpp:1727`, definition lines 7686-7694, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `support_points_density_relative` beginning at `PrintConfig.cpp:7696` is deferred to a later source-cited SLA support-points milestone.
- `support_points_minimal_distance`, pad settings, and later SLA settings following `support_points_density_relative` are deferred.
- `material_print_speed` from `PrintConfig.cpp:7855-7864` remains deferred.
- Runtime SLA support generation, support base geometry, support placement, and pillar/link planning behavior is deferred.

## Functional requirements

1. Add the 9 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA support-generation behavior, support base geometry, support placement, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_points_density_relative`, `support_points_minimal_distance`, or later SLA settings from `PrintConfig.cpp:7696+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA support runtime behavior and `PrintConfig.cpp:7696+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
