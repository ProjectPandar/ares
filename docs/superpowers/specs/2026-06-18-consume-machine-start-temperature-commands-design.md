# Consume Machine Start Temperature Commands Design

## Scope

Implement the next concrete Orca rewrite slice for `machine_start_gcode`: generated startup G-code must not duplicate automatic first-layer bed, nozzle, or chamber temperature commands when the user-provided machine start G-code already contains the corresponding temperature M-code.

This consumes existing Ares custom G-code and temperature options. It does not add option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp:528` defines `custom_gcode_sets_temperature(...)`, the helper used to detect custom temperature commands.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3082-3101` processes `machine_start_gcode`, calls automatic bed/nozzle/chamber temperature emission with the processed start G-code as the suppression input, and then writes the machine start G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3969-4000` emits first-layer bed temperature only when custom G-code does not contain `M140` or `M190`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4002-4015` emits first-layer nozzle temperature only when custom G-code does not contain `M104`, `M109`, or RepRapFirmware `G10`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3093-3097` emits chamber startup temperature only when custom G-code does not contain `M141` or `M191`.

## Ares Boundary

- `crates/ares-core/src/gcode.rs` currently emits automatic bed, nozzle, and chamber temperature startup commands before `gcode_start_custom::start_gcode(options)`.
- `crates/ares-core/src/gcode_startup.rs` owns the automatic startup temperature command formatting.
- `crates/ares-core/src/tests/custom_gcode_end.rs`, `bed_temperature_gcode.rs`, `nozzle_temperature_gcode.rs`, and `chamber_temperature_gcode.rs` already verify the adjacent behavior.

## Behavior

- Parse the processed `machine_start_gcode` text before emitting automatic startup temperature commands.
- If processed `machine_start_gcode` contains `M140` or `M190`, suppress the automatic first-layer bed command.
- If processed `machine_start_gcode` contains `M104` or `M109`, suppress the automatic first-layer nozzle command.
- For RepRapFirmware, also suppress the automatic first-layer nozzle command when processed `machine_start_gcode` contains `G10`.
- If processed `machine_start_gcode` contains `M141` or `M191`, suppress the automatic chamber startup command.
- Keep existing Klipper suppression behavior unchanged.
- Keep automatic commands before `machine_start_gcode` when not suppressed, matching current Ares order and the upstream sequencing.
- Use simple line-token detection for these command words. This slice does not implement a full G-code parser.

## Out Of Scope

- No suppression for second-layer temperature transitions.
- No parsing of temperatures from custom G-code into writer state.
- No support for lowercase or expression-generated command names.
- No changes to `machine_end_gcode`, filament start/end G-code, layer custom G-code, CLI, WASM, or option registry metadata.
- No new dependencies.

## Acceptance Criteria

- A `machine_start_gcode` containing `M190 S70` suppresses the automatic startup `M190` while preserving the custom `M190 S70`.
- A `machine_start_gcode` containing `M140 S70` also suppresses automatic startup `M190`.
- A `machine_start_gcode` containing `M104 S215` suppresses automatic startup `M104` while preserving the custom `M104 S215`.
- A RepRapFirmware slice with `machine_start_gcode: "G10 S215"` suppresses automatic startup `G10 S...`.
- A `machine_start_gcode` containing `M191 S45` suppresses automatic startup `M191` while preserving the custom `M191 S45`.
- Existing default startup temperature tests still pass when machine start G-code does not contain temperature commands.
- Verification passes with targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the Rust LOC gate.
