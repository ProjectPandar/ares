# M74 Spec: PrintConfig lateral lattice and infill anchor option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` lateral lattice angle, infill overhang angle, and infill anchor option-definition slice into `ares-core` option registry metadata by adding registry coverage for `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, `infill_anchor`, and `infill_anchor_max`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1103`: `lateral_lattice_angle_1` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2987-2995`: `lateral_lattice_angle_1` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1104`: `lateral_lattice_angle_2` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2997-3005`: `lateral_lattice_angle_2` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1105`: `infill_overhang_angle` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3007-3015`: `infill_overhang_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1195`: `infill_anchor` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3017-3043`: `infill_anchor` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1196`: `infill_anchor_max` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3045-3066`: `infill_anchor_max` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode/ratio/gui/enum metadata beyond the current registry boundary.
- Lateral lattice direction behavior, infill overhang behavior, and infill anchor runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3068+`: `inner_wall_acceleration`, `travel_acceleration`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `infill_anchor`, `infill_anchor_max`, and `infill_overhang_angle`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for `lateral_lattice_angle_1` and `lateral_lattice_angle_2`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/strength.rs`: extend source metadata assertions for the five options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_strength.rs`: extend public lookup coverage for the five options.
- `docs/roadmap.md` and `docs/milestones/m74-print-config-lateral-lattice-infill-anchor-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `lateral_lattice_angle_1` (`coFloat`, default `-45`, field at `PrintConfig.hpp:1103`, definition lines 2987-2995, Ares kind `Float`)
- `lateral_lattice_angle_2` (`coFloat`, default `45`, field at `PrintConfig.hpp:1104`, definition lines 2997-3005, Ares kind `Float`)
- `infill_overhang_angle` (`coFloat`, default `60`, field at `PrintConfig.hpp:1105`, definition lines 3007-3015, Ares kind `Float`)
- `infill_anchor` (`coFloatOrPercent`, default `400%`, field at `PrintConfig.hpp:1195`, definition lines 3017-3043, Ares kind `FloatOrPercent`)
- `infill_anchor_max` (`coFloatOrPercent`, default `20`, field at `PrintConfig.hpp:1196`, definition lines 3045-3066, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Float` and `FloatOrPercent`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, lateral lattice behavior, infill overhang behavior, infill anchor behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `inner_wall_acceleration`, `travel_acceleration`, or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, ratio, GUI, and enum metadata from `PrintConfig.cpp:2987-3066` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Lateral lattice behavior, infill overhang behavior, infill anchor behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `inner_wall_acceleration`, `travel_acceleration`, and following options from `PrintConfig.cpp:3068+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI/bounds/ratio metadata, lateral lattice/infill anchor behavior, slicing/extrusion/G-code behavior, and following `inner_wall_acceleration` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
