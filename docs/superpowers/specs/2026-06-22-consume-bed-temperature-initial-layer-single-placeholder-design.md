# Consume `bed_temperature_initial_layer_single` Placeholder Design

## Objective

Port OrcaSlicer's `bed_temperature_initial_layer_single` start-G-code placeholder into Ares so an existing bed-temperature option affects emitted G-code instead of remaining option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2987-3000` computes `target_bed_temp` from `bed_temperature_formula`, then registers `bed_temperature_initial_layer_single` as `ConfigOptionInt(target_bed_temp)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` with the placeholder parser before deciding whether to emit automatic first-layer bed-temperature G-code.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:489-509` maps `curr_bed_type` to the first-layer bed-temperature option key.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489-1501` defines the bed-type-specific first-layer bed-temperature vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041` defines first-layer bed-temperature defaults and numeric ranges.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2503-2510` defines `bed_temperature_formula` values `by_first_filament` and `by_highest_temp`.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_placeholders.rs` renders `[bed_temperature_initial_layer_single]` during `machine_start_gcode` placeholder expansion.
- `crates/ares-core/src/options/bed_temperature.rs` remains the owning parser/validator for first-layer bed-temperature options; this slice must reuse the existing formula-selected accessor instead of adding a second parser.
- `crates/ares-core/src/tests/bed_temperature_gcode.rs` owns regression tests for bed-temperature startup G-code and this placeholder.

## Included Behavior

1. `[bed_temperature_initial_layer_single]` expands only in `machine_start_gcode`.
2. The rendered value is the formula-selected scalar target bed temperature:
   - default `by_highest_temp` uses the highest value in the selected bed-type first-layer vector,
   - `by_first_filament` uses the first value in the selected bed-type first-layer vector,
   - a missing selected bed-type first-layer option uses the existing Orca-derived Ares default for that bed type.
3. The value renders as an integer string with no decimal suffix.
4. A selected value of `0` renders as `0`.
5. Invalid selected bed-temperature values, invalid `curr_bed_type`, or invalid `bed_temperature_formula` return `SliceError::InvalidInput` through the existing option validation path.
6. A start G-code line such as `M140 S[bed_temperature_initial_layer_single]` is expanded before automatic bed-temperature suppression is evaluated, so the rendered `M140` line suppresses the automatic `M190`.
7. Layer-change G-code does not expand `[bed_temperature_initial_layer_single]`.

## Deferred Behavior

- Do not implement `[bed_temperature_initial_layer]`, `[bed_temperature]`, or `[bed_temperature_initial_layer_vector]`.
- Do not add vector indexing, expression parsing, or a general Orca placeholder-parser rewrite.
- Do not change automatic first-layer bed-temperature emission outside the existing suppression interaction.
- Do not add new option metadata or dependencies.

## Acceptance Criteria

- New tests fail before implementation when run with `cargo nextest run -p ares-core bed_temperature_initial_layer_single`.
- After implementation, focused tests pass with `cargo nextest run -p ares-core bed_temperature_initial_layer_single`.
- Adjacent bed-temperature placeholder and startup tests pass with `cargo nextest run -p ares-core bed_temperature_gcode`.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The slice is limited to start-G-code placeholder rendering and tests. Rollback is deleting the added placeholder replacement and tests. No file I/O, terminal behavior, UI, OpenGL, or platform-specific code is introduced into `ares-core`.
