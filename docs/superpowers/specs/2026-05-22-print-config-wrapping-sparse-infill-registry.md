# M89 Spec: PrintConfig wrapping detection and sparse-infill utility registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` clumping/wrapping detection, sparse-infill filament, sparse-infill line width, infill/wall overlap, top/bottom infill/wall overlap, and sparse-infill speed option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1348`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3987-3991`: `enable_wrapping_detection` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1349`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3993-3998`: `wrapping_detection_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1350`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4000-4005`: `wrapping_exclude_area` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1121`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4007-4014`: `sparse_infill_filament` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1122`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4016-4026`: `sparse_infill_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1123`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4028-4039`: `infill_wall_overlap` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1124`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4041-4052`: `top_bottom_infill_wall_overlap` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1125`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4054-4061`: `sparse_infill_speed` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/ratio/gui-type/mode metadata beyond the current registry boundary.
- Wrapping/clumping detection logic and wipe-tower behavior.
- Wrapping exclude area geometry behavior.
- Sparse-infill filament/extruder routing.
- Sparse-infill line-width resolution over nozzle diameter.
- Infill/wall and top/bottom overlap geometry behavior.
- Sparse-infill speed runtime behavior.
- Typed accessors or behavior changes for the newly registered/refreshed keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4063+`: `inherits`, `inherits_group`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `enable_wrapping_detection` and keep process-range definitions below the 400 LOC threshold.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_filament.rs`: split the existing filament-tail definitions out of `pre_middle_process.rs` without changing their metadata.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merge the new `pre_middle_filament` shard between `pre_middle_process` and `pre_middle_tail` to preserve sorted lookup order.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definition for `infill_wall_overlap`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `sparse_infill_filament`, `top_bottom_infill_wall_overlap`, `wrapping_detection_layers`, and `wrapping_exclude_area`; refresh `sparse_infill_line_width` and `sparse_infill_speed` source citations to include `PrintConfig.hpp` field lines.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests for newly registered keys.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod wrapping;` for wrapping-specific metadata tests.
- `crates/ares-core/src/options/registry/tests/metadata/infill.rs`: source metadata assertions for sparse-infill utility keys.
- `crates/ares-core/src/options/registry/tests/metadata/wrapping.rs`: source metadata assertions for wrapping keys.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_wrapping;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_infill.rs`: public lookup coverage for sparse-infill utility keys.
- `crates/ares-core/src/options/tests/registry_lookup_wrapping.rs`: public lookup coverage for wrapping keys.
- `docs/roadmap.md` and `docs/milestones/m89-print-config-wrapping-sparse-infill-registry.md`: milestone sequencing docs.

## Included option definitions

Add or refresh registry metadata for these exact upstream options and default values:

- `enable_wrapping_detection` (`coBool`, default `false`, field at `PrintConfig.hpp:1348`, definition lines 3987-3991, Ares kind `Bool`)
- `wrapping_detection_layers` (`coInt`, default `20`, field at `PrintConfig.hpp:1349`, definition lines 3993-3998, Ares kind `Int`)
- `wrapping_exclude_area` (`coPoints`, default empty points represented as `0x0`, field at `PrintConfig.hpp:1350`, definition lines 4000-4005, Ares kind `Points`)
- `sparse_infill_filament` (`coInt`, default `1`, field at `PrintConfig.hpp:1121`, definition lines 4007-4014, Ares kind `Int`)
- `sparse_infill_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1122`, definition lines 4016-4026, Ares kind `FloatOrPercent`)
- `infill_wall_overlap` (`coPercent`, default `15`, field at `PrintConfig.hpp:1123`, definition lines 4028-4039, Ares kind `Percent`)
- `top_bottom_infill_wall_overlap` (`coPercent`, default `25`, field at `PrintConfig.hpp:1124`, definition lines 4041-4052, Ares kind `Percent`)
- `sparse_infill_speed` (`coFloat`, default `100`, field at `PrintConfig.hpp:1125`, definition lines 4054-4061, Ares kind `Float`)

`Sparse_infill_line_width` and `sparse_infill_speed` already exist in the registry before M89; M89 refreshes their source citations and adds focused coverage without changing their kinds/defaults.

## Functional requirements

1. Add the missing included options to existing sorted definition shards using existing value kinds only.
2. Refresh `sparse_infill_line_width` and `sparse_infill_speed` source citations to include their `PrintConfig.hpp` field lines while preserving kind/default.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wrapping detection behavior, sparse-infill filament routing, line-width resolution behavior, overlap behavior, speed behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `inherits`, `inherits_group`, or following options from `PrintConfig.cpp:4063+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC by splitting `pre_middle_process` into a focused `pre_middle_filament` shard when M89 pushes the existing file to the limit; create focused wrapping tests instead of growing unrelated near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3987-4061` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Wrapping/clumping detection, wrapping exclude area geometry, sparse-infill filament routing, line-width resolution, overlap behavior, sparse-infill speed runtime behavior, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `inherits`, `inherits_group`, and following options from `PrintConfig.cpp:4063+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all eight included keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eight included keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `inherits`/`inherits_group` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
