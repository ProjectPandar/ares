# Consume Filament Printable Header Export Design

## Goal

Consume the existing OrcaSlicer `filament_printable` option as concrete Ares G-code header output instead of leaving it as metadata-only option scaffolding.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1328` declares `filament_printable` as `ConfigOptionInts` inside `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2818-2826` defines the option as a bitmask-style integer vector, labels it "Filament printable", and sets default `ConfigOptionInts{3}`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1023-1031` serializes `ConfigOptionInts` by joining integer values with commas.
- `OrcaSlicer/src/libslic3r/Config.hpp:1070-1077` serializes each non-nil integer with stream integer formatting.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends non-banned full config keys into the G-code as `; key = serialized_value`.

## Rust Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` so `SliceOptions::filament_config_exports()` also validates and serializes `filament_printable` as an integer vector export.
- Extend `crates/ares-core/src/gcode_header.rs` so emitted headers can include `; filament_printable = ...`.
- Keep `crates/ares-core/src/gcode.rs` under the 400 LOC guard by moving filament config export emission ownership into `gcode_header.rs`; `gcode.rs` should still validate `options.filament_config_exports()?` before the BTT thumbnail header skip so invalid values are rejected even when the textual header is suppressed.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_printable` is absent, Ares emits no `; filament_printable = ...` line.
- If `filament_printable` is present as an integer JSON array, Ares emits the serialized integer vector in the G-code header:
  - `[3]` becomes `; filament_printable = 3`
  - `[3, 1]` becomes `; filament_printable = 3,1`
  - `[0]` becomes `; filament_printable = 0`
- Invalid `filament_printable` values return `SliceError::InvalidInput` mentioning `filament_printable`:
  - non-array values
  - arrays containing booleans, strings, floats, or integers outside `i32`
- Invalid `filament_printable` is rejected before BTT header skipping, matching the existing validation posture for other filament config exports.
- Existing filament header exports remain behaviorally unchanged:
  - `filament_colour`
  - `default_filament_colour`
  - `filament_ids`
  - `filament_soluble`

## Deferred Behavior

- Do not implement full extruder compatibility or printable-bitmask decision logic.
- Do not implement UI/profile compatibility reporting.
- Do not implement `filament_change_length`, `required_nozzle_HRC`, `filament_map_mode`, or `filament_map`.
- Do not add new crates, dependencies, file I/O, terminal behavior, UI behavior, or Ares-owned pipeline design.

## Architecture

`SliceOptions::filament_config_exports()` remains the single parser/serializer for filament config header values. `gcode.rs` invokes it early for validation, while `gcode_header.rs` invokes it when a header is actually emitted and appends present exports as `; key = value` lines. This keeps `ares-core` platform-neutral and avoids growing the already 400-line `gcode.rs` for each additional source-cited header config export.

Integer vector serialization should mirror upstream `ConfigOptionInts`: comma-separated signed integer text with no spaces. Ares stores parsed JSON numbers as Rust `i32` values for this export, because Orca's source type is `ConfigOptionInts`.

## Test Strategy

- Add `crates/ares-core/src/tests/filament_printable_gcode.rs` with focused tests covering single, multiple, zero, absent, invalid, and invalid-with-BTT-skip behavior.
- Run RED with `cargo nextest run -p ares-core filament_printable_gcode` before implementing runtime support.
- Run GREEN with the same focused nextest command after implementation.
- Run related header export regression tests:
  `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode filament_soluble_gcode filament_printable_gcode`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, each touched Rust file `<= 400` lines

## Acceptance Criteria

- `filament_printable` is no longer only metadata scaffolding; valid values appear in generated G-code header comments.
- Invalid `filament_printable` reaches `SliceError::InvalidInput` whether or not the header text is skipped.
- Existing filament header export tests continue to pass.
- `gcode.rs` remains at or below 400 LOC.
- The slice is committed and pushed after independent spec, plan, and implementation reviews return `VERDICT: APPROVE`.
