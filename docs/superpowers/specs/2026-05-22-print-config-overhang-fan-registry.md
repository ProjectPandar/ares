# M33 Spec: PrintConfig overhang fan option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering overhang/bridge cooling fan options without adding cooling, bridge-detection, or fan G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:304-310`: `OverhangFanThreshold` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1502-1504`: typed overhang fan fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:456-464`: `OverhangFanThreshold` enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1170-1211`: `PrintConfigDef::init_fff_params()` overhang fan option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: `OptionValueKind` vocabulary for upstream `coBools` and `coEnums`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs` and split submodules: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `enable_overhang_bridge_fan` (`coBools`, default `true`, lines 1170-1175)
- `overhang_fan_speed` (`coInts`, default `100`, lines 1177-1188)
- `overhang_fan_threshold` (`coEnums`, default `95%` from `Overhang_threshold_bridge`, lines 1190-1211)

## Functional requirements

1. Extend `OptionValueKind` only as needed for upstream `coBools` and `coEnums` (`Bools`, `Enums`).
2. Add the included options to the sorted definition table.
3. Preserve public API shape: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
4. Preserve binary-search lookup and sorted/no-duplicate test coverage.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, enum value APIs, fan speed control, cooling behavior, bridge-detection behavior, or fan G-code behavior for these options in this milestone.
7. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or UI behavior.
8. Update roadmap and milestone docs so E2E parity moves to M34.
9. Modified Rust files must remain under 400 LOC; because `registry/tests.rs` is near the limit, split registry tests before adding M33 assertions.

## Deferred behavior

- `bridge_angle` and following bridge/shell options from `PrintConfig.cpp:1213+` are deferred.
- Full enum value exposure for `OverhangFanThreshold` is deferred.
- Actual overhang cooling decisions, bridge detection integration, fan speed planning, and fan G-code behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
