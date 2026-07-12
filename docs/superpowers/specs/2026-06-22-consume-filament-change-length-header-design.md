# Consume Filament Change Length Header Export Design

## Goal

Consume the existing OrcaSlicer `filament_change_length` option as concrete Ares G-code header output instead of leaving it as metadata-only option scaffolding.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1329` declares `filament_change_length` as `ConfigOptionFloats` inside `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2804-2810` defines the option, labels it "Filament ramming length", uses millimeters, sets `min = 0`, and defaults to `ConfigOptionFloats{10}`.
- `OrcaSlicer/src/libslic3r/Config.hpp:845-853` serializes `ConfigOptionFloats` by joining values with commas.
- `OrcaSlicer/src/libslic3r/Config.hpp:910-919` serializes each finite float value and rejects invalid finite state for non-nullable values.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends non-banned full config keys into G-code as `; key = serialized_value`.
- Adjacent upstream behavior exists in `OrcaSlicer/src/libslic3r/Print.cpp:370-375`, `Print.cpp:3123-3132`, and `GCode/WipeTower.cpp:1482`, where `filament_change_length` influences wipe tower/support recalculation and filament change volume.

## Rust Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` so `SliceOptions::filament_config_exports()` also validates and serializes `filament_change_length` as a non-negative float vector export.
- Extend `crates/ares-core/src/gcode_header.rs` so emitted headers include `; filament_change_length = ...` when the option is present.
- Keep early validation through the existing `options.filament_config_exports()?` call in `crates/ares-core/src/gcode.rs`, so invalid values are rejected before BTT thumbnail header skipping.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_change_length` is absent, Ares emits no `; filament_change_length = ...` line.
- If `filament_change_length` is present as a JSON number array, Ares emits a comma-separated header config line:
  - `[10.0]` becomes `; filament_change_length = 10`
  - `[10.0, 2.5, 0.125]` becomes `; filament_change_length = 10,2.5,0.125`
  - `[0.0]` becomes `; filament_change_length = 0`
- Invalid values return `SliceError::InvalidInput` mentioning `filament_change_length`:
  - non-array values
  - arrays containing booleans, strings, objects, or nulls
  - arrays containing negative values
- Invalid `filament_change_length` is rejected before BTT header skipping.
- Existing filament header exports remain behaviorally unchanged:
  - `filament_colour`
  - `default_filament_colour`
  - `filament_ids`
  - `filament_soluble`
  - `filament_printable`

## Deferred Behavior

- Do not implement wipe tower geometry, wipe tower ramming behavior, support regeneration, or filament change volume sizing from `Print.cpp` / `GCode/WipeTower.cpp`.
- Do not implement `required_nozzle_HRC`, `filament_map_mode`, `filament_map`, or adjacent compatibility checks.
- Do not add new crates, dependencies, file I/O, terminal behavior, UI behavior, or Ares-owned pipeline design.

## Architecture

`SliceOptions::filament_config_exports()` remains the single parser/serializer for filament config header values consumed by Ares G-code header output. The new float-vector path mirrors the existing string, bool, and integer export helpers and is scoped only to G-code full-config style header output.

For finite float text in Ares headers, use the existing Ares decimal formatting convention already used by `gcode_header.rs`: trim trailing zeros and join values with commas. This gives stable deterministic output for the profile values covered in this slice while preserving the upstream vector semantics and deferring exact wipe tower behavior.

## Test Strategy

- Add `crates/ares-core/src/tests/filament_change_length_gcode.rs` with focused tests covering single, multiple, zero, absent, invalid, and invalid-with-BTT-skip behavior.
- Run RED with `cargo nextest run -p ares-core filament_change_length_gcode` before implementing runtime support.
- Run GREEN with the same focused nextest command after implementation.
- Run related header export regression tests:
  `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode filament_soluble_gcode filament_printable_gcode filament_change_length_gcode`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, each touched Rust file `<= 400` lines

## Acceptance Criteria

- `filament_change_length` is no longer only metadata scaffolding; valid values appear in generated G-code header comments.
- Invalid `filament_change_length` reaches `SliceError::InvalidInput` whether or not the header text is skipped.
- Existing filament header export tests continue to pass.
- Touched Rust files remain at or below 400 LOC.
- The slice is committed and pushed after independent spec, plan, and implementation reviews return `VERDICT: APPROVE`.
