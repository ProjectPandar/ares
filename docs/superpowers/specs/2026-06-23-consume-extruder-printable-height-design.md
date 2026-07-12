# Consume extruder printable height design

## Source boundary

This slice ports the first executable part of OrcaSlicer's `extruder_printable_height` height-limit path into Ares:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1540` owns the FDM `PrintConfig` tuple `((ConfigOptionFloatsNullable, extruder_printable_height))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:766-773` defines `extruder_printable_height` as non-negative millimeters, max `1000`, default `ConfigOptionFloatsNullable{0}`.
- `OrcaSlicer/src/libslic3r/Print.cpp:3050-3054` exposes configured per-extruder printable heights.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:307-315` and `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:1923-1929` treat generated layers or G-code paths above an extruder's configured printable height as unprintable.

The Rust destination is the existing `ares-core` layer-height validation boundary in `crates/ares-core/src/printable_height.rs`, called from `run_slicing_pipeline` immediately after `plan_layers`.

## Included behavior

- Parse `extruder_printable_height` at the same pre-G-code validation stage as `printable_height`.
- Accept the current Ares external option shapes for the first extruder: scalar number, numeric string, array number, array numeric string, and nullable/zero default forms.
- Treat missing, empty, `null`, or first value `0` as no per-extruder override, matching Orca's default nullable zero and preserving the global `printable_height` limit.
- Reject non-finite, negative, non-numeric, or unsupported first values with `SliceError::InvalidInput` that names `extruder_printable_height`.
- In the current Ares single-active-extruder path, reject a planned layer max `print_z` above the effective first-extruder height before slicing/G-code output.
- If both limits are present, enforce the stricter of global `printable_height` and first `extruder_printable_height`.
- Accept equality at the limit with the same epsilon behavior as existing `printable_height`.

## Deferred behavior

- Full Orca multi-extruder geometric unprintable-filament detection remains deferred because Ares does not yet assign generated walls/solid infill/sparse infill to separate extruder IDs in the same way as `PrintObject::detect_extruder_geometric_unprintables`.
- Per-filament map limiting, `limit_filament_maps`, print area error payloads, localized object-specific diagnostics, and UI/reporting semantics remain deferred.
- `extruder_printable_area`, `BuildVolume` 3D per-extruder volumes, timelapse liftable-extruder selection, SLA printable height, and object placement remain deferred.
- This slice does not add new option metadata and does not change generated moves when the configured height is sufficient.

## Acceptance criteria

- A focused RED run with `cargo nextest run -p ares-core extruder_printable_height` fails before implementation because planned layers above `extruder_printable_height[0]` are still accepted.
- After implementation, the focused nextest command passes.
- Existing `printable_height` tests still pass and demonstrate global-height behavior is preserved.
- Public `slice(...)` rejects an over-height first-extruder limit before G-code output.
- Full verification passes with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and touched Rust file LOC checks.

## Safety and rollback

The change is isolated to in-memory `ares-core` option parsing and layer validation. It introduces no filesystem, terminal, UI, OpenGL, or native-only behavior. Rollback is a small revert of the new tests, parser/validation edits, and roadmap note.
