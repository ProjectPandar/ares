# M46 Spec: PrintConfig compatible profile option registry slice

## Goal
Port the adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` compatible profile option-definition slice into `ares-core` option registry metadata, covering no-CLI compatible printer/profile list and expression-group keys without changing profile composition, expression evaluation, preset filtering, slicing, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1695-1748`: `PrintConfigDef::init_fff_params()` compatible printer/profile no-CLI option definitions.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/Preset.cpp` and `PresetBundle.cpp`: compatibility-expression evaluation, preset filtering, and full profile composition behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1750+`: `print_sequence`, `print_order`, and following print-order options.
- Project-file persistence semantics for expression-group fields beyond preserving option values in `SliceOptions`.
- UI visibility, CLI no-CLI enforcement, filesystem/network integrations, slicing, extrusion, and G-code behavior.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `compatible_*` keys that sort before `cool_plate_temp` plus `different_settings_to_system`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definition shard for `print_compatible_printers` and `upward_compatible_machine`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs` plus `metadata/profile.rs`: focused profile compatibility metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `compatible_printers` (`coStrings`, default empty strings list, lines 1695-1699)
- `upward_compatible_machine` (`coStrings`, default empty strings list, lines 1702-1706)
- `compatible_printers_condition` (`coString`, default empty string, lines 1708-1715)
- `compatible_prints` (`coStrings`, default empty strings list, lines 1717-1721)
- `compatible_prints_condition` (`coString`, default empty string, lines 1723-1730)
- `compatible_machine_expression_group` (`coStrings`, default empty strings list, lines 1734-1736)
- `compatible_process_expression_group` (`coStrings`, default empty strings list, lines 1737-1739)
- `different_settings_to_system` (`coStrings`, default empty strings list, lines 1742-1744)
- `print_compatible_printers` (`coStrings`, default empty strings list, lines 1746-1748)

The current `OptionDefinition` default representation for empty `coString` and empty `coStrings` values is the existing empty string literal `""`.

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::String` and `OptionValueKind::Strings`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, compatibility-expression evaluation, preset filtering, profile composition behavior changes, CLI no-CLI enforcement, project-file persistence semantics, object override behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `print_sequence`, `print_order`, or following options from `PrintConfig.cpp:1750+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M47, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/tooltip/mode/CLI metadata from `PrintConfig.cpp:1695-1748` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Compatibility-expression evaluation, preset filtering, and profile composition behavior changes are deferred to source-cited `Preset.*` / `PresetBundle.*` milestones.
- Project-file persistence semantics for expression groups are deferred beyond generic `SliceOptions` value preservation.
- CLI no-CLI enforcement, UI visibility behavior, object override handling, slicing behavior, extrusion behavior, and G-code behavior are deferred.
- `print_sequence`, `print_order`, and following options from `PrintConfig.cpp:1750+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all nine new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, CLI no-CLI enforcement, compatibility-expression behavior, preset filtering, profile composition changes, project-file semantics, object override behavior, slicing/extrusion/G-code behavior, and following print-order options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
