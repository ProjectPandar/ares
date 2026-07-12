# M108 Spec: PrintConfig wipe speed and loop registry slice

## Goal
Port the adjacent wipe-speed and loop-wipe option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1183`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5502-5508`: `role_based_wipe_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1185`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5510-5515`: `wipe_on_loops` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1186`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5517-5526`: `wipe_before_external_loop` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1184`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5528-5538`: `wipe_speed` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/category/ratio-over metadata beyond the current registry boundary.
- Role-based wipe speed selection, wipe speed calculation against travel speed, loop-wipe movement, external-loop wipe placement, and any wipe-path planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5540+`: skirt distance/start angle/height, draft shield, skirt type/loops/speed, and following options. Existing Ares skirt registry/runtime behavior is not changed by this milestone.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `role_based_wipe_speed` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `wipe_before_external_loop`, `wipe_on_loops`, and `wipe_speed` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all four definitions.
- `docs/roadmap.md` and `docs/milestones/m108-print-config-wipe-speed-loop-registry.md`: milestone sequencing docs.

## Included option definitions

- `role_based_wipe_speed` (`coBool`, default `true`, field at `PrintConfig.hpp:1183`, definition lines 5502-5508, Ares kind `Bool`)
- `wipe_on_loops` (`coBool`, default `false`, field at `PrintConfig.hpp:1185`, definition lines 5510-5515, Ares kind `Bool`)
- `wipe_before_external_loop` (`coBool`, default `false`, field at `PrintConfig.hpp:1186`, definition lines 5517-5526, Ares kind `Bool`)
- `wipe_speed` (`coFloatOrPercent`, default `80%`, field at `PrintConfig.hpp:1184`, definition lines 5528-5538, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the four missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, role-based wipe speed selection, wipe speed calculation, loop-wipe movement, external-loop wipe placement, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or change following skirt/draft-shield options from `PrintConfig.cpp:5540+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, wipe runtime behavior, existing skirt behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5540+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
