# M114 Spec: PrintConfig wipe-tower and ramming registry slice

## Goal
Port the adjacent wipe-tower type, purge/ramming, tool-change-on-wipe-tower, and sparse-layer wipe-tower option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:74-77`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:212-216`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1457`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5821-5830`: `wipe_tower_type` enum and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1458`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5832-5836`: `purge_in_prime_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1459`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5838-5842`: `enable_filament_ramming` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1460`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5844-5852`: `tool_change_on_wipe_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1391`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5855-5861`: `wipe_tower_no_sparse_layers` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/modes beyond the current registry metadata boundary.
- Wipe-tower implementation selection, prime-tower purge behavior, filament ramming behavior, tool-change travel behavior, sparse-layer wipe-tower suppression, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5863+`: `single_extruder_multi_material_priming`, slice closing/slicing mode, support, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `enable_filament_ramming` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `purge_in_prime_tower` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: add `tool_change_on_wipe_tower`, `wipe_tower_no_sparse_layers`, and `wipe_tower_type` in sorted order.
- If needed to keep `tail.rs` below 400 LOC, mechanically split the later `tail.rs` entries into a new sorted shard merged immediately after `tail.rs` and before `tail_final.rs`.
- Registry key, metadata, fixture-count, and public lookup tests cover all five definitions.
- `docs/roadmap.md` and `docs/milestones/m114-print-config-wipe-tower-ramming-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_type` (`coEnum`, `WipeTowerType`, default `type2`, enum keys `type1`/`type2`, field at `PrintConfig.hpp:1457`, enum lines `PrintConfig.hpp:74-77` and `PrintConfig.cpp:212-216`, definition lines 5821-5830, Ares kind `Enum`)
- `purge_in_prime_tower` (`coBool`, default `true`, field at `PrintConfig.hpp:1458`, definition lines 5832-5836, Ares kind `Bool`)
- `enable_filament_ramming` (`coBool`, default `true`, field at `PrintConfig.hpp:1459`, definition lines 5838-5842, Ares kind `Bool`)
- `tool_change_on_wipe_tower` (`coBool`, default `false`, field at `PrintConfig.hpp:1460`, definition lines 5844-5852, Ares kind `Bool`)
- `wipe_tower_no_sparse_layers` (`coBool`, default `false`, field at `PrintConfig.hpp:1391`, definition lines 5855-5861, Ares kind `Bool`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe-tower behavior, ramming behavior, tool-change travel behavior, sparse-layer wipe-tower behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `single_extruder_multi_material_priming` or following options from `PrintConfig.cpp:5863+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5863+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
