# M113 Spec: PrintConfig start G-code and filament-change registry slice

## Goal
Port the adjacent file/machine/filament start G-code and single-extruder manual filament-change option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1385`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5777-5787`: `file_start_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1386`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5789-5796`: `machine_start_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1387`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5798-5805`: `filament_start_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1388`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5807-5811`: `single_extruder_multi_material` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1389`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5813-5819`: `manual_filament_change` option definition.

Related upstream behavior explicitly deferred:

- UI multiline/full-width/height/mode metadata beyond the current registry boundary.
- File header G-code emission, machine start G-code emission, filament start G-code emission, placeholder expansion, single-extruder multi-material runtime behavior, manual filament-change Tx omission/M600 behavior, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5821+`: `wipe_tower_type`, purge/ramming/tool-change/wipe-tower options, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: add `filament_start_gcode` and `file_start_gcode` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add `machine_start_gcode` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `manual_filament_change` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `single_extruder_multi_material` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all five definitions.
- `docs/roadmap.md` and `docs/milestones/m113-print-config-start-gcode-filament-change-registry.md`: milestone sequencing docs.

## Included option definitions

- `file_start_gcode` (`coString`, default empty string, field at `PrintConfig.hpp:1385`, definition lines 5777-5787, Ares kind `String`)
- `machine_start_gcode` (`coString`, default `G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n`, field at `PrintConfig.hpp:1386`, definition lines 5789-5796, Ares kind `String`)
- `filament_start_gcode` (`coStrings`, default ` `, field at `PrintConfig.hpp:1387`, definition lines 5798-5805, Ares kind `Strings`)
- `single_extruder_multi_material` (`coBool`, default `true`, field at `PrintConfig.hpp:1388`, definition lines 5807-5811, Ares kind `Bool`)
- `manual_filament_change` (`coBool`, default `false`, field at `PrintConfig.hpp:1389`, definition lines 5813-5819, Ares kind `Bool`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, start-G-code emission behavior, placeholder expansion, single-extruder multi-material behavior, manual filament-change behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following wipe-tower/ramming/tool-change options from `PrintConfig.cpp:5821+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5821+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
