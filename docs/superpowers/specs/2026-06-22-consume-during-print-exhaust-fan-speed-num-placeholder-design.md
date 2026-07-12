# Consume During Print Exhaust Fan Speed Num Placeholder Design

## Objective

Port OrcaSlicer's `during_print_exhaust_fan_speed_num` start-G-code placeholder into Ares so the existing `during_print_exhaust_fan_speed` option can affect user-authored `machine_start_gcode` templates as a 0-255 PWM vector, not only Ares' automatic exhaust fan command.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:3021-3026` builds `during_print_exhaust_fan_speed_num` by converting each `m_config.during_print_exhaust_fan_speed` percent value with `(int)(item / 100.0 * 255)` and registers it as `ConfigOptionInts`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3079-3085` processes `machine_start_gcode` with the placeholder parser before automatic startup-command emission.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1525` declares `during_print_exhaust_fan_speed` as a `ConfigOptionInts` print config field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1819-1826` defines `during_print_exhaust_fan_speed` as a percent integer vector with default `60` and range `0..=100`.

## Ares Destination Boundary

- `crates/ares-core/src/options/exhaust_fan.rs` adds a reusable accessor for parsed `during_print_exhaust_fan_speed` values converted to Orca's 0-255 placeholder integers.
- `crates/ares-core/src/gcode_placeholders.rs` renders `[during_print_exhaust_fan_speed_num]` during `machine_start_gcode` placeholder expansion.
- `crates/ares-core/src/tests/during_print_exhaust_fan_speed_num_placeholder_gcode.rs` owns focused placeholder behavior tests.
- `crates/ares-core/src/tests/mod.rs` declares the new test module.

## Included Behavior

1. `[during_print_exhaust_fan_speed_num]` expands only in `machine_start_gcode`.
2. The placeholder renders the parsed `during_print_exhaust_fan_speed` integer vector as comma-separated PWM integers with no spaces.
3. Conversion uses Orca's truncating formula: `percent / 100.0 * 255`, cast down to an integer. For example, `60` renders `153`, `80` renders `204`, and `100` renders `255`.
4. Missing `during_print_exhaust_fan_speed` uses Orca's default vector `[60]`, so the placeholder renders `153`.
5. Numeric scalar, numeric list, and existing numeric string vector forms accepted by Ares' integer-vector parser render as parsed values.
6. Placeholder rendering is independent of `support_air_filtration`, `activate_air_filtration`, and `activate_air_filtration_during_print`; those flags still control automatic exhaust fan commands only.
7. A `machine_start_gcode` line that uses the placeholder renders inside the existing Ares start sequence without changing automatic exhaust fan ordering. If automatic during-print exhaust fan startup is enabled, the existing `M106 P3` startup command still appears before the rendered `machine_start_gcode` line in this slice.
8. Invalid values outside `0..=100` or malformed vector values return `SliceError::InvalidInput` through the existing validation path.
9. Layer-change G-code does not expand this placeholder.

## Deferred Behavior

- Do not change automatic during-print or completion exhaust fan command behavior.
- Do not migrate Ares' start-sequence ordering toward Orca's later `GCode.cpp:3144-3155` exhaust fan placement in this placeholder slice.
- Do not add `complete_print_exhaust_fan_speed_num`; this slice ports only the upstream start placeholder that Orca registers.
- Do not add brace-expression parsing, vector indexing, or a general Orca placeholder-parser rewrite.
- Do not change `support_air_filtration`, `activate_air_filtration`, `activate_air_filtration_during_print`, or `activate_air_filtration_on_completion` parsing.
- Do not add option metadata, crates, dependencies, filesystem behavior, UI behavior, or platform-specific code.

## Acceptance Criteria

- New tests fail before implementation with `cargo nextest run -p ares-core during_print_exhaust_fan_speed_num_placeholder`.
- After implementation, focused tests pass with `cargo nextest run -p ares-core during_print_exhaust_fan_speed_num_placeholder`.
- Existing exhaust fan behavior tests pass with `cargo nextest run -p ares-core exhaust_fan_gcode`.
- Focused tests assert the current Ares ordering when automatic during-print exhaust fan startup is enabled: `M106 P3 S...` remains before the rendered `machine_start_gcode` placeholder line.
- Full verification passes with:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
- Touched Rust files remain at or below 400 LOC.

## Safety And Rollback

The slice is limited to one options accessor, one `machine_start_gcode` placeholder replacement, and focused tests. Rollback is deleting the new accessor, deleting the placeholder replacement, and deleting the new test module declaration/file. No file I/O, terminal behavior, UI, OpenGL, platform-specific code, or new dependency is introduced into `ares-core`.
