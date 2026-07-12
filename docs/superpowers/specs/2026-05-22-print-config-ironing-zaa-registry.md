# M91 Spec: PrintConfig ironing and Z contouring registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` ironing and Z-layer anti-aliasing / Z contouring option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:100-106`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:257-263`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1138`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4161-4176`: `ironing_type` enum values and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-98`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1139`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-255`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4178-4188`: `ironing_pattern` enum values and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1140`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4190-4200`: `ironing_flow` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1141`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4202-4210`: `ironing_spacing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1142`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4212-4220`: `ironing_inset` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1144`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4222-4229`: `ironing_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1145`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4231-4239`: `ironing_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1146`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4241-4246`: `ironing_angle_fixed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4248-4256`: `ironing_expansion` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1237`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4258-4263`: `zaa_enabled` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1240`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4265-4275`: `zaa_minimize_perimeter_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1238`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4277-4282`: `zaa_dont_alternate_fill_direction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1239`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4284-4293`: `zaa_min_z` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/mode/enum-label metadata beyond the current registry boundary.
- Ironing surface selection, pattern generation, flow/spacing/inset/speed/angle/expansion behavior.
- Z contouring / Z-layer anti-aliasing behavior, slicing plane changes, and fill-direction alternation behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4295+`: `layer_change_gcode` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle_tail.rs`: add sorted definitions for `ironing_angle`, `ironing_angle_fixed`, `ironing_expansion`, `ironing_flow`, `ironing_inset`, `ironing_pattern`, `ironing_spacing`, `ironing_speed`, and `ironing_type` around the existing `ironing_fan_speed` definition.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `zaa_dont_alternate_fill_direction`, `zaa_enabled`, `zaa_min_z`, and `zaa_minimize_perimeter_height` after the existing `wrapping_*` definitions.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod ironing;`.
- `crates/ares-core/src/options/registry/tests/metadata/ironing.rs`: source metadata assertions for all thirteen options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_ironing;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_ironing.rs`: public lookup coverage for all thirteen options.
- `docs/roadmap.md` and `docs/milestones/m91-print-config-ironing-zaa-registry.md`: milestone sequencing docs.

## Included option definitions

- `ironing_type` (`coEnum`, default `no ironing`, field at `PrintConfig.hpp:1138`, enum at `PrintConfig.hpp:100-106`, enum map at `PrintConfig.cpp:257-263`, definition lines 4161-4176, Ares kind `Enum`)
- `ironing_pattern` (`coEnum`, default `rectilinear`, field at `PrintConfig.hpp:1139`, enum at `PrintConfig.hpp:87-98`, enum map at `PrintConfig.cpp:225-255`, definition lines 4178-4188, Ares kind `Enum`)
- `ironing_flow` (`coPercent`, default `10`, field at `PrintConfig.hpp:1140`, definition lines 4190-4200, Ares kind `Percent`)
- `ironing_spacing` (`coFloat`, default `0.1`, field at `PrintConfig.hpp:1141`, definition lines 4202-4210, Ares kind `Float`)
- `ironing_inset` (`coFloat`, default `0`, field at `PrintConfig.hpp:1142`, definition lines 4212-4220, Ares kind `Float`)
- `ironing_speed` (`coFloat`, default `20`, field at `PrintConfig.hpp:1144`, definition lines 4222-4229, Ares kind `Float`)
- `ironing_angle` (`coFloat`, default `0`, field at `PrintConfig.hpp:1145`, definition lines 4231-4239, Ares kind `Float`)
- `ironing_angle_fixed` (`coBool`, default `false`, field at `PrintConfig.hpp:1146`, definition lines 4241-4246, Ares kind `Bool`)
- `ironing_expansion` (`coFloat`, default `0`, definition lines 4248-4256, Ares kind `Float`)
- `zaa_enabled` (`coBool`, default `false`, field at `PrintConfig.hpp:1237`, definition lines 4258-4263, Ares kind `Bool`)
- `zaa_minimize_perimeter_height` (`coFloat`, default `35`, field at `PrintConfig.hpp:1240`, definition lines 4265-4275, Ares kind `Float`)
- `zaa_dont_alternate_fill_direction` (`coBool`, default `false`, field at `PrintConfig.hpp:1238`, definition lines 4277-4282, Ares kind `Bool`)
- `zaa_min_z` (`coFloat`, default `0.05`, field at `PrintConfig.hpp:1239`, definition lines 4284-4293, Ares kind `Float`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, ironing behavior, Z contouring behavior, slicing-plane behavior, fill-direction behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `layer_change_gcode` or following options from `PrintConfig.cpp:4295+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; if a shard crosses the limit, split behavior-preservingly and document/review the amendment before commit.

## Acceptance checks

- Registry tests prove all thirteen new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all thirteen new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `layer_change_gcode` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
