# Consume Internal Bridge Fan Speed Design

## Goal

Consume OrcaSlicer `internal_bridge_fan_speed` as concrete Ares G-code behavior for already-modeled internal bridge print moves, instead of adding more option metadata.

## Upstream Source Boundary

These anchors are for the repository-local `OrcaSlicer` checkout at commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24` on branch `main`.

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1629` declares `internal_bridge_fan_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3350-3359` defines the option as "Internal bridges fan speed", percent range `-1..100`, default `-1`, with `-1` meaning use overhang fan speed behavior instead.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6820-6840` defines role-based fan marker start/end emission in `append_role_based_fan_marker`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6914-6915` emits `_INTERNAL_BRIDGE_FAN_START` / `_INTERNAL_BRIDGE_FAN_END` markers through `append_role_based_fan_marker(erInternalBridgeInfill, "_INTERNAL_BRIDGE", path.role() == erInternalBridgeInfill)` in the non-variable-speed path when overhang/bridge fan handling is active.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7075-7076` keeps the same internal bridge fan marker active for `erInternalBridgeInfill` in the variable-speed path.

## Ares Destination Boundary

- `crates/ares-core` remains the only changed crate.
- `SliceOptions` parses the existing `internal_bridge_fan_speed` value into a runtime helper in the current part-cooling option module.
- `gcode`/move emission consumes the helper for `PrintPathRole::InternalBridge` print moves.
- Tests live in a new focused test shard if the existing part-cooling G-code test file is too close to the 400 LOC limit.

## Behavior

When `internal_bridge_fan_speed` is a percent from `0` to `100`, Ares must emit a part-cooling fan command before an internal bridge print move if the current part-cooling fan state is not already that speed.

When the next print move is not `PrintPathRole::InternalBridge`, Ares must exit the internal bridge override before that print move. If the layer has a baseline part-cooling fan speed, Ares restores that baseline if it differs from the current fan state. If the layer has no baseline part-cooling fan speed, Ares emits `GCodeWriter::set_fan(0)` before the non-internal-bridge print move so the internal bridge override cannot leak into later roles.

When `internal_bridge_fan_speed` is `-1` or absent, Ares must preserve current behavior: no role-specific fan command is emitted for internal bridge moves.

Internal bridge fan commands use the existing `GCodeWriter::set_fan` formatting and therefore inherit current gcode flavor and `part_cooling_fan_min_pwm` behavior.

## Accepted Inputs

- Missing `internal_bridge_fan_speed`: disabled, equivalent to Orca default `-1`.
- Numeric scalar integer from `-1` through `100`.
- Numeric vector/list/string forms accepted by existing Orca-style percent-list parsers; Ares consumes the first value for the current single-filament pipeline.

Invalid values below `-1`, above `100`, non-integral values, empty lists, or non-numeric list members must return `SliceError::InvalidInput` naming `internal_bridge_fan_speed`.

## Testing

- Add a focused unit/integration test that builds a small `SlicingPipeline` containing:
  - a baseline layer fan speed,
  - a non-internal-bridge print move,
  - an internal bridge print move,
  - another non-internal-bridge print move.
- The test must assert the emitted fan sequence includes the baseline fan command, an internal bridge override command before the internal bridge print move, and a baseline restore before the later non-internal-bridge print move.
- Add a test with no layer baseline fan speed proving Ares emits fan-off before a later non-internal-bridge print move after an internal bridge override.
- Add a test proving `-1` suppresses internal-bridge fan override output.
- Add a test proving invalid `internal_bridge_fan_speed` reaches `SliceError::InvalidInput`.
- Run at minimum:
  - `cargo fmt --check`
  - targeted internal bridge fan tests
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - a LOC gate confirming edited `crates/ares-core/src/*.rs` files do not exceed 400 lines.

## Deferred Behavior

- Full Orca `CoolingBuffer` marker post-processing is deferred.
- `overhang_fan_speed`, `overhang_fan_threshold`, support interface fan speed, ironing fan speed, variable-speed overlap regions, multi-extruder selection, and `FanMover` parity are deferred.
- Ares will not invent support/ironing roles or overhang overlap classification in this slice.
- The `-1` fallback to Orca overhang fan-speed behavior remains deferred because Ares does not yet have the corresponding upstream overhang role fan slice.

## Docs Impact

No public API or user guide update is required in this slice. The behavior is covered by the spec document and tests because Ares has not yet introduced end-user option documentation beyond source-cited milestone/spec artifacts for these runtime option-consumption slices.

## Safety

The change is local, deterministic G-code emission. It performs no file I/O in `ares-core`, adds no dependency, and remains WASM-safe.
