# M144 Spec: PrintConfig XY compensation and polyhole registry slice

## Goal
Port the adjacent XY compensation and polyhole option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1001`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6907-6915`: `xy_hole_compensation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1002`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6917-6925`: `xy_contour_compensation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1202`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6927-6934`: `hole_to_polyhole` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1203`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6936-6947`: `hole_to_polyhole_threshold` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1204`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6949-6954`: `hole_to_polyhole_twisted` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/max/mode/gui metadata beyond the current registry metadata boundary.
- XY hole/contour expansion or contraction in `PrintObjectSlice.cpp`, polyhole detection/conversion in `PrintObject.cpp`, and any PrintObject invalidation behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6956+`: `thumbnails`, `thumbnails_format`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add `hole_to_polyhole`, `hole_to_polyhole_threshold`, and `hole_to_polyhole_twisted` after `high_current_on_filament_swap` and before `host_type`.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs`: add `xy_contour_compensation` and `xy_hole_compensation` after `wrapping_exclude_area` and before the `tail_z` shard.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add the three `hole_to_polyhole*` keys in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `xy_contour_compensation` and `xy_hole_compensation` after `wrapping_exclude_area` and before `z_hop`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/xy_polyhole.rs`: add metadata assertions for all five definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_xy_polyhole.rs`: add public lookup assertions for all five definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all five covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by five.
- `docs/roadmap.md` and `docs/milestones/m144-print-config-xy-polyhole-registry.md`: milestone sequencing docs.

## Included option definitions

- `xy_hole_compensation` (`coFloat`, default `0`, field at `PrintConfig.hpp:1001`, definition lines 6907-6915, Ares kind `Float`)
- `xy_contour_compensation` (`coFloat`, default `0`, field at `PrintConfig.hpp:1002`, definition lines 6917-6925, Ares kind `Float`)
- `hole_to_polyhole` (`coBool`, default `false`, field at `PrintConfig.hpp:1202`, definition lines 6927-6934, Ares kind `Bool`)
- `hole_to_polyhole_threshold` (`coFloatOrPercent`, default `0.01`, field at `PrintConfig.hpp:1203`, definition lines 6936-6947, Ares kind `FloatOrPercent`)
- `hole_to_polyhole_twisted` (`coBool`, default `true`, field at `PrintConfig.hpp:1204`, definition lines 6949-6954, Ares kind `Bool`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, XY compensation behavior, polyhole detection/conversion/twist behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `thumbnails`, `thumbnails_format`, or following options from `PrintConfig.cpp:6956+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove all five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6956+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
