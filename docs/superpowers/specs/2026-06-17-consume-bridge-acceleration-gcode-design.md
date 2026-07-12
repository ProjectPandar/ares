# Consume Bridge Acceleration In G-code Design

## Context

Ares already registers and wires bridge speed and flow options, and it now emits `M204` acceleration changes from concrete print/travel move kinematics. The next slice must consume an existing acceleration option in generated G-code instead of adding more option metadata.

This slice ports the bridge acceleration precedence from OrcaSlicer:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1046-1048` declares `initial_layer_acceleration`, `bridge_acceleration`, and `travel_acceleration` in the print config option set.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3104-3112` registers `bridge_acceleration` as `coFloatOrPercent`, minimum `0`, ratio-over `outer_wall_acceleration`, and default `50%`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6347-6355` chooses `initial_layer_acceleration` first for first-layer print moves, then chooses `bridge_acceleration` when `is_bridge(path.role())`, before sparse infill, wall, top-surface, or default acceleration.

## Scope

Implement runtime consumption of `bridge_acceleration` for existing Ares bridge print roles:

- `PrintPathRole::Bridge`
- `PrintPathRole::InternalBridge`

Do not add bridge detection, support generation, thicker bridge generation, or new path construction. This slice only affects paths that already carry a bridge role.

Rust destination boundary:

- `crates/ares-core/src/options/acceleration.rs` parses and resolves the option into acceleration settings.
- `crates/ares-core/src/speeds.rs` stores the resolved value and applies print-role acceleration precedence.
- `crates/ares-core/src/tests/acceleration_gcode.rs` covers generated G-code behavior and invalid input.
- `crates/ares-core/src/pipeline/tests/internal_bridge.rs` may remain unchanged unless a focused fixture is reused; no registry metadata files are in scope.

## Behavior

`bridge_acceleration` is parsed from `SliceOptions` as a non-negative number or a percentage string.

- Missing value defaults to `50%` of `outer_wall_acceleration`, matching Orca's `ratio_over = "outer_wall_acceleration"` registration.
- Numeric values are absolute `mm/s^2`.
- Percentage values are resolved over the parsed `outer_wall_acceleration`.
- Invalid values are rejected with `SliceError::InvalidInput`.
- `0` disables the bridge-specific override and falls through to the later acceleration precedence.

Acceleration precedence for print moves becomes:

1. If `default_acceleration` is `0`, emit no acceleration commands.
2. For travel moves, use `travel_acceleration` only when positive; do not fall back to default.
3. For first-layer print moves, positive `initial_layer_acceleration` wins over every role-specific acceleration, including bridge acceleration.
4. For non-first-layer bridge print moves, positive `bridge_acceleration` wins over sparse infill, wall, and default acceleration.
5. Existing sparse infill, outer wall, inner wall, and default behavior remains unchanged.

The existing `M204` writer behavior remains unchanged: round with existing formatting, suppress unchanged commands, and append the existing comment when `gcode_comments` is true.

## Tests

Add focused tests that prove:

- A manually constructed `Bridge` print path receives `M204` from `bridge_acceleration`.
- A manually constructed `InternalBridge` print path receives `M204` from `bridge_acceleration`.
- Omitted `bridge_acceleration` defaults to `50%` of `outer_wall_acceleration`.
- Percentage bridge acceleration is resolved over `outer_wall_acceleration`, not `default_acceleration`.
- Positive `initial_layer_acceleration` overrides bridge acceleration on first-layer bridge paths.
- `bridge_acceleration: 0` falls through to default print acceleration for bridge paths.
- Invalid bridge acceleration values are rejected.

Existing acceleration tests continue to cover default suppression, travel behavior, rounding, and comment emission.

## Documentation Impact

No user-facing docs are required beyond this source-cited design and implementation plan. The option already exists in generated option metadata; this slice changes runtime G-code behavior.

## Out Of Scope

- `bridge_no_support`
- `thick_bridges`
- support-material bridge path generation
- bridge geometry classification
- top-surface or solid-infill acceleration
- any new option registry rows
