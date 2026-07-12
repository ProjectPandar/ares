# Consume Travel Speed Z Design

## Goal

Implement a source-cited OrcaSlicer rewrite slice that makes the parsed `travel_speed_z` option change Ares Z-only travel G-code feedrates. Positive `travel_speed_z` should control layer-change `G1 Z... F...` moves, while `0` keeps Orca's fallback to `initial_layer_travel_speed` on the first layer and `travel_speed` after the first layer.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1396-1397` declares `travel_speed` and `travel_speed_z` as adjacent `GCodeConfig` float options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6618-6626` registers `travel_speed_z` as a `coFloat` option, sets minimum `0`, and defaults it to `0`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:831-846` implements `_travel_to_z`: read `config.travel_speed_z.value`; when it is `0`, fall back to `initial_layer_travel_speed` on the first layer or `travel_speed` otherwise; emit `F speed * 60.0`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:849-857` applies the same speed fallback in `_spiral_travel_to_z`.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs` already parses `travel_speed` and `initial_layer_travel_speed` into `SpeedOptions`; it will also parse non-negative `travel_speed_z` without adding a new public option registry slice.
- `crates/ares-core/src/speeds/config.rs` owns `SpeedOptions`; it will store `travel_speed_z` and expose a Z-travel feedrate helper.
- `crates/ares-core/src/gcode.rs` owns final in-memory G-code formatting; it will use the Z-travel helper only for layer-change `writer.travel_to_z_with_comment(...)` calls.
- `crates/ares-core/src/pipeline/tests/initial_layer_speeds.rs` already inspects layer Z travel feedrates; it will cover the G-code behavior.

## Included Behavior

- Missing `travel_speed_z` behaves like Orca default `0`.
- Explicit `travel_speed_z = 0` keeps the existing fallback: first-layer Z travel uses `initial_layer_travel_speed`, and later layer Z travel uses `travel_speed`.
- Positive `travel_speed_z` controls first-layer and later layer Z-only travel feedrates as `travel_speed_z * 60.0`.
- XY travel feedrates continue to use `initial_layer_travel_speed` on the first layer and `travel_speed` after the first layer.
- `travel_speed_z` accepts JSON numbers and numeric strings, matching existing Ares numeric speed parsing.
- Negative, non-finite, non-numeric, boolean, null, and percent `travel_speed_z` values return `SliceError::InvalidInput`.

## Deferred Behavior

- Z-hop, slope lift, spiral lift, `_spiral_travel_to_z`, and combined XYZ travel paths remain out of scope because Ares does not yet port those upstream writer paths.
- Machine limit simulation and `silent_mode` remain out of scope.
- No new crates, dependencies, file I/O, CLI-only behavior, option metadata changes, or Ares-owned pipeline redesign are introduced.

## Acceptance Criteria

- `SliceOptions::default().speed_options()` reports `travel_speed_z` as `0`.
- Parsing `travel_speed_z = 25` and `travel_speed_z = "35"` reaches `SpeedOptions`.
- Invalid `travel_speed_z` values are rejected before G-code output.
- With `travel_speed_z = 25`, layer-change `G1 Z...` feedrates are `1500.0` on first and later layers, while XY travel feedrates still follow `initial_layer_travel_speed` and `travel_speed`.
- With `travel_speed_z = 0`, layer-change Z feedrates keep the existing fallback behavior.
- No Rust file under `crates/` exceeds 400 LOC.
