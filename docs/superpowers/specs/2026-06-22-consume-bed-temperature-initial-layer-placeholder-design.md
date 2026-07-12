# Consume `bed_temperature_initial_layer` Placeholder Design

## Objective

Port OrcaSlicer's `bed_temperature_initial_layer` start-G-code placeholder into Ares so the selected first-layer bed-temperature vector can affect emitted G-code through the upstream placeholder name, not only through the SoftFever alias `first_layer_bed_temperature`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2987-3000` resolves the selected bed type, reads `first_bed_temp_opt` with `get_bed_temp_1st_layer_key((BedType)curr_bed_type)`, and registers `bed_temperature_initial_layer` as `ConfigOptionInts(*first_bed_temp_opt)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3006-3008` registers `first_layer_bed_temperature` as a later alias for the same `first_bed_temp_opt`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` with the placeholder parser before automatic first-layer bed-temperature G-code suppression.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:489-509` maps `curr_bed_type` to the first-layer bed-temperature option key.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489-1501` defines the bed-type-specific first-layer bed-temperature vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041` defines first-layer bed-temperature defaults and numeric ranges.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_placeholders.rs` renders `[bed_temperature_initial_layer]` during `machine_start_gcode` placeholder expansion.
- `crates/ares-core/src/options/bed_temperature.rs` remains the owning parser/validator for first-layer bed-temperature vectors; this slice must reuse `SliceOptions::first_layer_bed_temperature_values()`.
- `crates/ares-core/src/tests/bed_temperature_initial_layer_placeholder_gcode.rs` owns focused regression tests for this upstream placeholder name.
- `crates/ares-core/src/tests/mod.rs` declares the new test module.

## Included Behavior

1. `[bed_temperature_initial_layer]` expands only in `machine_start_gcode`.
2. The rendered value is the selected bed-type first-layer bed-temperature vector formatted the same way as existing vector placeholders: comma-separated integer values with no spaces.
3. Missing selected bed-type first-layer temperature uses the existing Orca-derived Ares default for that bed type.
4. Numeric string/list forms accepted by `first_layer_bed_temperature_values()` render as the parsed integer vector.
5. A start G-code line such as `M140 S[bed_temperature_initial_layer]` is expanded before automatic bed-temperature suppression is evaluated, so the rendered `M140` line suppresses the automatic `M190`.
6. Invalid selected bed-temperature values or invalid `curr_bed_type` return `SliceError::InvalidInput` through the existing option validation path.
7. Layer-change G-code does not expand `[bed_temperature_initial_layer]`.

## Deferred Behavior

- Do not implement `[bed_temperature]` or `[bed_temperature_initial_layer_vector]`.
- Do not add vector indexing, expression parsing, or a general Orca placeholder-parser rewrite.
- Do not change `first_layer_bed_temperature`, `bed_temperature_initial_layer_single`, automatic bed startup selection, or second-layer temperature transition behavior.
- Do not add option metadata, crates, or dependencies.

## Acceptance Criteria

- New tests fail before implementation when run with `cargo nextest run -p ares-core bed_temperature_initial_layer_placeholder`.
- After implementation, focused tests pass with `cargo nextest run -p ares-core bed_temperature_initial_layer_placeholder`.
- Adjacent bed-temperature tests pass with `cargo nextest run -p ares-core bed_temperature_gcode`.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The slice is limited to one `machine_start_gcode` placeholder replacement plus focused tests. Rollback is deleting the added replacement and test module declaration/file. No file I/O, terminal behavior, UI, OpenGL, platform-specific code, or new dependency is introduced into `ares-core`.
