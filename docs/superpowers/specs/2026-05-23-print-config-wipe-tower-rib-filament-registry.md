# M140 Spec: PrintConfig wipe-tower rib and filament registry slice

## Goal
Port the adjacent wipe-tower rib, fillet, and perimeter-filament option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1598`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6775-6782`: `wipe_tower_extra_rib_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1599`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6784-6791`: `wipe_tower_rib_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1600`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6793-6797`: `wipe_tower_fillet_wall` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1601`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6800-6808`: `wipe_tower_filament` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/max/mode/gui_type/category metadata beyond the current registry metadata boundary.
- Runtime rib sizing, rib-width constraints, fillet wall geometry, wipe-tower perimeter filament selection, non-soluble preference, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6810+`: `wiping_volumes_extruders`, `prime_tower_skip_points`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: mechanically move the existing wipe-tower/wrapping suffix definitions out so this file remains below 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs`: create a new sorted shard containing the existing moved `wipe_tower_*` and `wrapping_*` definitions plus the four covered M140 definitions in lexicographic order.
- `crates/ares-core/src/options/registry/definitions/table.rs`: register and merge the new shard immediately after `tail_terminal_suffix` and before `tail_z`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_rib_filament.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_rib_filament.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all four covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by four.
- `docs/roadmap.md` and `docs/milestones/m140-print-config-wipe-tower-rib-filament-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_extra_rib_length` (`coFloat`, default `0`, field at `PrintConfig.hpp:1598`, definition lines 6775-6782, Ares kind `Float`)
- `wipe_tower_rib_width` (`coFloat`, default `8`, field at `PrintConfig.hpp:1599`, definition lines 6784-6791, Ares kind `Float`)
- `wipe_tower_fillet_wall` (`coBool`, default `true`, field at `PrintConfig.hpp:1600`, definition lines 6793-6797, Ares kind `Bool`)
- `wipe_tower_filament` (`coInt`, default `0`, field at `PrintConfig.hpp:1601`, definition lines 6800-6808, Ares kind `Int`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, rib behavior, fillet behavior, filament-selection behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wiping_volumes_extruders`, `prime_tower_skip_points`, or following options from `PrintConfig.cpp:6810+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split only the registry table shard needed to satisfy this limit.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- The mechanical registry shard split moves definitions without changing their metadata.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6810+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
