# M82 Spec: PrintConfig process and G-code utility registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` process/G-code utility option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filter_out_gap_fill`, `gap_infill_speed`, `precise_z_height`, `enable_arc_fitting`, `gcode_add_line_number`, `scan_first_layer`, and `enable_power_loss_recovery`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1190`, `PrintConfig.cpp:3578-3585`: `filter_out_gap_fill` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1120`, `PrintConfig.cpp:3587-3594`: `gap_infill_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1059`, `PrintConfig.cpp:3597-3604`: `precise_z_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1298`, `PrintConfig.cpp:3607-3616`: `enable_arc_fitting` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1353`, `PrintConfig.cpp:3618-3622`: `gcode_add_line_number` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1346`, `PrintConfig.cpp:3625-3629`: `scan_first_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:125-129`, `PrintConfig.cpp:185-190`, `PrintConfig.hpp:1347`, `PrintConfig.cpp:3632-3643`: `enable_power_loss_recovery` enum metadata and option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/mode/enum label metadata beyond the current registry boundary.
- Gap-fill filtering, gap speed application, precise-Z layer-height adjustment, arc fitting, G-code line numbering, first-layer camera scan integration, and power-loss recovery G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3652+`: `nozzle_type` and following options.
- Slicing, extrusion, G-code output behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definitions for `enable_arc_fitting` and `enable_power_loss_recovery`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `filter_out_gap_fill`, `gap_infill_speed`, and `gcode_add_line_number`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `precise_z_height`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `scan_first_layer`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod process;`.
- `crates/ares-core/src/options/registry/tests/metadata/process.rs`: source metadata assertions for all seven options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_process;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_process.rs`: public lookup coverage for all seven options.
- `docs/roadmap.md` and `docs/milestones/m82-print-config-process-gcode-utility-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filter_out_gap_fill` (`coFloat`, default `0`, field at `PrintConfig.hpp:1190`, definition lines 3578-3585, Ares kind `Float`)
- `gap_infill_speed` (`coFloat`, default `30`, field at `PrintConfig.hpp:1120`, definition lines 3587-3594, Ares kind `Float`)
- `precise_z_height` (`coBool`, default `false`, field at `PrintConfig.hpp:1059`, definition lines 3597-3604, Ares kind `Bool`)
- `enable_arc_fitting` (`coBool`, default `false`, field at `PrintConfig.hpp:1298`, definition lines 3607-3616, Ares kind `Bool`)
- `gcode_add_line_number` (`coBool`, default `false`, field at `PrintConfig.hpp:1353`, definition lines 3618-3622, Ares kind `Bool`)
- `scan_first_layer` (`coBool`, default `false`, field at `PrintConfig.hpp:1346`, definition lines 3625-3629, Ares kind `Bool`)
- `enable_power_loss_recovery` (`coEnum`, default `printer_configuration`, field at `PrintConfig.hpp:1347`, enum at `PrintConfig.hpp:125-129`, enum map at `PrintConfig.cpp:185-190`, definition lines 3632-3643, Ares kind `Enum`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using `Float`, `Bool`, and `Enum`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, gap filtering, gap speed behavior, precise-Z behavior, arc-fitting behavior, line-number output, first-layer scan behavior, power-loss recovery G-code behavior, slicing behavior, extrusion behavior, or G-code output behavior for these options in this milestone.
6. Do not add or alter `nozzle_type` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3578-3643` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; enum key/default identity is preserved through string defaults and source citations only.
- Gap-fill filtering, gap speed application, precise-Z layer adjustment, arc fitting, line numbering, first-layer scan integration, power-loss recovery G-code, typed accessors, slicing, extrusion, and G-code output behavior are deferred to later source-cited milestones.
- `nozzle_type` and following options from `PrintConfig.cpp:3652+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all seven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all seven new keys.
- Plan/spec explicitly account for deferred UI/bounds/enum-label metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `nozzle_type` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
