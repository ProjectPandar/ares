# Consume Overhang Bridge Fan Design

## Goal

Consume the existing `enable_overhang_bridge_fan` and `overhang_fan_speed` options in generated part-cooling fan G-code as a source-cited Rust rewrite slice of OrcaSlicer's role-based bridge fan override. Integrate this with Ares' existing `internal_bridge_fan_speed` runtime behavior so internal bridges can use Orca's overhang-fan fallback when their dedicated speed is `-1`.

## Source Boundary

This slice ports the narrow runtime behavior from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1502-1503`, which declares `enable_overhang_bridge_fan` and `overhang_fan_speed` on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1170-1188`, which defines `enable_overhang_bridge_fan` as default `true` and `overhang_fan_speed` as a `0..=100` integer percent defaulting to `100`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6775-6810`, where bridge infill and overhang perimeter paths are eligible for overhang fan markers when the feature is enabled.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6906-6916` and `7057-7077`, where enabled bridge/overhang paths receive `_OVERHANG_FAN_START` markers and internal bridge paths receive `_INTERNAL_BRIDGE_FAN_START` markers.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:770-788`, where `overhang_fan_speed` is read, ramp-adjusted with the regular fan ramp, used only when greater than the current baseline fan candidate, and used as the fallback internal-bridge speed when `internal_bridge_fan_speed < 0`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:796-807`, where close-fan-first-layers disables overhang and internal-bridge fan controls before the configured threshold.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:850-869` and `984-990`, where role fan start/end markers switch to the role fan speed and then restore the regular fan baseline.

Existing Ares registry and option-line metadata for these options are reused. No option metadata is added.

## Ares Destination Boundary

The Rust destination boundary is limited to:

- `crates/ares-core/src/options/part_cooling_fan.rs`: parse `enable_overhang_bridge_fan`, parse `overhang_fan_speed`, and expose a small role-fan control value alongside the existing `internal_bridge_fan_speed`.
- `crates/ares-core/src/options/tests/part_cooling_fan_runtime.rs`: keep existing part-fan and internal-bridge tests unless moving tests is required to preserve the 400 LOC gate.
- `crates/ares-core/src/options/tests/role_fan_runtime.rs`: runtime option parsing and role-fan selection tests for this slice.
- `crates/ares-core/src/options/tests.rs`: register `role_fan_runtime` if the new runtime test file is used.
- `crates/ares-core/src/gcode.rs`: pass the parsed role-fan control and close-first-layers eligibility into the role-fan G-code state machine.
- `crates/ares-core/src/gcode_role_fan.rs`: extend the existing role-fan state machine from internal bridges only to bridge and internal-bridge fan overrides.
- `crates/ares-core/src/pipeline/tests/internal_bridge_fan_gcode.rs`: pipeline-level G-code tests for internal-bridge role fan overrides.
- `crates/ares-core/src/pipeline/tests/overhang_bridge_fan_gcode.rs`: pipeline-level G-code tests for bridge role fan overrides.
- `crates/ares-core/src/pipeline/tests/role_fan_gcode_support.rs`: shared pipeline fixture for role fan G-code tests, split out to keep all Rust files under the 400 LOC gate.

No registry definitions, generated option-line metadata, public API, CLI, WASM, profile loading, UI, geometry classification, support generation, ironing generation, overhang-perimeter detection, fan command formatting, or unrelated pipeline behavior is in scope. All changed Rust files under `crates/` must remain at or below 400 LOC.

## Runtime Behavior

Ares must parse:

- `enable_overhang_bridge_fan`: strict JSON boolean, default `true`, using the existing `SliceOptions::bool_option` behavior.
- `overhang_fan_speed`: first integer-compatible vector entry, default `100`, valid range `0..=100`.
- `internal_bridge_fan_speed`: existing `-1..=100` first-entry behavior remains, but `-1` now means "use the overhang bridge fan fallback when that fallback is active" instead of "no role fan behavior at all".

`overhang_fan_speed` accepts the same first-entry forms already used for integer-like fan vectors in this module: scalar integer JSON number, integer string, first entry of a JSON number array, first entry of a semicolon-separated string, and first entry of a comma-separated string. It rejects fractional numbers or strings, negative values, values above `100`, empty strings, strings with empty list entries, empty arrays, arrays containing non-numeric values, booleans, null, and objects with `SliceError::InvalidInput` mentioning `overhang_fan_speed`.

For Ares' current print roles:

- `PrintPathRole::Bridge` maps to Orca's bridge infill overhang fan path. When `enable_overhang_bridge_fan` is true, the current layer is at or after `close_fan_the_first_x_layers`, and the adjusted `overhang_fan_speed` is greater than the current regular fan baseline speed, Ares emits `M106` for the adjusted overhang speed before the bridge extrusion move and restores the regular fan baseline after leaving the bridge role.
- `PrintPathRole::InternalBridge` maps to Orca's internal bridge fan path. When `enable_overhang_bridge_fan` is true and the current layer is at or after `close_fan_the_first_x_layers`, explicit `internal_bridge_fan_speed` values `0..=100` override the fan before internal-bridge moves and restore the regular baseline afterward, preserving Orca's behavior of reading this speed after overhang ramp adjustment. When `internal_bridge_fan_speed == -1`, Ares uses the adjusted overhang bridge fan fallback only if that adjusted fallback is greater than the current regular fan baseline speed.
- If the regular baseline fan is disabled by `fan_max_speed = 0` but the layer is past the close-first-layers threshold, `overhang_fan_speed > 0` may still turn the fan on for bridge or fallback internal-bridge moves, matching Orca's role-fan control being separate from the regular fan maximum.
- If the layer is before `close_fan_the_first_x_layers`, no bridge or internal-bridge role fan override is emitted, matching Orca's `CoolingBuffer.cpp:796-807` branch.
- If `enable_overhang_bridge_fan` is false, no bridge or internal-bridge role fan override is emitted, including explicit `internal_bridge_fan_speed`.
- Existing `part_cooling_fan_min_pwm` behavior still applies when formatting any non-zero role fan command.

Role fan speeds must use the same full-fan-speed-layer ramp factor that Orca applies to `overhang_fan_speed` in `CoolingBuffer.cpp:771-775`:

- Let `layer_id` be the zero-based layer index, `close` be `close_fan_the_first_x_layers`, and `full` be `full_fan_speed_layer`.
- If `layer_id + 1 < full` and `full > close`, compute `factor = (layer_id + 1 - close) / (full - close)`.
- The adjusted role speed is `floor(raw_role_speed * factor + 0.5)` clamped to `0..=100`.
- Otherwise the adjusted role speed is the raw role speed.

Bridge overrides and fallback internal-bridge overrides compare this adjusted overhang speed to the current regular fan baseline. Explicit `internal_bridge_fan_speed` values `0..=100` are not ramp-adjusted because Orca reads them after the overhang ramp block in `CoolingBuffer.cpp:783-784`.

This slice intentionally does not detect overhang perimeters from geometry. It only consumes the source-cited bridge and internal-bridge role paths that Ares already models.

## Deferred Behavior

This slice does not implement:

- `overhang_fan_threshold` geometry-based overhang-perimeter classification.
- Support-interface fan behavior from `support_material_interface_fan_speed`.
- Ironing fan behavior from `ironing_fan_speed`.
- Layer-time cooling from `slow_down_layer_time` or `fan_cooling_layer_time`.
- Print slowdown from `slow_down_min_speed` or `slow_down_for_layer_cooling`.
- `reduce_fan_stop_start_freq`.
- Auxiliary, exhaust, chamber, or air-filtration fan behavior.
- Multi-extruder fan switching beyond Ares' existing first-entry semantics.
- FanMover delayed command placement.
- New option registry metadata or generated `PrintConfig` classes.

## Docs Impact

No user-facing documentation, architecture decision record, roadmap update, or example update is required. The SDD spec and plan are the traceable design artifacts for this internal runtime slice.

## Acceptance Criteria

- `fan_min_speed = 20`, `fan_max_speed = 40`, `full_fan_speed_layer = 1`, `close_fan_the_first_x_layers = 0`, and `overhang_fan_speed = 75` emit an overhang bridge fan command before `PrintPathRole::Bridge` extrusion and restore the prior regular fan baseline afterward.
- With default fan speeds and `close_fan_the_first_x_layers = 0`, a bridge path emits no overhang bridge fan override because the adjusted default `overhang_fan_speed = 100` is not greater than the regular baseline `fan_max_speed = 100`.
- `enable_overhang_bridge_fan = false` suppresses bridge and internal-bridge role fan commands.
- `overhang_fan_speed` lower than or equal to the regular fan baseline does not emit a bridge role fan override.
- `fan_max_speed = 0`, `close_fan_the_first_x_layers = 0`, and `overhang_fan_speed = 75` emit `M106 S191` for a bridge path and `M106 S0` after returning to a non-bridge role.
- `fan_max_speed = 0`, `close_fan_the_first_x_layers = 0`, `full_fan_speed_layer = 4`, and `overhang_fan_speed = 100` emit `M106 S63` for a layer-0 bridge path, proving the role fan speed is ramp-adjusted to 25 percent and then formatted through Ares' existing fan PWM writer rather than emitted as raw 100 percent.
- `close_fan_the_first_x_layers = 1` suppresses bridge role fan commands on layer 0.
- Explicit `internal_bridge_fan_speed = 75` still overrides internal-bridge paths when `enable_overhang_bridge_fan` is true.
- Explicit `internal_bridge_fan_speed = 75` with `full_fan_speed_layer = 4` still emits `M106 S191`, proving explicit internal-bridge speed is not ramp-adjusted.
- `internal_bridge_fan_speed = -1` uses the overhang bridge fan fallback for internal bridges when the fallback is active.
- Invalid `enable_overhang_bridge_fan`, `overhang_fan_speed`, or `internal_bridge_fan_speed` values return `SliceError::InvalidInput` mentioning the offending option key.
- No new option metadata is added.
- All changed Rust files under `crates/` remain at or below 400 LOC.

## Verification Criteria

The implementation is not complete until all of the following pass with fresh output:

- Targeted runtime option tests for `enable_overhang_bridge_fan`, `overhang_fan_speed`, fallback internal-bridge selection, close-first-layers gating, and invalid inputs.
- Targeted pipeline-level G-code tests for bridge override/restore, disabled gate, lower-than-baseline suppression, zero-baseline bridge override, close-first-layers suppression, explicit internal-bridge override, and internal-bridge fallback.
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; bad=1 } END { exit bad }'`
