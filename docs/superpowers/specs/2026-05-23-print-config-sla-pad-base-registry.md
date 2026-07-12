# M165 Spec: PrintConfig SLA pad base registry slice

## Goal
Port the first SLA pad/base-pool settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1733-1755`, `PrintConfig.cpp:7712-7766`: SLA pad enable, pad wall thickness/height, brim size, maximum merge distance, and wall slope option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/min/max/mode metadata beyond the current registry metadata boundary.
- SLA pad generation, pad/base geometry, base-pool merging, wall slope behavior, zero-elevation pad mode, and runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7768+`: `pad_around_object`, zero-elevation object-pad connector settings, hollowing settings, `material_print_speed`, and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_after_material.rs`: add sorted `pad_brim_size`, `pad_enable`, `pad_max_merge_distance`, `pad_wall_height`, `pad_wall_slope`, and `pad_wall_thickness` after `overhang_reverse_threshold` and before `parking_pos_retraction`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs` and `crates/ares-core/src/options/registry/tests/keys/third.rs`: mechanically move `parking_pos_retraction` and all following keys from `second.rs` to the beginning of `third.rs`; add the 6 covered `pad_*` keys in sorted order in `third.rs` before `parking_pos_retraction`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_pad_base.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_pad_base.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add fixture values for the covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 6.
- `docs/roadmap.md` and `docs/milestones/m165-print-config-sla-pad-base-registry.md`: milestone sequencing docs.

## Included option definitions

- `pad_enable` (`coBool`, default `true`, field at `PrintConfig.hpp:1736`, definition lines 7712-7717, Ares kind `Bool`)
- `pad_wall_thickness` (`coFloat`, default `2`, field at `PrintConfig.hpp:1739`, definition lines 7719-7727, Ares kind `Float`)
- `pad_wall_height` (`coFloat`, default `0`, field at `PrintConfig.hpp:1742`, definition lines 7729-7737, Ares kind `Float`)
- `pad_brim_size` (`coFloat`, default `1.6`, field at `PrintConfig.hpp:1745`, definition lines 7739-7747, Ares kind `Float`)
- `pad_max_merge_distance` (`coFloat`, default `50`, field at `PrintConfig.hpp:1749`, definition lines 7749-7756, Ares kind `Float`)
- `pad_wall_slope` (`coFloat`, default `90`, field at `PrintConfig.hpp:1755`, definition lines 7758-7766, Ares kind `Float`)

## Explicit non-included adjacent behavior

- `pad_around_object` beginning at `PrintConfig.cpp:7768` is deferred to a later source-cited SLA zero-elevation pad milestone.
- `pad_around_object_everywhere`, `pad_object_gap`, object connector settings, hollowing settings, `material_print_speed`, and later SLA settings following `pad_around_object` are deferred.
- Runtime SLA pad generation, pad/base geometry, base-pool merging, wall slope behavior, and zero-elevation pad behavior is deferred.

## Functional requirements

1. Add the 6 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA pad-generation behavior, pad/base geometry, zero-elevation pad behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `pad_around_object` or later SLA settings from `PrintConfig.cpp:7768+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; mechanically split `keys/second.rs` into `keys/third.rs` as described because `keys/second.rs` is already near the limit.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Key-list shard split preserves the exact expected key order and keeps changed Rust files below 400 LOC.
- Plan/spec explicitly account for deferred SLA pad runtime behavior and `PrintConfig.cpp:7768+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
