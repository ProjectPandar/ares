# M166 Spec: PrintConfig SLA zero-elevation pad registry slice

## Goal
Port the SLA zero-elevation object-pad settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1757-1780`, `PrintConfig.cpp:7768-7817`: SLA zero-elevation object-pad enable flags, object gap, connector stride, connector width, and connector penetration option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/min/max/mode metadata beyond the current registry metadata boundary.
- Zero-elevation pad mode, object-pad connector geometry, SLA pad generation, pad/base geometry, and runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7819+`: hollowing settings, `material_print_speed`, and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_after_material.rs`: keep the existing `pad_*` base definitions and interleave the 6 covered zero-elevation pad definitions so the full pad block remains lexicographically sorted as: `pad_around_object`, `pad_around_object_everywhere`, `pad_brim_size`, `pad_enable`, `pad_max_merge_distance`, `pad_object_connector_penetration`, `pad_object_connector_stride`, `pad_object_connector_width`, `pad_object_gap`, `pad_wall_height`, `pad_wall_slope`, `pad_wall_thickness`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_after_pad.rs`: create a new shard containing `parking_pos_retraction` and every following definition currently in `late_tail_after_material.rs`, preserving their current order.
- `crates/ares-core/src/options/registry/definitions/table.rs`: register and merge the new `late_tail_after_pad` shard immediately after `late_tail_after_material` and before `late_tail_final`.
- `crates/ares-core/src/options/registry/tests/keys/third.rs`: interleave the 6 covered `pad_*` keys so the full pad block remains lexicographically sorted as: `pad_around_object`, `pad_around_object_everywhere`, `pad_brim_size`, `pad_enable`, `pad_max_merge_distance`, `pad_object_connector_penetration`, `pad_object_connector_stride`, `pad_object_connector_width`, `pad_object_gap`, `pad_wall_height`, `pad_wall_slope`, `pad_wall_thickness`, then `parking_pos_retraction`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_pad_zero_elevation.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_pad_zero_elevation.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 6.
- `docs/roadmap.md` and `docs/milestones/m166-print-config-sla-pad-zero-elevation-registry.md`: milestone sequencing docs.

## Included option definitions

- `pad_around_object` (`coBool`, default `false`, field at `PrintConfig.hpp:1766`, definition lines 7768-7773, Ares kind `Bool`)
- `pad_around_object_everywhere` (`coBool`, default `false`, field at `PrintConfig.hpp:1768`, definition lines 7775-7780, Ares kind `Bool`)
- `pad_object_gap` (`coFloat`, default `1`, field at `PrintConfig.hpp:1771`, definition lines 7782-7790, Ares kind `Float`)
- `pad_object_connector_stride` (`coFloat`, default `10`, field at `PrintConfig.hpp:1774`, definition lines 7792-7799, Ares kind `Float`)
- `pad_object_connector_width` (`coFloat`, default `0.5`, field at `PrintConfig.hpp:1777`, definition lines 7801-7808, Ares kind `Float`)
- `pad_object_connector_penetration` (`coFloat`, default `0.3`, field at `PrintConfig.hpp:1780`, definition lines 7810-7817, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `hollowing_enable` beginning at `PrintConfig.cpp:7819` is deferred to a later source-cited SLA hollowing milestone.
- Hollowing thickness/quality/closing-distance, `material_print_speed`, and later SLA settings following `hollowing_enable` are deferred.
- Runtime zero-elevation pad mode, object-pad connector geometry, and SLA pad generation behavior is deferred.

## Functional requirements

1. Add the 6 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, zero-elevation pad behavior, object-pad connector geometry, SLA pad-generation behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `hollowing_enable` or later SLA settings from `PrintConfig.cpp:7819+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; mechanically split `late_tail_after_material.rs` as described because it is already near the limit.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Definition shard split preserves exact option order and keeps changed Rust files below 400 LOC.
- Plan/spec explicitly account for deferred zero-elevation pad runtime behavior and `PrintConfig.cpp:7819+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
