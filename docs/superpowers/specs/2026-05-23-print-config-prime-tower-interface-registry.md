# M141 Spec: PrintConfig prime-tower interface registry slice

## Goal
Port the adjacent wiping-volume and prime-tower interface option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1602`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6810-6815`: `wiping_volumes_extruders` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1584`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6817-6821`: `prime_tower_skip_points` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1585`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6823-6825`: `prime_tower_flat_ironing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1586`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6827-6831`: `enable_tower_interface_features` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1587`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6833-6837`: `enable_tower_interface_cooldown_during_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1583`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6839-6845`: `prime_tower_infill_gap` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode/category metadata beyond the current registry metadata boundary.
- Runtime wiping-volume matrix/vector interpretation, purge-volume computation, prime-tower skip-point behavior, prime-tower flat-ironing behavior, tower-interface feature handling, interface cooldown temperature behavior, prime-tower infill-gap application, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6847+`: `flush_into_infill`, `flush_into_support`, `flush_into_objects`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add the two `enable_tower_interface_*` definitions in lexicographic order between `enable_support` and `enable_wrapping_detection`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add the three `prime_tower_*` definitions in lexicographic order around the existing `prime_tower_brim_width`, `prime_tower_enable_framework`, and `prime_tower_width` entries.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_wipe.rs`: add `wiping_volumes_extruders` after `wipe_tower_y` and before `wrapping_detection_gcode`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add the two `enable_tower_interface_*` expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the three `prime_tower_*` keys and `wiping_volumes_extruders` in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/prime_tower_interface.rs`: add metadata assertions for all six definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_prime_tower_interface.rs`: add public lookup assertions for all six definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all six covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by six.
- `docs/roadmap.md` and `docs/milestones/m141-print-config-prime-tower-interface-registry.md`: milestone sequencing docs.

## Included option definitions

- `wiping_volumes_extruders` (`coFloats`, default `70,70,70,70,70,70,70,70,70,70`, field at `PrintConfig.hpp:1602`, definition lines 6810-6815, Ares kind `Floats`)
- `prime_tower_skip_points` (`coBool`, default `true`, field at `PrintConfig.hpp:1584`, definition lines 6817-6821, Ares kind `Bool`)
- `prime_tower_flat_ironing` (`coBool`, default `false`, field at `PrintConfig.hpp:1585`, definition lines 6823-6825, Ares kind `Bool`)
- `enable_tower_interface_features` (`coBool`, default `false`, field at `PrintConfig.hpp:1586`, definition lines 6827-6831, Ares kind `Bool`)
- `enable_tower_interface_cooldown_during_tower` (`coBool`, default `false`, field at `PrintConfig.hpp:1587`, definition lines 6833-6837, Ares kind `Bool`)
- `prime_tower_infill_gap` (`coPercent`, default `150`, field at `PrintConfig.hpp:1583`, definition lines 6839-6845, Ares kind `Percent`)

## Functional requirements

1. Add the six missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wiping-volume behavior, prime-tower skip behavior, flat-ironing behavior, interface-feature behavior, cooldown behavior, infill-gap behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `flush_into_infill`, `flush_into_support`, `flush_into_objects`, or following options from `PrintConfig.cpp:6847+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove all six covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all six covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6847+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
