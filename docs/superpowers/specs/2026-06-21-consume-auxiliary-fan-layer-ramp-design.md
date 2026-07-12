# Consume Auxiliary Fan Layer Ramp Design

## Goal

Consume the existing OrcaSlicer auxiliary fan layer options into concrete Ares `M106 P2` G-code behavior. This slice must not add new option metadata; it replaces the current Ares compatibility shell that drives the auxiliary fan from the generic part-cooling close-layer option.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1475-1478` declares the `PrintConfig` options `additional_cooling_fan_speed`, `close_additional_fan_first_x_layers`, `additional_fan_full_speed_layer`, and `first_x_layer_fan_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4660-4686` defines `additional_cooling_fan_speed`, `close_additional_fan_first_x_layers`, and `additional_fan_full_speed_layer`. The `additional_fan_full_speed_layer` tooltip is the source for the required linear auxiliary-fan ramp semantics.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:743, 815-818, 896-897` reads `additional_cooling_fan_speed` and emits auxiliary fan changes through `GCodeWriter::set_additional_fan` when `auxiliary_fan` is enabled.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1142-1147` formats auxiliary fan commands as `M106 P2 S<floor(255 * percent / 100)>`.

## Current Ares Boundary

- `crates/ares-core/src/options/auxiliary_fan.rs` already parses `auxiliary_fan`, `additional_cooling_fan_speed`, and auxiliary fan placeholders.
- `crates/ares-core/src/gcode_auxiliary_fan.rs` already writes layer-level `M106 P2` commands using `GCodeWriter::set_additional_fan`.
- `crates/ares-core/src/gcode.rs` already asks `AuxiliaryFanControl::speed_for_layer(...)` for each generated layer.
- `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs` is near the 400 LOC limit and must be split before adding new runtime tests.

## Included Behavior

1. `SliceOptions::auxiliary_fan_control()` must parse `close_additional_fan_first_x_layers` instead of `close_fan_the_first_x_layers` for auxiliary fan layer suppression.
2. `SliceOptions::auxiliary_fan_control()` must parse `additional_fan_full_speed_layer` and carry it into `AuxiliaryFanControl`.
3. `AuxiliaryFanControl::speed_for_layer(layer_index)` must return:
   - `None` when `auxiliary_fan` is false.
   - `None` when `additional_cooling_fan_speed` is `0`.
   - `None` for layers whose zero-based `layer_index` is below `close_additional_fan_first_x_layers`.
   - the full configured speed when `additional_fan_full_speed_layer <= close_additional_fan_first_x_layers`.
   - the full configured speed when `layer_index + 1 >= additional_fan_full_speed_layer`.
   - otherwise a rounded linear ramp: `round(speed * (layer_index + 1 - close_additional_fan_first_x_layers) / (additional_fan_full_speed_layer - close_additional_fan_first_x_layers))`, clamped to `0..=100`.
4. Existing defaults must preserve current default output: with `auxiliary_fan = true`, `additional_cooling_fan_speed = 70`, default close layer `1`, and default full-speed layer `0`, Ares still emits the first auxiliary fan command on layer `1` at `M106 P2 S178`, then shuts it down before `M2`.
5. Klipper flavor continues to skip auxiliary fan commands.
6. Completion shutdown behavior stays unchanged: once Ares starts the auxiliary fan, it emits `M106 P2 S0` before program end for non-Klipper output.

## Deferred Behavior

- `first_x_layer_fan_speed` remains placeholder/custom-G-code data in this slice. The searched Orca C++ boundary exposes it through placeholders, not through the automatic `CoolingBuffer` auxiliary fan writer.
- Full Orca `CoolingBuffer` parity remains deferred: multi-extruder auxiliary fan state, force-resume markers, custom G-code interactions, support/ironing fan marker integration, and exact post-processor command placement are not implemented here.
- Ares keeps one current object/layer fan control path and does not introduce a new cooling-buffer subsystem.

## Tests

- Add or move focused option tests so every touched Rust test file remains at or below 400 LOC.
- A runtime option test must prove `close_additional_fan_first_x_layers` suppresses auxiliary fan output until that zero-based layer threshold.
- A runtime option test must prove `additional_fan_full_speed_layer` applies the linear ramp.
- A runtime option test must prove the generic `close_fan_the_first_x_layers` no longer controls auxiliary fan `M106 P2` behavior.
- A G-code test must prove a real slice emits ramped `M106 P2` commands before layer segment output and uses the existing `GCodeWriter` PWM floor conversion.
- Existing auxiliary fan tests for disabled fan, zero speed, default close threshold, Klipper skipping, invalid values, and completion shutdown must continue to pass.

## Documentation

Update `docs/roadmap.md` with a completed runtime-slice note naming the same Orca source boundary and deferred behavior.

## Verification

- `cargo fmt --check`
- Focused nextest checks for auxiliary fan option and G-code tests.
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC guard for touched Rust files.
