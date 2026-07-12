# Consume Overhang Fan Threshold Bridge Design

## Purpose

Consume the existing OrcaSlicer `overhang_fan_threshold` option into Ares role-based part-cooling fan behavior for the narrow source slice Ares can represent today. The slice must turn the default Orca overhang/bridge fan role handling and the `0%` threshold branch into concrete G-code behavior instead of leaving the option as metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:304-310` defines `OverhangFanThreshold`, including `Overhang_threshold_bridge`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:457-462` maps enum keys `"0%"`, `"10%"`, `"25%"`, `"50%"`, `"75%"`, and `"95%"`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1190-1211` defines `overhang_fan_threshold` and defaults it to `Overhang_threshold_bridge` (`"95%"`).
- `OrcaSlicer/src/libslic3r/GCode.cpp:6775-6810` uses `overhang_fan_threshold` in `check_overhang_fan`; `erBridgeInfill` and `erOverhangPerimeter` return `true` before the threshold switch, `Overhang_threshold_none` returns `is_external_perimeter(role)`, and the other enum branches require per-segment `overlap`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6907-6916` and `7057-7074` feed that threshold decision into overhang and internal-bridge fan markers.

## Ares Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns parsed part-cooling fan runtime options and `RoleFanControl` / `LayerRoleFanControl`.
- `crates/ares-core/src/gcode_role_fan.rs` owns role fan G-code switching before each print move.
- `crates/ares-core/src/pipeline/tests/overhang_bridge_fan_gcode.rs` owns pipeline-level G-code assertions for overhang/bridge fan behavior.

## Included Behavior

- Parse `overhang_fan_threshold` values accepted by Orca for this option: `"0%"`, `"10%"`, `"25%"`, `"50%"`, `"75%"`, `"95%"`.
- Preserve Orca's default: absent `overhang_fan_threshold` behaves as `"95%"`.
- Apply `overhang_fan_speed` to `PrintPathRole::OverhangPerimeter` whenever `enable_overhang_bridge_fan` is true and layer fan ramping allows role fan overrides. This matches Orca's unconditional `erOverhangPerimeter` branch.
- Preserve external bridge `PrintPathRole::Bridge` and `PrintPathRole::InternalBridge` fan override behavior.
- Make `overhang_fan_threshold: "0%"` concrete by also applying `overhang_fan_speed` to `PrintPathRole::ExternalPerimeter`, matching Orca's `Overhang_threshold_none` / `is_external_perimeter(role)` branch as far as Ares can represent it.
- Treat `"10%"`, `"25%"`, `"50%"`, `"75%"`, and `"95%"` as supported threshold values that do not force ordinary `PrintPathRole::ExternalPerimeter` role-fan overrides until Ares carries Orca-equivalent per-segment overlap data. They still preserve bridge, internal-bridge, and overhang-perimeter role fan behavior.
- Keep existing `enable_overhang_bridge_fan`, `overhang_fan_speed`, `internal_bridge_fan_speed`, layer fan ramping, fan min/max baseline restoration, and close-first-layers behavior unchanged.
- Reject unsupported or malformed `overhang_fan_threshold` values with `SliceError::InvalidInput` when role fan control is built.

## Deferred Behavior

- Ares does not yet carry Orca's per-segment overhang overlap values through `LayerPrintPaths`, extrusion moves, speed moves, or G-code moves. The threshold values `"10%"`, `"25%"`, `"50%"`, `"75%"`, and `"95%"` will parse successfully but cannot yet distinguish ordinary external perimeter segments by unsupported-width percentage.
- Full Orca parity for `check_overhang_fan(overlap, role)` is deferred until Ares has a source-cited path or move field for overhang overlap. This slice must not invent a placeholder overlap model.
- BBL-printer-specific cooling marker post-processing remains outside this slice.

## Docs Impact

No user-facing configuration guide currently documents Ares runtime coverage for `overhang_fan_threshold`. This slice is documented by this source-cited spec and pipeline tests; no additional docs file needs to change.

## Acceptance Criteria

- A new focused nextest test proves absent/default `overhang_fan_threshold` applies `overhang_fan_speed` to `PrintPathRole::OverhangPerimeter`.
- A focused nextest test proves `overhang_fan_threshold: "0%"` applies `overhang_fan_speed` to `PrintPathRole::ExternalPerimeter`, matching Orca's "force for all outer walls" branch as far as Ares can represent it.
- A focused nextest test proves an intermediate accepted threshold, such as `"50%"`, parses and preserves overhang-perimeter fan behavior without forcing an ordinary external perimeter fan override.
- Existing bridge and internal-bridge fan tests continue to pass, proving bridge fan overrides are preserved under the default `"95%"` threshold.
- Invalid threshold values produce `SliceError::InvalidInput` mentioning `overhang_fan_threshold`.
- Verification uses `cargo nextest run`, not `cargo test`.

## Safety

The change is limited to `ares-core` pure option parsing and G-code generation. It introduces no filesystem, terminal, UI, OpenGL, native-only, or non-WASM behavior.
