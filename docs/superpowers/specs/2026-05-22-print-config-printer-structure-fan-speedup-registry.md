# M84 Spec: PrintConfig printer structure and fan speed-up registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` printer-structure, best-object-position, auxiliary-fan, and fan speed-up/kick-start option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:357-363`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:494-501`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1406`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3681-3696`: `printer_structure` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1541`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3698-3702`: `best_object_pos` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1404`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3704-3708`: `auxiliary_fan` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1312`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3710-3721`: `fan_speedup_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1311`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3723-3727`: `fan_speedup_overhangs` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1310`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3729-3738`: `fan_kickstart` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/mode/enum label metadata beyond the current registry boundary.
- Printer-structure runtime semantics and auto-arrange best-object-position behavior.
- Fan command scheduling, overhang-specific fan speed-up behavior, fan kick-start command emission, and PWM behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3740+`: `part_cooling_fan_min_pwm` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `auxiliary_fan` and `best_object_pos`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definitions for `fan_kickstart`, `fan_speedup_overhangs`, and `fan_speedup_time`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `printer_structure`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod printer;`.
- `crates/ares-core/src/options/registry/tests/metadata/printer.rs`: source metadata assertions for all six options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_printer;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_printer.rs`: public lookup coverage for all six options.
- `docs/roadmap.md` and `docs/milestones/m84-print-config-printer-structure-fan-speedup-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `printer_structure` (`coEnum`, default `psUndefine`, field at `PrintConfig.hpp:1406`, enum at `PrintConfig.hpp:357-363`, enum map at `PrintConfig.cpp:494-501`, definition lines 3681-3696, Ares kind `Enum`, default string `undefine`)
- `best_object_pos` (`coPoint`, default `Vec2d(0.5, 0.5)`, field at `PrintConfig.hpp:1541`, definition lines 3698-3702, Ares kind `Point`, default string `0.5x0.5`)
- `auxiliary_fan` (`coBool`, default `false`, field at `PrintConfig.hpp:1404`, definition lines 3704-3708, Ares kind `Bool`)
- `fan_speedup_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1312`, definition lines 3710-3721, Ares kind `Float`)
- `fan_speedup_overhangs` (`coBool`, default `true`, field at `PrintConfig.hpp:1311`, definition lines 3723-3727, Ares kind `Bool`)
- `fan_kickstart` (`coFloat`, default `0`, field at `PrintConfig.hpp:1310`, definition lines 3729-3738, Ares kind `Float`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, printer-structure behavior, best-object-position behavior, fan scheduling/kick-start behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `part_cooling_fan_min_pwm` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3681-3738` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Printer structure semantics, auto-arrange best-object-position behavior, fan speed-up scheduling, fan kick-start command behavior, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `part_cooling_fan_min_pwm` and following options from `PrintConfig.cpp:3740+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all six new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `part_cooling_fan_min_pwm` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
