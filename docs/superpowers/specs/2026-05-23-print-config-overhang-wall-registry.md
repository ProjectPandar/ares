# M100 Spec: PrintConfig make-overhang and wall registry slice

## Goal
Port the adjacent make-overhang-printable, overhang-wall detection, wall-filament, and inner-wall width/speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1199`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850-4855`: `make_overhang_printable` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1032`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4857-4867`: `make_overhang_printable_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1033`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4869-4877`: `make_overhang_printable_hole_size` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1153`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4879-4885`: `detect_overhang_wall` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1154`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4887-4894`: `wall_filament` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1155`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4896-4906`: `inner_wall_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1156`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4908-4916`: `inner_wall_speed` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/max-literal/gui-type/alias/ratio metadata beyond the current registry boundary.
- Make-overhang-printable geometry modification, overhang-wall detection, wall-filament routing, inner-wall line-width resolution, inner-wall speed planning, and G-code generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4918+`: `wall_loops`, `alternate_extra_wall`, `post_process`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `detect_overhang_wall`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted definitions for `make_overhang_printable`, `make_overhang_printable_angle`, and `make_overhang_printable_hole_size`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `inner_wall_line_width` and `inner_wall_speed`.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add sorted definition for `wall_filament`.
- Registry key, metadata, fixture-count, and public lookup tests cover all seven definitions.
- `docs/roadmap.md` and `docs/milestones/m100-print-config-overhang-wall-registry.md`: milestone sequencing docs.

## Included option definitions

- `make_overhang_printable` (`coBool`, default `false`, field at `PrintConfig.hpp:1199`, definition lines 4850-4855, Ares kind `Bool`)
- `make_overhang_printable_angle` (`coFloat`, default `55`, field at `PrintConfig.hpp:1032`, definition lines 4857-4867, Ares kind `Float`)
- `make_overhang_printable_hole_size` (`coFloat`, default `0`, field at `PrintConfig.hpp:1033`, definition lines 4869-4877, Ares kind `Float`)
- `detect_overhang_wall` (`coBool`, default `true`, field at `PrintConfig.hpp:1153`, definition lines 4879-4885, Ares kind `Bool`)
- `wall_filament` (`coInt`, default `1`, field at `PrintConfig.hpp:1154`, definition lines 4887-4894, Ares kind `Int`)
- `inner_wall_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1155`, definition lines 4896-4906, Ares kind `FloatOrPercent`)
- `inner_wall_speed` (`coFloat`, default `60`, field at `PrintConfig.hpp:1156`, definition lines 4908-4916, Ares kind `Float`)

## Functional requirements

1. Add the seven missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, make-overhang geometry behavior, overhang-wall detection behavior, wall-filament routing behavior, line-width resolution, speed planning, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following `wall_loops`, `alternate_extra_wall`, `post_process`, or later options from `PrintConfig.cpp:4918+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the seven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all seven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, make-overhang/wall runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4918+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
