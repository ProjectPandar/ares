# Consume Chamber Temperature Placeholders Design

## Objective

Port OrcaSlicer's `chamber_temperature` and `overall_chamber_temperature` start-G-code placeholders into Ares so existing chamber-temperature options can affect user-authored `machine_start_gcode` templates, not only automatic `M191` startup and `M141` shutdown commands.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:2979-2984` iterates writer extruders, ORs `activate_chamber_temp_control`, and computes `max_chamber_temp` from `m_config.chamber_temperature`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2999-3000` registers `chamber_temperature` as `ConfigOptionInts(m_config.chamber_temperature)` and `overall_chamber_temperature` as `ConfigOptionInt(max_chamber_temp)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` with the placeholder parser before automatic startup-command suppression.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1637` declares `chamber_temperature` as a `ConfigOptionInts` print config field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6457-6476` defines the `chamber_temperature` option metadata, integer vector type, default `0`, and range.

## Ares Destination Boundary

- `crates/ares-core/src/options/chamber_temperature.rs` adds reusable vector and scalar accessors for `chamber_temperature`.
- `crates/ares-core/src/gcode_placeholders.rs` renders `[chamber_temperature]` and `[overall_chamber_temperature]` during `machine_start_gcode` placeholder expansion.
- `crates/ares-core/src/tests/chamber_temperature_placeholder_gcode.rs` owns focused placeholder behavior tests.
- `crates/ares-core/src/tests/mod.rs` declares the new test module.

## Included Behavior

1. `[chamber_temperature]` expands only in `machine_start_gcode`.
2. `[chamber_temperature]` renders the parsed chamber-temperature integer vector as comma-separated integers with no spaces.
3. `[overall_chamber_temperature]` expands only in `machine_start_gcode`.
4. `[overall_chamber_temperature]` renders the maximum value from the parsed chamber-temperature vector.
5. Missing `chamber_temperature` uses the existing Ares default vector `[0]`, so both placeholders render `0`.
6. Numeric string/list forms accepted by the existing integer-vector parser render as parsed integer values.
7. Placeholder rendering is independent of `activate_chamber_temp_control`; activation still controls automatic chamber startup and shutdown commands only.
8. A `machine_start_gcode` line such as `M191 S[overall_chamber_temperature]` expands before automatic chamber-startup suppression is evaluated, so the rendered `M191` line suppresses the automatic startup `M191`.
9. Invalid `chamber_temperature` values return `SliceError::InvalidInput` through the existing validation path.
10. Layer-change G-code does not expand either placeholder.

## Deferred Behavior

- Do not change automatic chamber startup or shutdown command behavior.
- Do not add chamber-temperature expression parsing, vector indexing, or a general Orca placeholder-parser rewrite.
- Do not change `activate_chamber_temp_control` parsing.
- Do not add option metadata, crates, dependencies, filesystem behavior, UI behavior, or platform-specific code.

## Acceptance Criteria

- New tests fail before implementation with `cargo nextest run -p ares-core chamber_temperature_placeholder`.
- After implementation, focused tests pass with `cargo nextest run -p ares-core chamber_temperature_placeholder`.
- Existing chamber behavior tests pass with `cargo nextest run -p ares-core chamber_temperature_gcode`.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The slice is limited to one options accessor refactor, two `machine_start_gcode` placeholder replacements, and focused tests. Rollback is deleting the new accessors, restoring `chamber_temperature_control()` to inline parsing, and deleting the placeholder replacements plus test module declaration/file. No file I/O, terminal behavior, UI, OpenGL, platform-specific code, or new dependency is introduced into `ares-core`.
