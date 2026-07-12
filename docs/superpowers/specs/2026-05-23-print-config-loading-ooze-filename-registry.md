# M99 Spec: PrintConfig loading move, start/end points, ooze, and filename registry slice

## Goal
Port the adjacent extra-loading, start/end point, infill-retraction, ooze-prevention, and filename-format option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1432`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4812-4819`: `extra_loading_move` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1614`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4821-4827`: `start_end_points` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1544`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4829-4835`: `reduce_infill_retraction` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1545`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4837-4841`: `ooze_prevention` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1546`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4843-4848`: `filename_format` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/full-width/readonly/develop-mode metadata beyond the current registry boundary.
- MMU loading/unloading behavior, cutter/start-end point runtime behavior, infill retraction suppression, ooze-prevention inactive-extruder temperature control, filename template rendering, and G-code generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850+`: `make_overhang_printable` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `extra_loading_move`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: add sorted definition for `filename_format`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add sorted definition for `ooze_prevention`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `reduce_infill_retraction`.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add sorted definition for `start_end_points`.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs`: split the near-limit known-count fixture before adding values.
- Registry key, metadata, fixture-count, and public lookup tests cover all five definitions.
- `docs/roadmap.md` and `docs/milestones/m99-print-config-loading-ooze-filename-registry.md`: milestone sequencing docs.

## Included option definitions

- `extra_loading_move` (`coFloat`, default `-2`, field at `PrintConfig.hpp:1432`, definition lines 4812-4819, Ares kind `Float`)
- `start_end_points` (`coPoints`, default `30x-3,54x245`, field at `PrintConfig.hpp:1614`, definition lines 4821-4827, Ares kind `Points`)
- `reduce_infill_retraction` (`coBool`, default `false`, field at `PrintConfig.hpp:1544`, definition lines 4829-4835, Ares kind `Bool`)
- `ooze_prevention` (`coBool`, default `false`, field at `PrintConfig.hpp:1545`, definition lines 4837-4841, Ares kind `Bool`)
- `filename_format` (`coString`, default `{input_filename_base}_{filament_type[initial_tool]}_{print_time}.gcode`, field at `PrintConfig.hpp:1546`, definition lines 4843-4848, Ares kind `String`)

## Functional requirements

1. Split the near-limit known-count fixture before adding more fixture values.
2. Add the five missing options to sorted definition shards using existing value kinds only.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, MMU loading behavior, cutter/start-end runtime behavior, infill retraction suppression, ooze-prevention temperature behavior, filename rendering, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter following `make_overhang_printable` or later options from `PrintConfig.cpp:4850+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, MMU/start-end/retraction/ooze/filename runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4850+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
