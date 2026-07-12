# Consume `first_x_layer_fan_speed` in auxiliary fan G-code

## Problem

Ares already parses OrcaSlicer auxiliary fan options and exposes `first_x_layer_fan_speed` through layer-change placeholders, but the runtime auxiliary fan control ignores it. This leaves an existing option as metadata-only: generated `M106 P2` auxiliary fan commands can ramp from zero to `additional_cooling_fan_speed`, but cannot use the special first-X-layer auxiliary fan speed from Orca profiles.

This slice consumes the existing option in concrete slicing/G-code behavior instead of adding more option metadata.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1475-1478` declares:
  - `additional_cooling_fan_speed`
  - `close_additional_fan_first_x_layers`
  - `additional_fan_full_speed_layer`
  - `first_x_layer_fan_speed`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4679-4695` defines `additional_fan_full_speed_layer` ramp semantics and `first_x_layer_fan_speed` as a `0..=100` percent value with default `0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2836-2839` publishes `first_x_layer_fan_speed`, `close_additional_fan_first_x_layers`, and `additional_fan_full_speed_layer` to the placeholder parser.
- `OrcaSlicer/resources/profiles/BBL/machine/Bambu Lab X2D 0.4 nozzle.json:9` uses those placeholders in `layer_change_gcode` to emit `M106 P2` auxiliary fan commands:
  - first-X layers: `first_x_layer_fan_speed`
  - ramp layers: linear interpolation from `first_x_layer_fan_speed` to `additional_cooling_fan_speed`
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:743,815-818` uses `additional_cooling_fan_speed` with `GCodeWriter::set_additional_fan()` for the existing full-speed auxiliary fan runtime path.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1141-1151` and `GCodeWriter.hpp:107-108` define additional fan output as `M106 P2 S...`.

The first-X auxiliary fan behavior is intentionally sourced from Orca's bundled Bambu machine profile template rather than from `CoolingBuffer`: Orca's C++ core publishes these `PrintConfig` values as placeholders, and the shipped profile template is the upstream source that turns those placeholders into first-X and ramp layer-change `M106 P2` commands. The full-speed fallback is not sourced from the X2D template; this slice preserves Ares' existing `additional_cooling_fan_speed` runtime behavior, whose upstream boundary is `CoolingBuffer.cpp` plus `GCodeWriter::set_additional_fan()`. Ares already has a direct Rust auxiliary-fan runtime path instead of a general profile-template interpreter; this slice ports the same upstream first-X/ramp placeholder expression into that existing runtime path. It does not invent a new Ares fan policy.

## Ares destination boundary

- `crates/ares-core/src/options/auxiliary_fan.rs`
  - Extend `AuxiliaryFanControl` to carry parsed `first_x_layer_fan_speed`.
  - Make `SliceOptions::auxiliary_fan_control()` validate and consume `first_x_layer_fan_speed`, not only `auxiliary_fan_placeholders()`.
  - Update `AuxiliaryFanControl::speed_for_layer()` to compute first-X and ramp speeds.
- `crates/ares-core/src/gcode_auxiliary_fan.rs`
  - Keep the existing `M106 P2` emission path and Klipper suppression.
  - No new writer API is required.
- Focused tests under `crates/ares-core/src/options/tests/` and `crates/ares-core/src/pipeline/tests/` prove both the control calculation and concrete generated G-code.

## Included behavior

- `first_x_layer_fan_speed` accepts the same first-value numeric forms as existing placeholder parsing, validates finite percent values in `0..=100`, and defaults to `0`.
- Numeric domain and calculation order:
  - parse `first_x_layer_fan_speed` as a percent-domain `f64`;
  - keep interpolation in percent-domain `f64`, using `additional_cooling_fan_speed` as a percent-domain integer converted to `f64`;
  - convert only the final desired percent to the existing `u8` percent returned by `speed_for_layer()`;
  - use nearest-percent rounding for that conversion: `(percent + 0.5).floor().clamp(0.0, 100.0) as u8`;
  - `GCodeWriter::set_additional_fan()` then converts the `u8` percent to PWM with its existing floor rule, `floor(255 * percent / 100)`.
- `AuxiliaryFanControl::speed_for_layer(layer_index)` uses Orca's one-based layer comparisons:
  - if `auxiliary_fan` is false, no auxiliary fan command is produced;
  - if `layer_index + 1 <= close_additional_fan_first_x_layers`, use `first_x_layer_fan_speed`;
  - else if `layer_index + 1 < additional_fan_full_speed_layer` and `additional_fan_full_speed_layer > close_additional_fan_first_x_layers`, linearly interpolate from `first_x_layer_fan_speed` to `additional_cooling_fan_speed`;
  - otherwise use `additional_cooling_fan_speed`.
- Exact `Option<u8>` command semantics:
  - `None` means no auxiliary fan command should be sent for that layer.
  - `Some(n)` for `n > 0` means send or retain the desired auxiliary fan speed.
  - A desired speed of `0` returns `None` when it would only be an initial no-op, preserving existing default silence before the close threshold.
  - A desired speed of `0` returns `Some(0)` only after a nonzero `first_x_layer_fan_speed` could have turned the fan on during at least one close-threshold layer, so `gcode_auxiliary_fan::layer_command()` can emit `M106 P2 S0` through its existing state tracking.
- Existing default behavior remains unchanged when `first_x_layer_fan_speed` is omitted or `0`: the fan remains silent before the close threshold, then follows the existing ramp/full-speed behavior without initial `M106 P2 S0` noise.
- If nonzero `first_x_layer_fan_speed` turns the fan on but the later target speed is `0`, layer progression emits `M106 P2 S0` once the first-X window ends.
- Completion shutdown remains enabled if either `first_x_layer_fan_speed` or `additional_cooling_fan_speed` can turn the auxiliary fan on.

## Deferred behavior

- Do not implement an Orca machine-profile template interpreter.
- Do not add `M106 P10`, `M142`, chamber/vitrification gates, `max_additional_fan`, current/next extruder selection, or multi-extruder fan selection in this slice.
- Do not implement support, ironing, or full `CoolingBuffer` parity here.
- Do not add dependencies, new crates, native file I/O, terminal behavior, UI, OpenGL, or WASM-incompatible code.

## Docs impact

No roadmap or architecture document update is required for this slice. The change consumes an already-scaffolded option inside the existing auxiliary fan runtime path and does not change crate boundaries, public architecture, milestone ordering, or user-facing CLI/API contracts. The spec, implementation plan, tests, and commit message are the durable documentation for this narrow option-to-behavior port.

## Acceptance tests

- Add RED tests proving current code ignores `first_x_layer_fan_speed`:
  - auxiliary fan control returns first-X speeds for early layers;
  - ramp output interpolates from first-X speed rather than zero;
  - omitted or `0` first-X speed remains silent before the close threshold;
  - nonzero first-X speed with later `additional_cooling_fan_speed = 0` can emit shutdown speed `0` after the first-X window;
  - invalid runtime `first_x_layer_fan_speed` values are rejected by `auxiliary_fan_control()`;
  - concrete G-code for the first layer contains exact `M106 P2` PWM from `first_x_layer_fan_speed`.
- Exact acceptance examples:
  - with `auxiliary_fan = true`, `first_x_layer_fan_speed = 12.5`, `additional_cooling_fan_speed = 70`, and `close_additional_fan_first_x_layers = 2`, `speed_for_layer(0)` and `speed_for_layer(1)` return `Some(13)`, and first-layer G-code emits `M106 P2 S33` because `floor(255 * 13 / 100) = 33`;
  - with `auxiliary_fan = true`, `first_x_layer_fan_speed = 20`, `additional_cooling_fan_speed = 80`, `close_additional_fan_first_x_layers = 2`, and `additional_fan_full_speed_layer = 5`, layer speeds are `Some(20)`, `Some(20)`, `Some(40)`, `Some(60)`, `Some(80)`, then `Some(80)`;
  - with `first_x_layer_fan_speed = 0`, the existing ramp example remains `None`, `None`, `Some(27)`, `Some(53)`, `Some(80)`, so no initial `M106 P2 S0` is emitted.
- GREEN verification must use `cargo nextest run`, not `cargo test`.
- Full verification before commit:
  - `cargo fmt --check`
  - focused `cargo nextest run -p ares-core ...`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust file LOC check, preserving the repo's 400 LOC limit.
