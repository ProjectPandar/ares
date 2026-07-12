# M105 Spec: PrintConfig extruder variant and ID registry slice

## Goal
Port the adjacent extruder variant list, AMS count, printer/print/filament extruder IDs, and printer/print/filament extruder variant option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5244`: `extruder_variant_list` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1410`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5246-5250`: `extruder_ams_count` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1411`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5257`: `printer_extruder_id` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1413`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5259-5264`: `printer_extruder_variant` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1412`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5266-5270`: `master_extruder_id` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1077`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5272-5277`: `print_extruder_id` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1078`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5279-5284`: `print_extruder_variant` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1338`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5292-5297`: `filament_extruder_variant` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5299-5304`: `filament_self_index` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/cli metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5286-5290`: commented-out `filament_extruder_id` block; do not add it as an Ares registry definition.
- AMS-count parsing helpers declared at `PrintConfig.hpp:514-515`, `save_extruder_ams_count_to_string`, extruder variant normalization, printer/print/filament extruder mapping, preset compatibility behavior, and DynamicPrintConfig variant-resolution helpers.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306+`: `retract_restart_extra`, `retract_restart_extra_toolchange`, `retraction_speed`, `deretraction_speed`, firmware retraction, calibration marks, M73, seam options, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `extruder_ams_count` and `extruder_variant_list` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_filament.rs`: add `filament_extruder_variant` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: add `filament_self_index` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `master_extruder_id` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `print_extruder_id`, `print_extruder_variant`, `printer_extruder_id`, and `printer_extruder_variant` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all nine definitions.
- `docs/roadmap.md` and `docs/milestones/m105-print-config-extruder-variant-id-registry.md`: milestone sequencing docs.

## Included option definitions

- `extruder_variant_list` (`coStrings`, default `Direct Drive Standard`, definition lines 5239-5244, Ares kind `Strings`)
- `extruder_ams_count` (`coStrings`, default empty, field at `PrintConfig.hpp:1410`, definition lines 5246-5250, Ares kind `Strings`)
- `printer_extruder_id` (`coInts`, default `1`, field at `PrintConfig.hpp:1411`, definition lines 5252-5257, Ares kind `Ints`)
- `printer_extruder_variant` (`coStrings`, default `Direct Drive Standard`, field at `PrintConfig.hpp:1413`, definition lines 5259-5264, Ares kind `Strings`)
- `master_extruder_id` (`coInt`, default `1`, field at `PrintConfig.hpp:1412`, definition lines 5266-5270, Ares kind `Int`)
- `print_extruder_id` (`coInts`, default `1`, field at `PrintConfig.hpp:1077`, definition lines 5272-5277, Ares kind `Ints`)
- `print_extruder_variant` (`coStrings`, default `Direct Drive Standard`, field at `PrintConfig.hpp:1078`, definition lines 5279-5284, Ares kind `Strings`)
- `filament_extruder_variant` (`coStrings`, default `Direct Drive Standard`, field at `PrintConfig.hpp:1338`, definition lines 5292-5297, Ares kind `Strings`)
- `filament_self_index` (`coInts`, default `1`, definition lines 5299-5304, Ares kind `Ints`)

## Functional requirements

1. Add the nine missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, AMS-count parsing helpers, extruder variant normalization, printer/print/filament extruder mapping, preset compatibility behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add commented-out upstream `filament_extruder_id` or following restart/retraction speed and seam options from `PrintConfig.cpp:5306+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, AMS/extruder variant runtime behavior, slicing/extrusion/G-code behavior, commented-out `filament_extruder_id`, and following `PrintConfig.cpp:5306+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
