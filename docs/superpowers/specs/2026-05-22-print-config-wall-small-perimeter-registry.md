# M51 Spec: PrintConfig wall ordering and small perimeter option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` wall and small-perimeter option-definition slice into `ares-core` option registry metadata by adding missing registry coverage for `small_perimeter_speed`, `small_perimeter_threshold`, `wall_sequence`, and `wall_direction` while preserving already registered `outer_wall_line_width`, `outer_wall_speed`, and `is_infill_first` metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:132-137`: `WallSequence` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:140-144`: `WallDirection` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1191-1192`: `small_perimeter_speed` and `small_perimeter_threshold` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1209-1212`: `wall_sequence`, `is_infill_first`, and `wall_direction` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:277-290`: `WallSequence` and `WallDirection` enum key maps.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2110`: option definitions for this slice.

Related upstream behavior explicitly deferred:

- Small-perimeter detection, threshold evaluation, speed planning, and feedrate changes.
- Wall print ordering and wall direction path generation.
- Typed accessors or behavior changes for `wall_sequence`, `wall_direction`, `small_perimeter_speed`, or `small_perimeter_threshold`.
- UI label/category/tooltip/enum-label/mode/sidetext/ratio/min/max metadata.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2112+`: `extruder` and following options.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for `small_perimeter_speed`, `small_perimeter_threshold`, `wall_direction`, and `wall_sequence`; preserve existing `outer_wall_line_width`, `outer_wall_speed`, and `is_infill_first` definitions.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: small-perimeter speed metadata assertions.
- `crates/ares-core/src/options/registry/tests/metadata/quality.rs`: wall sequence/direction and small-perimeter threshold metadata assertions.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_wall.rs`: new public lookup coverage file so existing near-400-LOC `registry_lookup.rs` is not expanded.
- `crates/ares-core/src/options/tests.rs`: include the new lookup test module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `small_perimeter_speed` (`coFloatOrPercent`, default `50%`, field at `PrintConfig.hpp:1191`, definition lines 2049-2059)
- `small_perimeter_threshold` (`coFloat`, default `0`, field at `PrintConfig.hpp:1192`, definition lines 2061-2068)
- `wall_sequence` (`coEnum`, default `inner wall/outer wall`, enum at `PrintConfig.hpp:132-137`, field at `PrintConfig.hpp:1209`, enum map lines 277-283, definition lines 2070-2091)
- `wall_direction` (`coEnum`, default `ccw`, enum at `PrintConfig.hpp:140-144`, field at `PrintConfig.hpp:1212`, enum map lines 286-290, definition lines 2100-2110)

Existing adjacent keys remain unchanged:

- `outer_wall_line_width` (`coFloatOrPercent`, default `0`, already registered from `PrintConfig.cpp:2027-2037`)
- `outer_wall_speed` (`coFloat`, default `60`, already registered from `PrintConfig.cpp:2039-2047`)
- `is_infill_first` (`coBool`, default `false`, already registered from `PrintConfig.cpp:2093-2098`)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing `OptionValueKind::FloatOrPercent`, `Float`, and `Enum`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wall order behavior, wall direction path generation, small-perimeter speed behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `extruder`, `extruder_clearance_height_to_rod`, or following options from `PrintConfig.cpp:2112+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M52.
10. Keep modified Rust files under 400 LOC; add a new lookup test file instead of expanding `registry_lookup.rs`.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2027-2110` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Small-perimeter speed application, wall ordering, wall direction path generation, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `extruder` and following options from `PrintConfig.cpp:2112+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all four new keys in a new focused test module.
- Plan/spec explicitly account for deferred upstream UI metadata, wall behavior, small-perimeter speed behavior, slicing/extrusion/G-code behavior, and following extruder scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
