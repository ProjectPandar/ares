# Consume Filament Is Support Header Export Design

## Goal

Consume the existing OrcaSlicer `filament_is_support` option as concrete Ares G-code header output instead of leaving it as partial internal display-only behavior.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1327` declares `filament_is_support` as `ConfigOptionBools` inside `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2812-2816` defines the option, labels it "Support material", and defaults it to `ConfigOptionBools { false }`.
- `OrcaSlicer/src/libslic3r/Config.hpp:1894-1903` serializes `ConfigOptionBools` by joining values with commas.
- `OrcaSlicer/src/libslic3r/Config.hpp:1951-1958` serializes each non-null bool as `1` or `0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5575` appends non-banned full config keys into G-code as `; key = serialized_value`.
- Adjacent upstream behavior exists in `OrcaSlicer/src/libslic3r/GCode.cpp:4807-4814`, `GCode/WipeTower.cpp:1549`, and `GCode/ToolOrdering.cpp`, where `filament_is_support` influences support material handling and tool ordering.

## Rust Destination Boundary

- Extend `crates/ares-core/src/options/filament_type.rs` so `SliceOptions::filament_config_exports()` also validates and serializes `filament_is_support` as a bool vector export.
- Extend `crates/ares-core/src/gcode_header.rs` so emitted headers include `; filament_is_support = ...` when the option is present.
- Keep early validation through the existing `options.filament_config_exports()?` call in `crates/ares-core/src/gcode.rs`, so invalid values are rejected before BTT thumbnail header skipping.
- Keep existing `SliceOptions::filament_type_display()` support-material display behavior unchanged.
- Add focused async G-code tests under `crates/ares-core/src/tests/`.

## Included Behavior

- If `filament_is_support` is absent, Ares emits no `; filament_is_support = ...` line.
- If `filament_is_support` is present as a JSON boolean array, Ares emits a comma-separated header config line:
  - `[true]` becomes `; filament_is_support = 1`
  - `[true, false]` becomes `; filament_is_support = 1,0`
  - `[false]` becomes `; filament_is_support = 0`
- Invalid values return `SliceError::InvalidInput` mentioning `filament_is_support`:
  - non-array values
  - arrays containing numbers, strings, objects, or nulls
- Invalid `filament_is_support` is rejected before BTT header skipping.
- Existing filament header exports remain behaviorally unchanged:
  - `filament_colour`
  - `default_filament_colour`
  - `filament_ids`
  - `filament_soluble`
  - `filament_printable`
  - `filament_change_length`

## Deferred Behavior

- Do not implement support-material tool ordering, wipe tower support-material flags, support generation, or support interface behavior from `GCode.cpp`, `GCode/WipeTower.cpp`, or `GCode/ToolOrdering.cpp`.
- Do not change the current `filament_type_display()` behavior beyond preserving its validation and fallback behavior.
- Do not implement `required_nozzle_HRC`, `filament_map_mode`, `filament_map`, or adjacent compatibility checks.
- Do not add new crates, dependencies, file I/O, terminal behavior, UI behavior, or Ares-owned pipeline design.

## Architecture

`SliceOptions::filament_config_exports()` remains the single parser/serializer for filament config header values consumed by Ares G-code header output. The new `filament_is_support` path reuses the existing bool-vector export helper that already serializes `filament_soluble` as upstream-style `1` and `0` values.

The implementation appends a new header export beside the other filament config comments. The current early validation call in `gcode.rs` remains the pre-header-skip validation gate, so invalid values fail even when BTT thumbnails suppress the normal header text.

## Test Strategy

- Add `crates/ares-core/src/tests/filament_is_support_gcode.rs` with focused tests covering single true, multiple values, false, absent, invalid, display-preservation, and invalid-with-BTT-skip behavior.
- Run RED with `cargo nextest run -p ares-core filament_is_support_gcode` before implementing header export support.
- Run GREEN with the same focused nextest command after implementation.
- Run related header export regression tests:
  `cargo nextest run -p ares-core filament_colour_gcode default_filament_colour_gcode filament_ids_gcode filament_soluble_gcode filament_printable_gcode filament_change_length_gcode filament_is_support_gcode`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, each touched Rust file `<= 400` lines

## Acceptance Criteria

- `filament_is_support` valid values appear in generated G-code header comments.
- Invalid `filament_is_support` reaches `SliceError::InvalidInput` whether or not the header text is skipped.
- Existing support-material display behavior still maps support PLA to `Sup.PLA`.
- Existing filament header export tests continue to pass.
- Touched Rust files remain at or below 400 LOC.
- The slice is committed and pushed after independent spec, plan, and implementation reviews return `VERDICT: APPROVE`.
