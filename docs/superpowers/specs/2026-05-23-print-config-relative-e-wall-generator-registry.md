# M146 Spec: PrintConfig relative E and wall-generator registry slice

## Goal
Port the adjacent relative extrusion and wall-generator option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1418`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6980-6987`: `use_relative_e_distances` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:294-300`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1020`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:520-524`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6989-7001`: `PerimeterGeneratorType` enum map and `wall_generator` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode metadata beyond the current registry metadata boundary.
- Relative extrusion addressing in `GCodeWriter`, `GCode`, `GCodeReader`, `PressureEqualizer`, `CoolingBuffer`, `SpiralVase`, `Extruder`, and wipe-tower validation.
- Classic/Arachne perimeter generation and all wall-transition behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003+`: `wall_transition_length`, `wall_transition_filter_deviation`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `use_relative_e_distances` after `use_firmware_retraction` and before `volumetric_speed_coefficients`; add `wall_generator` after `wall_filament` and before `wall_loops`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add both keys in matching sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/relative_e_wall_generator.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_relative_e_wall_generator.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for both covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by two.
- `docs/roadmap.md` and `docs/milestones/m146-print-config-relative-e-wall-generator-registry.md`: milestone sequencing docs.

## Included option definitions

- `use_relative_e_distances` (`coBool`, default `true`, field at `PrintConfig.hpp:1418`, definition lines 6980-6987, Ares kind `Bool`)
- `wall_generator` (`coEnum`, default `arachne`, enum at `PrintConfig.hpp:294-300`, field at `PrintConfig.hpp:1020`, enum map lines 520-524, definition lines 6989-7001, Ares kind `Enum`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, relative-E G-code behavior, wipe-tower relative-E validation, wall-generator/perimeter behavior, wall-transition behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wall_transition_length`, `wall_transition_filter_deviation`, or following options from `PrintConfig.cpp:7003+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; no registry shard split is expected for this milestone unless implementation evidence shows it is required.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, relative-E G-code behavior, wall-generator/perimeter behavior, wall-transition behavior, and following `PrintConfig.cpp:7003+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
