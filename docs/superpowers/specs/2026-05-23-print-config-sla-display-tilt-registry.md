# M154 Spec: PrintConfig SLA display and tilt registry slice

## Goal
Port the first SLA printer settings from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:260-263`, `PrintConfig.cpp:400-404`: `SLADisplayOrientation` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1830-1836`, `PrintConfig.cpp:7235-7284`: SLA display size, pixel, mirror, and orientation option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1845-1847`, `PrintConfig.cpp:7286-7310`: SLA tilt timing and area-fill option definitions.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/min/max/enum metadata beyond the current registry metadata boundary.
- SLA display orientation, mirroring, pixel grid, tilt timing, and area-fill runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7312+`: `relative_correction` and later SLA settings.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `area_fill` after `alternate_extra_wall` and before `auxiliary_fan`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definitions for `display_*` keys after `disable_m73` and before `dont_filter_internal_bridges`, and `fast_tilt_time` after `fan_speedup_time` before the filament shard.
- `crates/ares-core/src/options/registry/definitions/table/tail_raft.rs`: add `slow_tilt_time` after `slow_down_min_speed` and before `slowdown_for_curled_perimeters`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add covered keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_display_tilt.rs`: add metadata assertions for all covered definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_display_tilt.rs`: add public lookup assertions for all covered definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs` and a new SLA values shard: add fixtures and move existing `start_end_points`, `template_custom_gcode`, `time_lapse_gcode`, and `wrapping_detection_gcode` fixtures into the shard so `values.rs` remains below 400 LOC.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 10.
- `docs/roadmap.md` and `docs/milestones/m154-print-config-sla-display-tilt-registry.md`: milestone sequencing docs.

## Included option definitions

- `display_width` (`coFloat`, default `120.`, field at `PrintConfig.hpp:1830`, definition lines 7235-7239, Ares kind `Float`)
- `display_height` (`coFloat`, default `68.`, field at `PrintConfig.hpp:1831`, definition lines 7241-7245, Ares kind `Float`)
- `display_pixels_x` (`coInt`, default `2560`, field at `PrintConfig.hpp:1832`, definition lines 7247-7252, Ares kind `Int`)
- `display_pixels_y` (`coInt`, default `1440`, field at `PrintConfig.hpp:1833`, definition lines 7254-7259, Ares kind `Int`)
- `display_mirror_x` (`coBool`, default `true`, field at `PrintConfig.hpp:1835`, definition lines 7261-7266, Ares kind `Bool`)
- `display_mirror_y` (`coBool`, default `false`, field at `PrintConfig.hpp:1836`, definition lines 7268-7273, Ares kind `Bool`)
- `display_orientation` (`coEnum`, default `portrait`, enum at `PrintConfig.hpp:260-263`, field at `PrintConfig.hpp:1834`, definition lines 7275-7284, Ares kind `Enum`)
- `fast_tilt_time` (`coFloat`, default `5.`, field at `PrintConfig.hpp:1845`, definition lines 7286-7293, Ares kind `Float`)
- `slow_tilt_time` (`coFloat`, default `8.`, field at `PrintConfig.hpp:1846`, definition lines 7295-7302, Ares kind `Float`)
- `area_fill` (`coFloat`, default `50.`, field at `PrintConfig.hpp:1847`, definition lines 7304-7310, Ares kind `Float`)

## Functional requirements

1. Add the 10 missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA display behavior, tilt timing behavior, area-fill behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `relative_correction` or later SLA settings from `PrintConfig.cpp:7312+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split known-count fixtures as needed.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred SLA runtime behavior and `PrintConfig.cpp:7312+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
