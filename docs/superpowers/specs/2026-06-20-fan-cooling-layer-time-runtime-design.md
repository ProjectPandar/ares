# Fan Cooling Layer Time Runtime Design

## Goal

Consume OrcaSlicer's `fan_cooling_layer_time` option in Ares G-code emission so short layers raise the baseline part-cooling fan command by interpolating between `fan_max_speed` and `fan_min_speed` from the already generated layer speed plan.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1521` declares `fan_cooling_layer_time` as a `ConfigOptionFloats` print setting.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2349-2357` defines `fan_cooling_layer_time`, documents fan interpolation by layer printing time, constrains it to `0..=1000`, and defaults it to `60.0`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:722-812` computes the baseline part-cooling fan command for a layer. After the first-layer fan suppression gate, it reads `fan_min_speed`, `fan_max_speed`, `slow_down_layer_time`, and `fan_cooling_layer_time`; if `layer_time < slow_down_layer_time` it uses full fan, else if `layer_time < fan_cooling_layer_time` it interpolates between max and min fan speeds. The existing full-fan-layer ramp then scales the computed baseline fan value before `GCodeWriter::set_fan`.

## Ares Destination Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns single-extruder part-cooling fan option parsing and layer fan baseline selection.
- `crates/ares-core/src/gcode.rs` already has access to `layer_speed_moves` when emitting each layer and is the narrow destination for passing a layer time into fan baseline selection.
- `crates/ares-core/src/options/tests/part_cooling_fan_runtime.rs` covers parsing and fan baseline selection.
- `crates/ares-core/src/pipeline/tests/slow_down_layers.rs` is extended with one G-code proof that the option changes emitted `M106` commands through the current pipeline.

## Included Behavior

- Missing `fan_cooling_layer_time` defaults to `60.0` seconds.
- `fan_cooling_layer_time` accepts the same first-entry scalar/vector numeric forms currently used by Ares fan-speed vector options: number, numeric string, non-empty numeric array, and semicolon/comma-separated numeric string. Ares consumes the first entry because the current runtime path is single-extruder.
- Values must be finite and in `0.0..=1000.0`. Negative values, values above `1000.0`, non-numeric strings, non-finite strings, null, objects, empty arrays, and arrays whose first entry is invalid are rejected.
- Ares computes current layer print time from `layer_speed_moves`: sum XY distance divided by `speed_mm_s` for moves after the first print extrusion on that layer, using the same layer-local evidence already produced after role speed selection, slow-down-layers interpolation, small-perimeter handling, volumetric caps, and layer-time slowdown.
- If the layer is before `close_fan_the_first_x_layers`, the fan baseline remains suppressed.
- If `fan_max_speed` is `0`, fan baseline commands remain suppressed.
- If computed layer time is shorter than `slow_down_layer_time`, baseline fan uses `fan_max_speed`.
- If computed layer time is at least `slow_down_layer_time` and shorter than `fan_cooling_layer_time`, baseline fan speed is `round(t * fan_min_speed + (1 - t) * fan_max_speed)`, where `t = (layer_time - slow_down_layer_time) / (fan_cooling_layer_time - slow_down_layer_time)`.
- If computed layer time is at least `fan_cooling_layer_time`, baseline fan uses the current layer-number fan ramp behavior.
- The existing `full_fan_speed_layer` ramp continues to scale the computed baseline fan value for early layers.
- Existing role fan overrides for bridges/internal bridges, auxiliary fan commands, part-cooling minimum PWM clamping, pressure advance, speed planning, and extrusion behavior stay unchanged.

## Deferred Behavior

- Full Orca `CoolingBuffer` G-code post-processing is not ported.
- `reduce_fan_stop_start_freq` is not consumed in this slice.
- Multi-extruder fan selection and per-extruder layer-time fan changes are deferred until Ares has multi-extruder path ownership.
- Support-interface fan markers, ironing fan markers, custom G-code cooldown markers, arcs, wipes, and wipe-tower cooling behavior remain deferred.
- This slice does not add option metadata, crates, dependencies, CLI flags, WASM bindings, or independent Ares pipeline stages.

## Acceptance Criteria

- Runtime option tests prove `fan_cooling_layer_time` defaults to `60.0`, accepts supported first-entry numeric forms, and rejects invalid/out-of-range values.
- Runtime option tests prove baseline fan speed uses full fan below `slow_down_layer_time`, interpolates between min and max below `fan_cooling_layer_time`, and falls back to the existing layer-number ramp at or above `fan_cooling_layer_time`.
- Runtime option tests prove `close_fan_the_first_x_layers`, `fan_max_speed = 0`, and `full_fan_speed_layer` still apply.
- Pipeline/G-code tests prove `fan_cooling_layer_time` changes emitted part-cooling `M106` commands through the existing formatting path for a short layer.
- Existing part-cooling fan, role fan, layer-time slowdown, and slow-down-layers tests remain green under `cargo nextest run`.
- All touched Rust files remain at or below 400 LOC; split files before growing past that limit.

## Verification

- RED/GREEN targeted tests:
  - `cargo nextest run -p ares-core part_cooling_fan_runtime`
  - `cargo nextest run -p ares-core slow_down_layers`
- Full verification before commit:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - Rust LOC guard for touched Rust files.
