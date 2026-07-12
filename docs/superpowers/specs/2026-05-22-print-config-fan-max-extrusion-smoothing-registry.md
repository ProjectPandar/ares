# M96 Spec: PrintConfig fan max and extrusion-rate smoothing registry slice

## Goal
Port the adjacent fan maximum, maximum layer height citation, and extrusion-rate smoothing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1535`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4591-4599`: `fan_max_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1536`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4601-4608`: `max_layer_height` option-definition citation refresh; the option already exists in `ares-core` and keeps its current kind/default/typed behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1362`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4610-4629`: `max_volumetric_extrusion_rate_slope` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1363`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4631-4641`: `max_volumetric_extrusion_rate_slope_segment_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1364`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4643-4648`: `extrusion_rate_smoothing_external_perimeter_only` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/mode metadata beyond the current registry boundary.
- Part-cooling runtime behavior for `fan_max_speed`.
- Extrusion-rate smoothing behavior, speed planning, arc-fitting interactions, and G-code generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4651+`: fan minimum, additional cooling fan, first-layer auxiliary fan, min layer height, nozzle, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definitions for `extrusion_rate_smoothing_external_perimeter_only` and `fan_max_speed`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: refresh `max_layer_height` source and add sorted definitions for the two `max_volumetric_extrusion_rate_slope*` keys.
- `crates/ares-core/src/options/registry/tests/keys/first.rs` and `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the four new expected registry keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod extrusion_smoothing;`.
- `crates/ares-core/src/options/registry/tests/metadata/extrusion_smoothing.rs`: source metadata assertions for all five covered definitions, including `max_layer_height` citation refresh.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_extrusion_smoothing;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_extrusion_smoothing.rs`: public lookup coverage for all five covered definitions.
- `docs/roadmap.md` and `docs/milestones/m96-print-config-fan-max-extrusion-smoothing-registry.md`: milestone sequencing docs.

## Included option definitions

- `fan_max_speed` (`coFloats`, default `100`, field at `PrintConfig.hpp:1535`, definition lines 4591-4599, Ares kind `Floats`)
- `max_layer_height` (`coFloats`, default `0`, field at `PrintConfig.hpp:1536`, definition lines 4601-4608, already present; refresh source citation only)
- `max_volumetric_extrusion_rate_slope` (`coFloat`, default `0`, field at `PrintConfig.hpp:1362`, definition lines 4610-4629, Ares kind `Float`)
- `max_volumetric_extrusion_rate_slope_segment_length` (`coFloat`, default `3.0`, field at `PrintConfig.hpp:1363`, definition lines 4631-4641, Ares kind `Float`)
- `extrusion_rate_smoothing_external_perimeter_only` (`coBool`, default `false`, field at `PrintConfig.hpp:1364`, definition lines 4643-4648, Ares kind `Bool`)

## Functional requirements

1. Add the four missing options to sorted definition shards using existing value kinds only.
2. Refresh existing `max_layer_height` source metadata with the hpp field line while preserving its kind/default and current typed behavior.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, cooling behavior, extrusion-rate smoothing behavior, arc-fitting behavior, speed planning, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter following fan-min/additional-cooling/min-layer/nozzle options from `PrintConfig.cpp:4651+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four new keys and refreshed `max_layer_height` have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, runtime cooling/smoothing behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4651+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
