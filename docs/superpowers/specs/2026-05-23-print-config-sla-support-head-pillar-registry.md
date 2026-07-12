# M162 Spec: PrintConfig SLA support head and pillar registry slice

## Goal
Port the first SLA support head/pillar settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:265-269`, `PrintConfig.cpp:406-411`: `SLAPillarConnectionMode` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1674-1696`, `PrintConfig.cpp:7537-7611`: SLA support enable, support-head size, pillar size, small-pillar percentage, maximum bridges, and pillar connection mode option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max/enum label metadata beyond the current registry metadata boundary.
- SLA support creation, support-head geometry, pillar geometry, bridge planning, support placement, and runtime support generation behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7613+`: `support_buildplate_only`, support base settings, pad settings, and later SLA settings.
- `material_print_speed` from `PrintConfig.cpp:7855-7864`, which is outside this source slice.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add sorted `support_head_front_diameter`, `support_head_penetration`, `support_head_width`, `support_max_bridges_on_pillar`, `support_pillar_connection_mode`, `support_pillar_diameter`, `support_small_pillar_diameter_percent`, and `supports_enable` to the existing support shard.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_support_head_pillar.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_support_head_pillar.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 8.
- `docs/roadmap.md` and `docs/milestones/m162-print-config-sla-support-head-pillar-registry.md`: milestone sequencing docs.

## Included option definitions

- `supports_enable` (`coBool`, default `true`, field at `PrintConfig.hpp:1674`, definition lines 7537-7542, Ares kind `Bool`)
- `support_head_front_diameter` (`coFloat`, default `0.4`, field at `PrintConfig.hpp:1677`, definition lines 7544-7551, Ares kind `Float`)
- `support_head_penetration` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:1680`, definition lines 7553-7560, Ares kind `Float`)
- `support_head_width` (`coFloat`, default `1.0`, field at `PrintConfig.hpp:1683`, definition lines 7562-7570, Ares kind `Float`)
- `support_pillar_diameter` (`coFloat`, default `1.0`, field at `PrintConfig.hpp:1686`, definition lines 7572-7580, Ares kind `Float`)
- `support_small_pillar_diameter_percent` (`coPercent`, default `50`, field at `PrintConfig.hpp:1690`, definition lines 7582-7590, Ares kind `Percent`)
- `support_max_bridges_on_pillar` (`coInt`, default `3`, field at `PrintConfig.hpp:1693`, definition lines 7592-7600, Ares kind `Int`)
- `support_pillar_connection_mode` (`coEnum`, default `dynamic`, enum at `PrintConfig.hpp:265-269` and `PrintConfig.cpp:406-411`, field at `PrintConfig.hpp:1696`, definition lines 7600-7611, Ares kind `Enum`)

## Explicit non-included adjacent behavior

- `support_buildplate_only` beginning at `PrintConfig.cpp:7613` is deferred to a later source-cited SLA support milestone.
- SLA support base/pad settings following `support_buildplate_only` are deferred.
- `material_print_speed` from `PrintConfig.cpp:7855-7864` remains deferred.
- Runtime SLA support generation and pillar/head geometry behavior is deferred.

## Functional requirements

1. Add the 8 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA support-generation behavior, support-head geometry, pillar geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_buildplate_only` or later SLA support settings from `PrintConfig.cpp:7613+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA support runtime behavior and `PrintConfig.cpp:7613+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
