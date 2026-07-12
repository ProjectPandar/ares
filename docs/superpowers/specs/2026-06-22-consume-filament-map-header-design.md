# Consume Filament Map Header Export Design

## Goal

Consume the existing OrcaSlicer `filament_map` option scaffold into observable Ares G-code output by exporting it in the generated G-code config header.

This is a concrete behavior slice, not a new option-metadata milestone. The Rust change must use the already-known option key and make slicer output reflect user-provided filament-to-extruder map values.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1336` declares `filament_map` as a `ConfigOptionInts` member of `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2401-2405` defines the option with label "Filament map to extruder", tooltip text, developer mode, and default `ConfigOptionInts{1}`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1023-1031` serializes `ConfigOptionInts` as comma-separated integer values.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends every non-banned full config key as `; key = serialized_value` in G-code output.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3518-3578` also uses `filament_map` as a 1-based nozzle map for processor result filament/nozzle assignment bookkeeping; that runtime assignment behavior is adjacent but outside this header-export slice.

## Rust Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` because it already owns filament-related full-config header exports and existing `ConfigOptionInts`-style serialization.
- Extend `crates/ares-core/src/gcode_header.rs` so `format_header` appends `; filament_map = ...` when the option is present and valid.
- Add focused async slicing tests under `crates/ares-core/src/tests/filament_map_gcode.rs`.
- Register the test module in `crates/ares-core/src/tests/mod.rs`.

No new crate, dependency, public API, CLI behavior, filesystem behavior, UI behavior, OpenGL behavior, or independent Ares pipeline design is allowed.

## Included Behavior

- If `filament_map` is present as a JSON integer array, Ares emits one config header line:
  - `[1]` becomes `; filament_map = 1`
  - `[2, 1, 3]` becomes `; filament_map = 2,1,3`
  - `[0, -1]` becomes `; filament_map = 0,-1`, because the cited option definition does not declare a min/max constraint.
- If the option is absent, Ares does not emit a `filament_map` config export line.
- Invalid shapes or non-integer values return `SliceError::InvalidInput` mentioning `filament_map`.
- Validation must still happen before BTT thumbnail header skipping can hide invalid input, matching the existing filament config export validation pattern.

## Deferred Behavior

- Do not implement Orca `GCode::export_layer_filaments` processor result behavior, filament change counting, nozzle change sequence, or `optimal_assignment`.
- Do not implement multi-nozzle tool selection, toolchange G-code, filament remapping, or extrusion routing from `filament_map`.
- Do not modify `filament_map_mode`, `filament_extruder_variant`, `physical_extruder_map`, `support_object_skip_flush`, or `nozzle_flush_dataset`.
- Do not broaden generic option export beyond this one source-cited key.

## Acceptance Criteria

- A focused RED run of `cargo nextest run -p ares-core filament_map_gcode` fails before implementation because the header line and invalid-value checks are missing.
- After implementation, `cargo nextest run -p ares-core filament_map_gcode` passes.
- Existing adjacent filament header export tests still pass with a targeted nextest command.
- Full verification before commit uses:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, with every touched Rust file at or below 400 LOC
- Independent spec, plan, and implementation reviewers must return `VERDICT: APPROVE`.

## Risks

- `filament_map` values are 1-based when Orca uses them for nozzle assignment, but the cited `PrintConfig.cpp` option definition does not enforce a range. This slice must avoid adding stricter assignment validation until the assignment behavior itself is ported.
- `filament_map` already appears in profile composition and print-apply staging. This slice must not change that existing behavior; it only adds G-code config header consumption.
