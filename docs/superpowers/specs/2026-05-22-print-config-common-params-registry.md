# M26 Spec: PrintConfig common params option registry slice

## Goal
Port the next source-cited `libslic3r::PrintConfigDef` option-definition slice into `ares-core` by adding registry metadata for the first common printer/quality parameters from `PrintConfigDef::init_common_params`, without adding new slicing behavior or pipeline stages.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp`: `PrintConfigDef` / config option definition model.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:672-782`: first `PrintConfigDef::init_common_params()` option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: source-cited `OptionDefinition` metadata.
- `crates/ares-core/src/options/tests/registry_helpers.rs` and registry unit tests: coverage for lookup/count/sorted definitions.

## Included option definitions

Add registry metadata for these exact upstream options and default values. The same upstream range also contains `layer_height` (`PrintConfig.cpp:749-755`), which is already covered by the existing `ares-core` registry entry from prior milestones and is intentionally reused rather than re-added.

- `printer_technology` (`coEnum`, default `FFF`, lines 676-682)
- `printable_area` (`coPoints`, default `0x0,200x0,200x200,0x200`, lines 684-688)
- `extruder_printable_area` (`coPointsGroups`, default empty, lines 690-694)
- `bed_exclude_area` (`coPoints`, default `0x0`, lines 696-703)
- `bed_custom_texture` (`coString`, default empty, lines 705-709)
- `bed_custom_model` (`coString`, default empty, lines 711-715)
- `elefant_foot_compensation` (`coFloat`, default `0`, lines 717-724)
- `elefant_foot_compensation_layers` (`coInt`, default `1`, lines 726-735)
- `elefant_foot_layers_density` (`coPercent`, default `100`, lines 737-747)
- `printable_height` (`coFloat`, default `100`, lines 757-764)
- `extruder_printable_height` (`coFloats`, default `0`, lines 766-773)
- `preferred_orientation` (`coFloat`, default `0`, lines 775-782)

## Functional requirements

1. Extend `OptionValueKind` only as needed for the included upstream config kinds (`coString`, `coPoints`, `coPointsGroups`).
2. Add the included options to `OPTION_DEFINITIONS` in sorted key order.
3. Preserve `option_definition(key)` binary-search lookup behavior.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors or behavior for these options in this milestone unless already needed for registry tests.
6. Do not add a new pipeline stage, crate, dependency, or G-code behavior.
7. Update M26/M27 roadmap and milestone docs so E2E parity moves to M27.
8. Modified Rust files must remain under 400 LOC.

## Existing scaffold reused

- `layer_height` (`coFloat`, default `0.2`, `PrintConfig.cpp:749-755`) remains covered by the existing registry entry and typed accessor. This milestone does not duplicate or rename it.

## Deferred behavior

- Physical printer host common params after line 786 are deferred.
- FFF-specific option definitions from `PrintConfigDef::init_fff_params()` are deferred except options already covered by prior milestones.
- Typed accessors and behavior for `printable_area`, elephant-foot compensation, orientation, and printer host options are deferred to later source-cited milestones.
- Full option registry parity with every OrcaSlicer option remains incremental.

## Acceptance checks

- Registry tests prove new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while still preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
