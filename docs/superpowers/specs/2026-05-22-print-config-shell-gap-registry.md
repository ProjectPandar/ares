# M32 Spec: PrintConfig shell and gap-fill option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering before-layer-change G-code, bottom shell, and gap-fill target options without adding G-code hook execution or shell/gap-fill behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:241-243`: `GapFillTarget` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1038`: typed `ConfigOptionEnum<GapFillTarget>` field for `gap_fill_target`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1079-1080`: typed bottom shell fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1294`: typed G-code hook field for `before_layer_change_gcode`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:393-398`: `GapFillTarget` enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1110-1168`: `PrintConfigDef::init_fff_params()` before-layer-change, bottom shell, and gap-fill target option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `before_layer_change_gcode` (`coString`, default ``, lines 1110-1117)
- `bottom_shell_layers` (`coInt`, default `3`, lines 1119-1128)
- `bottom_shell_thickness` (`coFloat`, default `0`, lines 1130-1139)
- `gap_fill_target` (`coEnum`, default `nowhere` from `gftNowhere`, lines 1141-1168)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind` variants.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, enum value APIs, G-code hook execution, bottom shell behavior, or gap-fill behavior for these options in this milestone.
6. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or UI behavior.
7. Update roadmap and milestone docs so E2E parity moves to M33.
8. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Overhang fan options from `PrintConfig.cpp:1170-1211` are deferred.
- `bridge_angle` and following bridge/shell options from `PrintConfig.cpp:1213+` are deferred.
- Actual before-layer-change G-code insertion, bottom shell planning, and gap-fill generation are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
