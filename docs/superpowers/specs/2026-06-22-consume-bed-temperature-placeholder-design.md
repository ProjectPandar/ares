# Consume `bed_temperature` Placeholder Design

## Objective

Port OrcaSlicer's `bed_temperature` start-G-code placeholder into Ares so the selected bed-type other-layer bed-temperature vector can affect emitted G-code through the upstream placeholder name.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2987-2999` resolves the selected bed type, reads `bed_temp_opt` with `get_bed_temp_key((BedType)curr_bed_type)`, and registers `bed_temperature` as `ConfigOptionInts(*bed_temp_opt)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` with the placeholder parser before automatic first-layer bed-temperature G-code suppression.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:466-487` maps `curr_bed_type` to the other-layer bed-temperature option key.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489-1495` defines the bed-type-specific other-layer bed-temperature vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:923-982` defines other-layer bed-temperature defaults and numeric ranges.

## Ares Destination Boundary

- `crates/ares-core/src/options/bed_temperature.rs` adds an `other_layer_bed_temperature_values()` vector accessor and keeps `other_layer_bed_temperature()` as the formula-selected scalar accessor built on top of it.
- `crates/ares-core/src/gcode_placeholders.rs` renders `[bed_temperature]` during `machine_start_gcode` placeholder expansion.
- `crates/ares-core/src/tests/bed_temperature_placeholder_gcode.rs` owns focused regression tests for this upstream placeholder name.
- `crates/ares-core/src/tests/mod.rs` declares the new test module.

## Included Behavior

1. `[bed_temperature]` expands only in `machine_start_gcode`.
2. The rendered value is the selected bed-type other-layer bed-temperature vector formatted as comma-separated integer values with no spaces.
3. Missing selected bed-type other-layer temperature uses the existing Ares fallback behavior: reuse the formula-selected first-layer bed temperature as a single-value vector.
4. Numeric string/list forms accepted by the existing integer-vector parser render as the parsed integer vector.
5. A start G-code line such as `M140 S[bed_temperature]` is expanded before automatic bed-temperature suppression is evaluated, so the rendered `M140` line suppresses the automatic `M190`.
6. Invalid selected other-layer bed-temperature values, invalid `curr_bed_type`, or invalid `bed_temperature_formula` return `SliceError::InvalidInput` through the existing validation path.
7. Layer-change G-code does not expand `[bed_temperature]`.

## Deferred Behavior

- Do not implement `[bed_temperature_initial_layer_vector]`.
- Do not add vector indexing, expression parsing, or a general Orca placeholder-parser rewrite.
- Do not change `bed_temperature_initial_layer`, `first_layer_bed_temperature`, `bed_temperature_initial_layer_single`, automatic bed startup selection, or second-layer temperature transition behavior.
- Do not add option metadata, crates, or dependencies.

## Acceptance Criteria

- New tests fail before implementation when run with `cargo nextest run -p ares-core bed_temperature_placeholder`.
- After implementation, focused tests pass with `cargo nextest run -p ares-core bed_temperature_placeholder`.
- Existing other-layer transition tests pass with `cargo nextest run -p ares-core other_layer_temperature_gcode`.
- Adjacent machine-start bed-temperature tests pass with `cargo nextest run -p ares-core bed_temperature_gcode`.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The slice is limited to one vector accessor refactor, one `machine_start_gcode` placeholder replacement, and focused tests. Rollback is deleting the new accessor, restoring `other_layer_bed_temperature()` to inline parsing, and deleting the placeholder replacement plus test module declaration/file. No file I/O, terminal behavior, UI, OpenGL, platform-specific code, or new dependency is introduced into `ares-core`.
