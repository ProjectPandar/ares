# Consume Jerk Options in G-code Design

## Goal

Port the first concrete XY jerk consumption slice from OrcaSlicer into Ares so parsed jerk options affect emitted movement G-code. This is not another option-registration milestone; the output must include `M205 X... Y...` jerk commands before relevant travel and print moves.

## Upstream Boundary

This slice is source-cited to these OrcaSlicer boundaries:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1052-1058`: declares `default_jerk`, `outer_wall_jerk`, `inner_wall_jerk`, `infill_jerk`, `top_surface_jerk`, `initial_layer_jerk`, and `travel_jerk`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1423`: declares `initial_layer_travel_jerk`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169-3248`: defines jerk defaults and non-negative minimums. Defaults are `default_jerk = 0`, role print jerks = `9`, `travel_jerk = 12`, and `initial_layer_travel_jerk = 100%` over `travel_jerk`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6371-6394`: selects print jerk only when `default_jerk > 0`, with first-layer jerk taking precedence, then role jerk, then default jerk.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7297-7343`: selects travel jerk only when `default_jerk > 0`, using first-layer travel jerk on the first layer and `travel_jerk` otherwise.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:255-304`: emits XY jerk through `set_jerk_xy`, suppressing values below `0.01` and unchanged values, and writing ordinary firmware output as `M205 X{jerk} Y{jerk}`.

## Ares Destination Boundary

Implement this slice inside the existing Ares speed/G-code path:

- Parse jerk values from `SliceOptions` into a new kinematics options type.
- Attach selected jerk values to `SpeedMove` alongside feedrate and acceleration.
- Emit `M205 X... Y...` from `GCodeWriter` before the movement command when the selected jerk changes.
- Keep the behavior platform-neutral inside `ares-core`; no filesystem, UI, terminal, OpenGL, or native-only code.

The current Ares `SpeedOptions` and G-code modules are close to the 400 LOC repository limit. This slice must keep every Rust source file at or below 400 LOC by using focused modules/test files instead of expanding large files.

## Included Behavior

### Parsed Options

Parse these Orca option keys:

- `default_jerk`: non-negative number or numeric string, default `0`.
- `outer_wall_jerk`: non-negative number or numeric string, default `9`.
- `inner_wall_jerk`: non-negative number or numeric string, default `9`.
- `infill_jerk`: non-negative number or numeric string, default `9`.
- `initial_layer_jerk`: non-negative number or numeric string, default `9`.
- `travel_jerk`: non-negative number or numeric string, default `12`.
- `initial_layer_travel_jerk`: non-negative number/string or percent over `travel_jerk`, default `100%`.

Invalid negative, non-finite, boolean, object, and malformed percent values must return `SliceError::InvalidInput`.

### Selection Rules

Jerk selection follows the Orca `default_jerk > 0` gate:

- If `default_jerk == 0`, no jerk command is selected for any move.
- For first-layer print moves, use positive `initial_layer_jerk`.
- For non-first-layer print moves:
  - `PrintPathRole::ExternalPerimeter` uses positive `outer_wall_jerk`.
  - `PrintPathRole::InternalPerimeter` uses positive `inner_wall_jerk`.
  - `PrintPathRole::SparseInfill`, `PrintPathRole::Bridge`, and `PrintPathRole::InternalBridge` use positive `infill_jerk`.
  - `Skirt` and `Brim` fall back to `default_jerk`.
  - Any configured role jerk of `0` falls back to `default_jerk`.
- For first-layer travel moves, use positive resolved `initial_layer_travel_jerk`.
- For non-first-layer travel moves, use positive `travel_jerk`.
- Travel jerk values of `0` suppress the travel jerk command; they do not fall back to `default_jerk`.

### G-code Writer Rules

Add ordinary firmware XY jerk output only:

- `Some(jerk)` with `jerk >= 0.01` emits `M205 X{jerk} Y{jerk}\n` using the same 3-digit trimmed formatting as X/Y/F values.
- `None`, `jerk < 0.01`, and unchanged jerk values emit no command.
- When `gcode_comments` is enabled, append `; adjust jerk`.
- Emit jerk after acceleration selection and before the travel/print movement command.

## Deferred Behavior

This slice deliberately does not port:

- Klipper `SET_VELOCITY_LIMIT` and Orca `set_accel_and_jerk`.
- Repetier `M207`.
- Machine max jerk clamping and BBL `Z`/`E` suffixes.
- `top_surface_jerk`, because Ares has no top-surface print role yet.
- Short-travel special cases from `GCode.cpp:7311-7333`.
- Junction deviation and `default_junction_deviation`.
- Option registry metadata additions, unless required by already existing parser tests.

## Acceptance Criteria

- G-code output contains `M205 X... Y...` before moves whose selected jerk changes.
- `default_jerk = 0` suppresses all jerk commands while preserving movement output.
- First-layer print jerk overrides role jerk.
- Non-first-layer external perimeter, internal perimeter, and infill-like roles use their configured jerk values.
- First-layer travel jerk resolves percent values over `travel_jerk`.
- Writer suppresses unchanged jerk and values below `0.01`.
- Jerk comments are emitted only when `gcode_comments` is true.
- Invalid jerk values are rejected at option parsing.
- New tests live in focused files rather than expanding large acceleration tests.
- `cargo fmt --check`, targeted jerk tests, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repository LOC gate pass before commit.
