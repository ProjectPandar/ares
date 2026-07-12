# M88 Spec: PrintConfig skin, skeleton, and combined-infill registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` skin/skeleton infill density, depth, line-width, symmetric-infill, and combined-infill max-layer-height option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1126`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3898-3909`: `skeleton_infill_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1127`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3911-3922`: `skin_infill_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1129`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3924-3932`: `skin_infill_depth` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1128`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3934-3942`: `infill_lock_depth` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1130`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3944-3952`: `skin_infill_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1131`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3954-3962`: `skeleton_infill_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1098`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3964-3970`: `symmetric_infill_y_axis` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1134`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3972-3984`: `infill_combination_max_layer_height` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/category/sidetext/min/max/ratio/mode metadata beyond the current registry boundary.
- Skin/skeleton region generation and density behavior.
- Infill lock depth geometry/path behavior.
- Skin/skeleton line-width resolution over nozzle diameter.
- Symmetric Y-axis infill texture behavior.
- Combined-infill max-layer-height resolution and layer-combination behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3986+`: BBS clumping/wrapping detection and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `infill_combination_max_layer_height` and `infill_lock_depth`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `skeleton_infill_density`, `skeleton_infill_line_width`, `skin_infill_density`, `skin_infill_depth`, `skin_infill_line_width`, and `symmetric_infill_y_axis`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: reuse the existing `mod infill;` metadata module.
- `crates/ares-core/src/options/registry/tests/metadata/infill.rs`: source metadata assertions for all eight options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_infill.rs`: public lookup coverage for all eight options.
- `docs/roadmap.md` and `docs/milestones/m88-print-config-skin-skeleton-infill-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `skeleton_infill_density` (`coPercent`, default `25`, field at `PrintConfig.hpp:1126`, definition lines 3898-3909, Ares kind `Percent`)
- `skin_infill_density` (`coPercent`, default `25`, field at `PrintConfig.hpp:1127`, definition lines 3911-3922, Ares kind `Percent`)
- `skin_infill_depth` (`coFloat`, default `2`, field at `PrintConfig.hpp:1129`, definition lines 3924-3932, Ares kind `Float`)
- `infill_lock_depth` (`coFloat`, default `1`, field at `PrintConfig.hpp:1128`, definition lines 3934-3942, Ares kind `Float`)
- `skin_infill_line_width` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1130`, definition lines 3944-3952, Ares kind `FloatOrPercent`)
- `skeleton_infill_line_width` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1131`, definition lines 3954-3962, Ares kind `FloatOrPercent`)
- `symmetric_infill_y_axis` (`coBool`, default `false`, field at `PrintConfig.hpp:1098`, definition lines 3964-3970, Ares kind `Bool`)
- `infill_combination_max_layer_height` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1134`, definition lines 3972-3984, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, skin/skeleton infill behavior, infill-lock behavior, symmetric-infill behavior, combined-infill max-layer-height behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter BBS clumping/wrapping detection options from `PrintConfig.cpp:3986+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; reuse existing focused infill test files instead of growing unrelated near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3898-3984` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Skin/skeleton infill region behavior, infill lock depth, skin/skeleton line-width resolution, symmetric Y-axis infill, combined-infill max-layer-height behavior, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- BBS clumping/wrapping detection and following options from `PrintConfig.cpp:3986+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all eight new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eight new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following BBS clumping/wrapping detection scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
