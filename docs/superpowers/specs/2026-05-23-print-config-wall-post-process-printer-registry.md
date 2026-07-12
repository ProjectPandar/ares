# M101 Spec: PrintConfig wall loop, post-process, and printer identity registry slice

## Goal
Port the adjacent wall-loop, alternate-extra-wall, post-processing script, process role-change G-code, printer identity, and print/printer settings-id option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1158`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4918-4924`: `wall_loops` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1159`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4926-4933`: `alternate_extra_wall` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1547`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4935-4946`: `post_process` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1394`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4948-4955`: `process_change_extrusion_role_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1548`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4957-4961`: `printer_model` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1634`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4963-4970`: `printer_notes` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4972-4976`: `printer_variant` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4978-4981`: `print_settings_id` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4983-4986`: `printer_settings_id` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/max-literal/gui-type/flags/multiline/full-width/height/CLI metadata beyond the current registry boundary.
- Wall-loop generation, alternate-extra-wall planning, post-processing script execution, process role-change G-code insertion, printer preset identity semantics, and printer settings-id behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4988+`: raft support, resolution, retraction, and following options.
- Slicing, extrusion, G-code behavior, filesystem execution behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `alternate_extra_wall`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs` and `late_tail_final.rs`: split the near-limit late-tail shard behavior-preservingly and add sorted definition for `post_process`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs` or adjacent sorted shard: add sorted definitions for `print_settings_id`, `printer_model`, `printer_notes`, `printer_settings_id`, `printer_variant`, and `process_change_extrusion_role_gcode` according to the current merged registry order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add sorted definition for `wall_loops` after `wall_filament` and before `wall_sequence` if shard order remains appropriate.
- Registry key, metadata, fixture-count, and public lookup tests cover all nine definitions.
- `docs/roadmap.md` and `docs/milestones/m101-print-config-wall-post-process-printer-registry.md`: milestone sequencing docs.

## Included option definitions

- `wall_loops` (`coInt`, default `2`, field at `PrintConfig.hpp:1158`, definition lines 4918-4924, Ares kind `Int`)
- `alternate_extra_wall` (`coBool`, default `false`, field at `PrintConfig.hpp:1159`, definition lines 4926-4933, Ares kind `Bool`)
- `post_process` (`coStrings`, default empty strings list, field at `PrintConfig.hpp:1547`, definition lines 4935-4946, Ares kind `Strings`)
- `process_change_extrusion_role_gcode` (`coString`, default empty string, field at `PrintConfig.hpp:1394`, definition lines 4948-4955, Ares kind `String`)
- `printer_model` (`coString`, default empty string, field at `PrintConfig.hpp:1548`, definition lines 4957-4961, Ares kind `String`)
- `printer_notes` (`coString`, default empty string, field at `PrintConfig.hpp:1634`, definition lines 4963-4970, Ares kind `String`)
- `printer_variant` (`coString`, default empty string, definition lines 4972-4976, Ares kind `String`)
- `print_settings_id` (`coString`, default empty string, definition lines 4978-4981, Ares kind `String`)
- `printer_settings_id` (`coString`, default empty string, definition lines 4983-4986, Ares kind `String`)

## Functional requirements

1. Add the nine missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, wall-loop generation, alternate-extra-wall planning, post-processing execution, role-change G-code insertion, printer identity behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following raft, resolution, retraction, or later options from `PrintConfig.cpp:4988+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, wall/post-process/printer runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:4988+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
