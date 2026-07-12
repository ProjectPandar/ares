# M187 Spec: PrintConfig normalize_fdm spiral mode normalization

## Goal
Port OrcaSlicer's `spiral_mode` branch from `DynamicPrintConfig::normalize_fdm(int used_filaments)` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355-8369`: `spiral_mode` branch in `DynamicPrintConfig::normalize_fdm`, including `retract_when_changing_layer`, `filament_retract_when_changing_layer`, `wall_loops`, `alternate_extra_wall`, `top_shell_layers`, and `sparse_infill_density` mutations.

Context anchors:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5678-5684`: `spiral_mode` is a boolean defaulting to false.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5062-5067`: `retract_when_changing_layer` is a bool vector defaulting to `[false]`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8358-8362`: `filament_retract_when_changing_layer` is handled as `ConfigOptionBoolsNullable`, so existing nullable entries are accepted for length preservation before being overwritten with `false`.
- Existing Ares option metadata for the affected keys remains the source-cited option registry boundary already ported in earlier milestones.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8372+`: resolution clamp, prime tower, independent support layer height, filament-count behavior, and all later `normalize_fdm` branches.
- CLI spiral-vase validation in `PrintConfig.cpp:10208+`.
- Object arrangement, variant expansion, silent-mode behavior, typed option accessors beyond this API, option parsing changes outside this API, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, UI runtime behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/fdm_normalization.rs`: extend `SliceOptions::normalize_fdm(&mut self, used_filaments: usize) -> Result<(), SliceError>` with the spiral-mode branch and private bool/vector helpers.
- `crates/ares-core/src/options/tests/fdm_normalization.rs`: extend source-behavior tests.
- `docs/roadmap.md` and `docs/milestones/m187-print-config-normalize-fdm-spiral-mode.md`: milestone sequencing docs.

## Functional requirements

1. Keep the M186 `normalize_fdm` behavior unchanged.
2. If `spiral_mode` is absent, do not run this branch.
3. If `spiral_mode` is present and false, do not run this branch.
4. If `spiral_mode` is present and true, set `retract_when_changing_layer` to an array of `false` values.
5. If `retract_when_changing_layer` already exists as a bool array, preserve its length while setting every entry to `false`.
6. If `retract_when_changing_layer` is absent, insert `[false]`, matching the upstream default vector size.
7. If `spiral_mode` is true, set `filament_retract_when_changing_layer` to an array of `false` values.
8. If `filament_retract_when_changing_layer` already exists as an array containing boolean or null values, preserve its length while setting every entry to `false`.
9. If `filament_retract_when_changing_layer` is absent, insert `[false]` as the Ares default shell for the upstream created option.
10. If `spiral_mode` is true, set `wall_loops = 1`, `alternate_extra_wall = false`, `top_shell_layers = 0`, and `sparse_infill_density = 0` regardless of existing values.
11. Reject non-boolean `spiral_mode` values with `SliceError::InvalidInput`.
12. Reject non-array retraction values, non-boolean entries in `retract_when_changing_layer`, and entries other than boolean/null in `filament_retract_when_changing_layer` with `SliceError::InvalidInput`.
13. Do not add automatic deserialization normalization; callers must explicitly invoke `normalize_fdm`.
14. Do not add `PrintConfig.cpp:8372+` behavior, slicing, extrusion, G-code behavior, UI runtime behavior, new crates, or dependencies.
15. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent and false `spiral_mode` do not mutate spiral-specific keys.
- Tests prove true `spiral_mode` forces `wall_loops`, `alternate_extra_wall`, `top_shell_layers`, and `sparse_infill_density` values.
- Tests prove true `spiral_mode` sets existing `retract_when_changing_layer` bool arrays and nullable `filament_retract_when_changing_layer` arrays to all false while preserving lengths.
- Tests prove true `spiral_mode` inserts default `[false]` arrays when retraction arrays are missing.
- Tests prove invalid `spiral_mode` and invalid retraction-array values return `SliceError::InvalidInput` and do not panic.
- Tests prove M186 extruder role propagation still happens before/alongside the spiral branch.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8372+` runtime normalization branches.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
