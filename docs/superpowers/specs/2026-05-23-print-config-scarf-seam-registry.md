# M107 Spec: PrintConfig scarf seam registry slice

## Goal
Port the adjacent scarf-seam option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:216-220`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1224`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:360-365`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5392-5404`: `seam_slope_type` enum map and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1225`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5406-5411`: `seam_slope_conditional` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1226`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5413-5423`: `scarf_angle_threshold` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1234`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5425-5435`: `scarf_overhang_threshold` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1232`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5437-5449`: `scarf_joint_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1233`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5451-5458`: `scarf_joint_flow_ratio` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1227`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5460-5469`: `seam_slope_start_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1228`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5471-5476`: `seam_slope_entire_loop` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1229`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5478-5485`: `seam_slope_min_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1230`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5487-5493`: `seam_slope_steps` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1231`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5495-5500`: `seam_slope_inner_walls` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/category/ratio-over metadata beyond the current registry boundary.
- Scarf seam planning, conditional scarf selection, overhang estimation, scarf speed/flow application, seam slope geometry, scarf around whole wall, scarf length/step generation, and inner-wall scarf behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5502+`: `role_based_wipe_speed`, `wipe_speed`, loop-wipe options, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add all eleven definitions in sorted order between existing `scan_first_layer` / `seam_*` / `set_other_flow_ratios` entries.
- Registry key, metadata, fixture-count, and public lookup tests cover all eleven definitions.
- `docs/roadmap.md` and `docs/milestones/m107-print-config-scarf-seam-registry.md`: milestone sequencing docs.

## Included option definitions

- `seam_slope_type` (`coEnum`, default `none`, enum at `PrintConfig.hpp:216-220`, field at `PrintConfig.hpp:1224`, enum map at `PrintConfig.cpp:360-365`, definition lines 5392-5404, Ares kind `Enum`)
- `seam_slope_conditional` (`coBool`, default `false`, field at `PrintConfig.hpp:1225`, definition lines 5406-5411, Ares kind `Bool`)
- `scarf_angle_threshold` (`coInt`, default `155`, field at `PrintConfig.hpp:1226`, definition lines 5413-5423, Ares kind `Int`)
- `scarf_overhang_threshold` (`coPercent`, default `40`, field at `PrintConfig.hpp:1234`, definition lines 5425-5435, Ares kind `Percent`)
- `scarf_joint_speed` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1232`, definition lines 5437-5449, Ares kind `FloatOrPercent`)
- `scarf_joint_flow_ratio` (`coFloat`, default `1`, field at `PrintConfig.hpp:1233`, definition lines 5451-5458, Ares kind `Float`)
- `seam_slope_start_height` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1227`, definition lines 5460-5469, Ares kind `FloatOrPercent`)
- `seam_slope_entire_loop` (`coBool`, default `false`, field at `PrintConfig.hpp:1228`, definition lines 5471-5476, Ares kind `Bool`)
- `seam_slope_min_length` (`coFloat`, default `20`, field at `PrintConfig.hpp:1229`, definition lines 5478-5485, Ares kind `Float`)
- `seam_slope_steps` (`coInt`, default `10`, field at `PrintConfig.hpp:1230`, definition lines 5487-5493, Ares kind `Int`)
- `seam_slope_inner_walls` (`coBool`, default `false`, field at `PrintConfig.hpp:1231`, definition lines 5495-5500, Ares kind `Bool`)

## Functional requirements

1. Add the eleven missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, scarf seam planning, conditional scarf selection, overhang estimation, scarf speed/flow behavior, seam slope geometry behavior, inner-wall scarf behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following wipe-speed options from `PrintConfig.cpp:5502+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the eleven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eleven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, scarf seam runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5502+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
