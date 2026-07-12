# M50 Spec: PrintConfig G-code and shell pattern option registry slice

## Goal
Port the adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` G-code and shell-pattern option-definition slice into `ares-core` option registry metadata, covering end-G-code strings, object-by-object G-code, vertical shell thickness policy, and top/bottom/internal solid infill pattern keys without changing G-code execution, object scheduling, infill generation, slicing, extrusion, or output behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-98`: `InfillPattern` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:223-228`: `EnsureVerticalShellThickness` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1087`: `ensure_vertical_shell_thickness` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1090-1092`: `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1294-1300`: `before_layer_change_gcode`, `printing_by_object_gcode`, `machine_end_gcode`, and `filament_end_gcode` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-255`: `InfillPattern` enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:368-374`: `EnsureVerticalShellThickness` enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1940-2025`: option definitions for this slice.

Related upstream behavior explicitly deferred:

- Custom G-code insertion/execution, object-by-object scheduling, and output command changes.
- Vertical shell thickness calculations and solid infill generation.
- Top/bottom/internal solid infill pattern generation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027+`: `outer_wall_line_width`, `outer_wall_speed`, and following options.
- UI labels, enum labels, mode behavior, multiline/full-width/height metadata, preset/profile behavior, filesystem/network integrations, slicing, extrusion, and G-code behavior.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definitions for keys that sort before `is_infill_first`, plus removal of existing `is_infill_first`, `layer_height`, and `line_width` definitions so the file remains under 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definitions for existing moved `is_infill_first`, `layer_height`, and `line_width`, followed by `machine_end_gcode`, existing `max_*`, `printing_by_object_gcode`, `top_surface_pattern`, and any needed late entries.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/profile.rs`: G-code string metadata assertions.
- `crates/ares-core/src/options/registry/tests/metadata/quality.rs`: vertical shell and shell-pattern metadata assertions.
- `crates/ares-core/src/options/tests/registry_helpers.rs` and `registry_lookup.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `machine_end_gcode` (`coString`, default `M104 S0 ; turn off temperature\nG28 X0  ; home X axis\nM84     ; disable motors\n`, field at `PrintConfig.hpp:1299`, lines 1940-1947)
- `printing_by_object_gcode` (`coString`, default empty string, field at `PrintConfig.hpp:1295`, lines 1949-1956)
- `filament_end_gcode` (`coStrings`, default single-space string, field at `PrintConfig.hpp:1300`, lines 1958-1965)
- `ensure_vertical_shell_thickness` (`coEnum`, default `ensure_all`, enum at `PrintConfig.hpp:223-228`, field at `PrintConfig.hpp:1087`, enum map lines 368-374, definition lines 1967-1984)
- `top_surface_pattern` (`coEnum`, default `monotonicline`, enum at `PrintConfig.hpp:87-98`, field at `PrintConfig.hpp:1090`, enum map lines 225-255, definition lines 1986-2007)
- `bottom_surface_pattern` (`coEnum`, default `monotonic`, enum at `PrintConfig.hpp:87-98`, field at `PrintConfig.hpp:1091`, enum map lines 225-255, definition lines 2009-2016)
- `internal_solid_infill_pattern` (`coEnum`, default `monotonic`, enum at `PrintConfig.hpp:87-98`, field at `PrintConfig.hpp:1092`, enum map lines 225-255, definition lines 2018-2025)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::String`, `Strings`, and `Enum`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, custom G-code execution, object-by-object scheduling, vertical shell behavior, top/bottom/internal solid infill pattern generation, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `outer_wall_line_width`, `outer_wall_speed`, or following options from `PrintConfig.cpp:2027+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M51, or verify those docs if the rename already exists in the current worktree.
10. Keep modified Rust files under 400 LOC by moving existing `is_infill_first`, `layer_height`, and `line_width` definitions from `early.rs` to the start of `late.rs`, preserving exact key/kind/default/source. This is a shard-only move with no public metadata change.
11. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/enum-label/multiline/full-width/height/mode metadata from `PrintConfig.cpp:1940-2025` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Custom G-code execution, object-by-object scheduling, vertical shell generation, shell-pattern generation, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `outer_wall_line_width`, `outer_wall_speed`, and following options from `PrintConfig.cpp:2027+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all seven new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all seven new keys.
- Plan/spec explicitly account for deferred upstream UI metadata, custom G-code execution, object scheduling, vertical shell behavior, pattern generation, slicing/extrusion/G-code behavior, and following outer-wall scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
