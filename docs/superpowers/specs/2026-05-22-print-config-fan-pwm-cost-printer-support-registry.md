# M85 Spec: PrintConfig fan PWM, cost, and printer support registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` part-cooling PWM clamp, time-cost, chamber-temperature support, and air-filtration support option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1316`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3740-3760`: `part_cooling_fan_min_pwm` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1357`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3763-3769`: `time_cost` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1407`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3771-3777`: `support_chamber_temp_control` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1405`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3779-3783`: `support_air_filtration` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode/readonly metadata beyond the current registry boundary.
- Fan PWM clamp behavior and firmware threshold behavior.
- Time-cost calculation behavior.
- Chamber-temperature control behavior and M141 command emission.
- Air-filtration support behavior and M106 P3 command emission.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785+`: `gcode_flavor` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `part_cooling_fan_min_pwm`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `support_air_filtration`, `support_chamber_temp_control`, and `time_cost`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod printer_support;`.
- `crates/ares-core/src/options/registry/tests/metadata/printer_support.rs`: source metadata assertions for all four options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_printer_support;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_printer_support.rs`: public lookup coverage for all four options.
- `docs/roadmap.md` and `docs/milestones/m85-print-config-fan-pwm-cost-printer-support-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `part_cooling_fan_min_pwm` (`coInt`, default `0`, field at `PrintConfig.hpp:1316`, definition lines 3740-3760, Ares kind `Int`)
- `time_cost` (`coFloat`, default `0`, field at `PrintConfig.hpp:1357`, definition lines 3763-3769, Ares kind `Float`)
- `support_chamber_temp_control` (`coBool`, default `true`, field at `PrintConfig.hpp:1407`, definition lines 3771-3777, Ares kind `Bool`)
- `support_air_filtration` (`coBool`, default `true`, field at `PrintConfig.hpp:1405`, definition lines 3779-3783, Ares kind `Bool`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, part-cooling PWM clamp behavior, time-cost behavior, chamber-temperature behavior, air-filtration behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `gcode_flavor` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3740-3783` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Fan PWM clamping, time-cost calculation, chamber-temperature control, air-filtration G-code command behavior, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `gcode_flavor` and following options from `PrintConfig.cpp:3785+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `gcode_flavor` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
