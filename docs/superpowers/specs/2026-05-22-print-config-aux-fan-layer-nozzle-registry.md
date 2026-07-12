# M97 Spec: PrintConfig auxiliary fan, min layer, and nozzle registry slice

## Goal
Port the adjacent fan-minimum, auxiliary fan, minimum layer height citation, slow-down minimum speed, and nozzle diameter citation option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1537`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4651-4658`: `fan_min_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1475`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4660-4668`: `additional_cooling_fan_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1476`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4670-4677`: `close_additional_fan_first_x_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1477`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4679-4686`: `additional_fan_full_speed_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1478`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4688-4695`: `first_x_layer_fan_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1538`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4697-4704`: `min_layer_height` citation refresh; the option already exists in `ares-core` and keeps current kind/default/typed behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1542`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4706-4713`: `slow_down_min_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1543`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4715-4721`: `nozzle_diameter` citation refresh; the option already exists in `ares-core` and keeps current kind/default/typed behavior.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/mode metadata beyond the current registry boundary.
- Part-cooling behavior, auxiliary fan M106 P2 command emission, first-layer auxiliary fan behavior, slow-down speed planning, adaptive layer-height limit behavior, nozzle-specific geometry/extrusion behavior, and G-code generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723+`: `notes`, `host_type`, and following printer-host options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `additional_cooling_fan_speed`, `additional_fan_full_speed_layer`, and `close_additional_fan_first_x_layers`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `fan_min_speed`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definition for `first_x_layer_fan_speed`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `slow_down_min_speed`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: refresh `min_layer_height` and `nozzle_diameter` source citations.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: split the near-limit fixture test into a small module wrapper before adding values.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: moved and extended known-count fixture.
- Registry key, metadata, and public lookup tests cover all eight definitions/citation refreshes.
- `docs/roadmap.md` and `docs/milestones/m97-print-config-aux-fan-layer-nozzle-registry.md`: milestone sequencing docs.

## Included option definitions

- `fan_min_speed` (`coFloats`, default `20`, field at `PrintConfig.hpp:1537`, definition lines 4651-4658, Ares kind `Floats`)
- `additional_cooling_fan_speed` (`coInts`, default `0`, field at `PrintConfig.hpp:1475`, definition lines 4660-4668, Ares kind `Ints`)
- `close_additional_fan_first_x_layers` (`coInts`, default `1`, field at `PrintConfig.hpp:1476`, definition lines 4670-4677, Ares kind `Ints`)
- `additional_fan_full_speed_layer` (`coInts`, default `0`, field at `PrintConfig.hpp:1477`, definition lines 4679-4686, Ares kind `Ints`)
- `first_x_layer_fan_speed` (`coFloats`, default `0`, field at `PrintConfig.hpp:1478`, definition lines 4688-4695, Ares kind `Floats`)
- `min_layer_height` (`coFloats`, default `0.07`, field at `PrintConfig.hpp:1538`, definition lines 4697-4704, already present; refresh source citation only)
- `slow_down_min_speed` (`coFloats`, default `10`, field at `PrintConfig.hpp:1542`, definition lines 4706-4713, Ares kind `Floats`)
- `nozzle_diameter` (`coFloats`, default `0.4`, field at `PrintConfig.hpp:1543`, definition lines 4715-4721, already present; refresh source citation only)

## Functional requirements

1. Split `registry_helpers.rs` before adding more fixture entries.
2. Add the six missing options to sorted definition shards using existing value kinds only.
3. Refresh existing `min_layer_height` and `nozzle_diameter` source metadata with hpp field lines while preserving kind/default and current typed behavior.
4. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
5. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, cooling behavior, auxiliary fan G-code, speed planning, adaptive layer-height behavior, nozzle behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter following `notes`, `host_type`, or printer-host options from `PrintConfig.cpp:4723+`.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the six new keys and refreshed `min_layer_height`/`nozzle_diameter` have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eight covered definitions.
- Plan/spec explicitly account for deferred UI metadata, runtime cooling/aux-fan/speed/nozzle behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4723+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
