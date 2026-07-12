# Consume Physical Extruder Map G-code Design

## Scope

Consume the existing OrcaSlicer `physical_extruder_map` option in Ares runtime G-code placeholder rendering. This is a source-cited Rust rewrite slice, not a new Ares-owned pipeline feature.

Upstream boundaries:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1341` declares `((ConfigOptionInts, physical_extruder_map))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2407-2412` defines `physical_extruder_map` as "Map the logical extruder to physical extruder" with default `ConfigOptionInts{0}`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4545` adds `most_used_physical_extruder_id` to `layer_change_gcode` placeholder config using `physical_extruder_map.get_at(most_used_extruder)`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4999-5000` adds `most_used_physical_extruder_id` and `curr_physical_extruder_id` to `time_lapse_gcode`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5059-5060` adds the same physical-extruder placeholders to `wrapping_detection_gcode`.

## Current Ares State

Ares preserves `physical_extruder_map` in the option registry and default known-count values. The former source-line-only tuple test was removed by the Option pinning cleanup. Ares also already renders `layer_change_gcode`, `time_lapse_gcode`, and `wrapping_detection_gcode` placeholders in `crates/ares-core/src/gcode_placeholders.rs` and `crates/ares-core/src/gcode_wrapping_detection.rs`; the physical-extruder placeholders are either absent or hard-coded to logical extruder `0`.

## Design

Add a small runtime parser that resolves a logical extruder index to a physical extruder id from `physical_extruder_map`.

Rules:
- Missing `physical_extruder_map` uses Orca's default identity for logical extruder `0`, yielding physical id `0`.
- JSON arrays of non-negative integers are accepted, matching the existing registry default shape `[0]`.
- JSON numeric scalar and string scalar values are accepted for consistency with existing Ares option parsing patterns when users provide a single value.
- The resolved value for a logical extruder index uses Orca-style `get_at` behavior: if the requested logical index is beyond the vector length, use the final configured value.
- Empty arrays are invalid because there is no default-preserving element to read.
- Negative, non-integer, non-finite, object, boolean, and unparseable string values are invalid at the option boundary with an error naming `physical_extruder_map`.

Use the resolver in:
- `layer_change_gcode`: render `{most_used_physical_extruder_id}` and `[most_used_physical_extruder_id]` using logical extruder `0`, which is Ares' current single-tool layer context.
- `time_lapse_gcode`: render `{most_used_physical_extruder_id}`, `[most_used_physical_extruder_id]`, `{curr_physical_extruder_id}`, and `[curr_physical_extruder_id]` using logical extruder `0`.
- `wrapping_detection_gcode`: replace the current hard-coded physical ids with the same resolver for logical extruder `0`.

Existing `{layer_num}`, `{layer_z}`, and `{max_layer_z}` behavior must remain unchanged. The slice must not implement multi-tool scheduling, real most-used extruder selection, timelapse parking position placeholders, BBL-printer-specific wrapping behavior, G-code processor metadata, UI behavior, filesystem behavior, or new crates.

## Acceptance Criteria

- A layer-change custom G-code template containing physical-extruder placeholders renders the mapped physical id from `physical_extruder_map`.
- A time-lapse custom G-code template containing both physical-extruder placeholders renders the mapped physical id.
- A wrapping-detection custom G-code template containing both physical-extruder placeholders renders the mapped physical id instead of hard-coded `0`.
- Missing `physical_extruder_map` keeps current output for logical extruder `0`.
- Invalid `physical_extruder_map` input fails slicing with a `SliceError::InvalidInput` mentioning `physical_extruder_map`.
- The implementation stays inside `ares-core`, remains WASM-compatible, adds no dependencies, and keeps touched Rust files at or below 400 LOC.
- Verification uses `cargo nextest run`, including a focused test run and full workspace run, followed by rustfmt, clippy, wasm check, `git diff --check`, and a touched Rust LOC guard.

## Test Strategy

Use TDD with a new focused integration test module under `crates/ares-core/src/tests/`.

RED tests:
- `layer_change_gcode_uses_physical_extruder_map_for_most_used_placeholder`
- `time_lapse_gcode_uses_physical_extruder_map_for_current_and_most_used_placeholders`
- `wrapping_detection_gcode_uses_physical_extruder_map_placeholders`
- `invalid_physical_extruder_map_is_rejected`

Run the focused tests with:

```bash
cargo nextest run -p ares-core physical_extruder_map_gcode
```

Then implement the smallest runtime parser and placeholder substitutions needed to pass them.

## Documentation Impact

This spec and the implementation plan are the durable documentation for this slice. No public CLI, README, or user-facing configuration docs change is required because the option already exists in the registry and this slice only consumes it in runtime G-code placeholders.
