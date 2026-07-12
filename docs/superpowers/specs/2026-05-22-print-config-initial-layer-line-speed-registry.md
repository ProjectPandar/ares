# M78 Spec: PrintConfig initial-layer line, speed, and slow-down registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` initial-layer line width, print height, speed, travel speed, and slow-down option-definition slice into `ares-core` option registry metadata by adding registry coverage for `initial_layer_line_width`, `initial_layer_print_height`, `initial_layer_speed`, `initial_layer_infill_speed`, `initial_layer_travel_speed`, and `slow_down_layers`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1527`: `initial_layer_line_width` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251-3261`: `initial_layer_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1528`: `initial_layer_print_height` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3264-3270`: `initial_layer_print_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1529`: `initial_layer_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3280-3286`: `initial_layer_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1532`: `initial_layer_infill_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3288-3294`: `initial_layer_infill_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1421`: `initial_layer_travel_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3296-3304`: `initial_layer_travel_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1627`: `slow_down_layers` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3306-3314`: `slow_down_layers` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/max_literal/mode/ratio metadata beyond the current registry boundary.
- Initial-layer line-width resolution, print-height behavior, speed behavior, travel-speed ratio behavior, and slow-down runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316+`: `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, support/internal bridge fan speed, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for the five `initial_layer_*` options.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `slow_down_layers`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: extend source metadata assertions for the six options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_speed.rs`: extend public lookup coverage for the six options.
- `docs/roadmap.md` and `docs/milestones/m78-print-config-initial-layer-line-speed-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `initial_layer_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1527`, definition lines 3251-3261, Ares kind `FloatOrPercent`)
- `initial_layer_print_height` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:1528`, definition lines 3264-3270, Ares kind `Float`)
- `initial_layer_speed` (`coFloat`, default `30`, field at `PrintConfig.hpp:1529`, definition lines 3280-3286, Ares kind `Float`)
- `initial_layer_infill_speed` (`coFloat`, default `60`, field at `PrintConfig.hpp:1532`, definition lines 3288-3294, Ares kind `Float`)
- `initial_layer_travel_speed` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1421`, definition lines 3296-3304, Ares kind `FloatOrPercent`)
- `slow_down_layers` (`coInt`, default `0`, field at `PrintConfig.hpp:1627`, definition lines 3306-3314, Ares kind `Int`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Float`, `FloatOrPercent`, and `Int`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, initial-layer line-width behavior, print-height behavior, speed behavior, ratio behavior, slow-down behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, support/internal bridge fan speed, or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, ratio, and GUI metadata from `PrintConfig.cpp:3251-3314` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Initial-layer line-width resolution, print-height behavior, speed behavior, travel-speed ratio behavior, slow-down behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, support/internal bridge fan speed, and following options from `PrintConfig.cpp:3316+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all six new keys.
- Plan/spec explicitly account for deferred UI/bounds/ratio metadata, initial-layer behavior, slow-down behavior, slicing/extrusion/G-code behavior, and following nozzle/fan scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
