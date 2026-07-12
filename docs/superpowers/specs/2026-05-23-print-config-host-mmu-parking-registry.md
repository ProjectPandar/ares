# M98 Spec: PrintConfig notes, host, nozzle-volume, and MMU parking registry slice

## Goal
Port the adjacent notes, printer-host type, nozzle-volume, cooling-tube, high-current filament swap, and parking-position option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1633`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723-4731`: `notes` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:79-81`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:137-153`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4733-4768`: `host_type` enum and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1613`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4770-4777`: `nozzle_volume` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1428`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4779-4785`: `cooling_tube_retraction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1429`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4787-4793`: `cooling_tube_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1430`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4795-4801`: `high_current_on_filament_swap` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1431`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4803-4810`: `parking_pos_retraction` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/multiline/full-width/height/readonly/nullable/nocli metadata beyond the current registry boundary.
- Printer-host upload behavior and host-specific integrations.
- MMU cooling-tube move behavior, high-current filament-swap behavior, parking-position runtime use, and nozzle-volume runtime use.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4812+`: `extra_loading_move`, `start_end_points`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted definitions for `cooling_tube_length` and `cooling_tube_retraction`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `high_current_on_filament_swap` and `host_type`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted definitions for `notes`, `nozzle_volume`, and `parking_pos_retraction`.
- Registry key, metadata, fixture-count, and public lookup tests cover all seven definitions.
- `docs/roadmap.md` and `docs/milestones/m98-print-config-host-mmu-parking-registry.md`: milestone sequencing docs.

## Included option definitions

- `notes` (`coString`, default `""`, field at `PrintConfig.hpp:1633`, definition lines 4723-4731, Ares kind `String`)
- `host_type` (`coEnum`, default `octoprint`, enum declaration at `PrintConfig.hpp:79-81`, enum map at `PrintConfig.cpp:137-153`, definition lines 4733-4768, Ares kind `Enum`)
- `nozzle_volume` (`coFloats` with nullable default `0.0`, field at `PrintConfig.hpp:1613`, definition lines 4770-4777, Ares kind `FloatsNullable`)
- `cooling_tube_retraction` (`coFloat`, default `91.5`, field at `PrintConfig.hpp:1428`, definition lines 4779-4785, Ares kind `Float`)
- `cooling_tube_length` (`coFloat`, default `5`, field at `PrintConfig.hpp:1429`, definition lines 4787-4793, Ares kind `Float`)
- `high_current_on_filament_swap` (`coBool`, default `false`, field at `PrintConfig.hpp:1430`, definition lines 4795-4801, Ares kind `Bool`)
- `parking_pos_retraction` (`coFloat`, default `92`, field at `PrintConfig.hpp:1431`, definition lines 4803-4810, Ares kind `Float`)

## Functional requirements

1. Add the seven missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, printer-host upload behavior, MMU load/unload behavior, cooling-tube movement behavior, high-current swap behavior, parking behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following `extra_loading_move`, `start_end_points`, or later options from `PrintConfig.cpp:4812+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the seven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all seven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, printer-host/MMU/runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4812+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
