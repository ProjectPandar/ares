# M112 Spec: PrintConfig timelapse and preheat registry slice

## Goal
Port the adjacent timelapse, standby temperature, and preheat option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:281-284`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1615`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:431-435`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5728-5743`: `timelapse_type` enum map and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1565`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5745-5755`: `standby_temperature_delta` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1566`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5757-5765`: `preheat_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1567`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5767-5774`: `preheat_steps` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Timelapse capture behavior, smooth-timelapse prime-tower validation, ooze-prevention standby temperature application, preheat M104/M104.1 command insertion, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5777+`: `file_start_gcode`, `machine_start_gcode`, filament start/change G-code, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `preheat_steps` and `preheat_time` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: keep earlier `solid_*`, `sparse_*`, and `spiral_*` entries.
- Create `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: move the existing `staggered_inner_seams` through `zaa_minimize_perimeter_height` tail-final entries into this new sorted shard, then add `standby_temperature_delta` and `timelapse_type` in sorted order.
- Update `crates/ares-core/src/options/registry/definitions/table.rs` to merge `tail_terminal` immediately after `tail_final`, preserving the single sorted `OPTION_DEFINITIONS` stream.
- Registry key, metadata, fixture-count, and public lookup tests cover all four definitions.
- `docs/roadmap.md` and `docs/milestones/m112-print-config-timelapse-preheat-registry.md`: milestone sequencing docs.

## Included option definitions

- `timelapse_type` (`coEnum`, default `0`, enum at `PrintConfig.hpp:281-284`, field at `PrintConfig.hpp:1615`, enum map at `PrintConfig.cpp:431-435`, definition lines 5728-5743, Ares kind `Enum`)
- `standby_temperature_delta` (`coInt`, default `-5`, field at `PrintConfig.hpp:1565`, definition lines 5745-5755, Ares kind `Int`)
- `preheat_time` (`coFloat`, default `30`, field at `PrintConfig.hpp:1566`, definition lines 5757-5765, Ares kind `Float`)
- `preheat_steps` (`coInt`, default `1`, field at `PrintConfig.hpp:1567`, definition lines 5767-5774, Ares kind `Int`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Perform only the minimum mechanical registry-table shard split needed to keep modified Rust files under 400 LOC.
5. Preserve `SliceOptions` unknown-value storage and current public slicing/skirt/infill/speed API behavior.
6. Do not add typed parsing/accessors, timelapse runtime behavior, standby temperature behavior, preheat G-code behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add following file/machine/filament start G-code options from `PrintConfig.cpp:5777+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for the mechanical shard split, deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5777+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
