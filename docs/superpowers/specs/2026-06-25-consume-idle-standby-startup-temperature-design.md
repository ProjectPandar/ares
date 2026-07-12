# Consume Idle Standby Startup Temperature Design

## Goal

Consume the existing OrcaSlicer `ooze_prevention`, `idle_temperature`, and `standby_temperature_delta` options in Ares startup G-code for multi-tool first-layer nozzle temperature commands. This is a concrete runtime slice: Ares must emit per-tool startup nozzle temperature commands and apply the upstream inactive-tool temperature rule when ooze prevention is enabled, instead of only recording option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1545` declares `ooze_prevention`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4837-4843` defines `ooze_prevention` as a boolean option with default `false`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1565` declares `standby_temperature_delta`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5745-5755` defines `standby_temperature_delta` as an integer option with default `-5` and print-settings semantics for inactive extruder temperature variation.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1603` declares `idle_temperature`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6898-6905` defines `idle_temperature` as an integer vector with default `0` and filament-setting semantics for inactive nozzle temperature.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2276-2281` initializes ooze prevention as `ooze_prevention && !single_extruder_multi_material`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3999-4031` is the included runtime behavior: when custom start G-code does not already set nozzle temperature and the print is not single-extruder multi-material, Orca emits first-layer temperatures for all printing extruders. For non-initial tools under enabled ooze prevention, `idle_temperature[tool]` overrides the inactive tool temperature when non-zero; otherwise `standby_temperature_delta` is added to that tool's first-layer nozzle temperature.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-628` defines `ConfigOptionVector<T>::get_at` so vector indexes beyond the provided values fall back to the first value, not the last value.

## Ares Boundary

- `crates/ares-core/src/options/nozzle_temperature.rs`
  - Add typed accessors for all first-layer nozzle temperatures, `ooze_prevention`, idle temperatures, and standby temperature delta.
  - Preserve existing scalar/string/vector parsing style for temperature vectors.
  - Add signed scalar parsing for `standby_temperature_delta`.
- `crates/ares-core/src/gcode_startup.rs`
  - Replace the single-tool startup nozzle temperature helper with an all-tool helper that uses the upstream idle/standby rule.
  - Keep existing custom start-G-code suppression for `M104`, `M109`, and RepRapFirmware `G10`.
- `crates/ares-core/src/gcode_start_custom.rs`
  - Pass the existing `StartGCodeCommand::num_extruders` value into the startup helper.
- `crates/ares-core/src/tests/idle_standby_startup_temperature_gcode.rs`
  - Add focused G-code tests proving idle absolute temperature, standby delta fallback, default-disabled ooze prevention, RepRapFirmware formatting, custom-G-code suppression, and invalid input reporting.
- `crates/ares-core/src/tests/mod.rs`
  - Register the focused G-code test module.

## Included Behavior

1. For one extruder, startup output remains the existing single `M104` or RepRapFirmware `G10` command.
2. For multiple extruders, Ares emits one startup nozzle temperature command per extruder before `;LAYER_CHANGE`, using tool ids `T0`, `T1`, ... or RepRapFirmware `P0`, `P1`, ...
3. Tool 0 uses `nozzle_temperature_initial_layer[0]`.
4. When `ooze_prevention` is absent or `false`, tools after 0 use their own first-layer nozzle temperature without idle/standby adjustment.
5. When `ooze_prevention` is `true`, tools after 0 use `idle_temperature[tool]` when that value is non-zero.
6. When `ooze_prevention` is `true`, tools after 0 use `nozzle_temperature_initial_layer[tool] + standby_temperature_delta` when `idle_temperature[tool]` is zero or omitted.
7. The default `standby_temperature_delta` is `-5`.
8. The default `idle_temperature` is `[0]`; if the key is absent, all out-of-range idle lookups fall back to that first default value, so inactive tools use standby delta unless an explicit non-zero idle value is available at their index.
9. Short `nozzle_temperature_initial_layer` and `idle_temperature` vectors use Orca-style first-value fallback for tool indexes beyond the provided vector. For three tools, `idle_temperature = [0, 180]` means tool 1 uses `180` and tool 2 falls back to tool 0's `0`.
10. A computed inactive startup temperature less than or equal to zero emits no command for that inactive tool, matching the upstream `if (temp > 0)` gate.
11. Invalid `ooze_prevention` values produce `SliceError::InvalidInput` mentioning `ooze_prevention`.
12. Invalid `idle_temperature` values produce `SliceError::InvalidInput` mentioning `idle_temperature`.
13. Invalid `standby_temperature_delta` values produce `SliceError::InvalidInput` mentioning `standby_temperature_delta`.

## Deferred Behavior

- Full `OozePrevention::pre_toolchange` and `post_toolchange` behavior from `OrcaSlicer/src/libslic3r/GCode.cpp:267-296`.
- Tool-change, wipe-tower, preheat, and multi-material scheduling behavior.
- Detecting the exact set of used extruders from model assignments. This slice uses the existing Ares hardware count boundary (`nozzle_diameter` length), consistent with existing `[num_extruders]` startup placeholder behavior.
- `single_extruder_multi_material` runtime behavior. Ares does not currently have the corresponding startup path, so this slice treats the existing default Ares path as non-SEMM and gates only on `ooze_prevention`.
- Updating generated metadata milestone modules that intentionally say these options are deferred. Those files describe prior metadata slices; this runtime slice is documented by this spec, plan, tests, and commit.

## Acceptance Criteria

- Focused RED run: after adding the new tests and before implementation, `cargo nextest run -p ares-core idle_standby_startup_temperature` fails because only one startup nozzle command is emitted and idle/standby inputs are ignored.
- Focused GREEN run: after implementation, `cargo nextest run -p ares-core idle_standby_startup_temperature` passes.
- Exact normal-flavor expected startup lines for `nozzle_diameter = [0.4, 0.6, 0.8]`, `nozzle_temperature_initial_layer = [210, 230, 240]`, `ooze_prevention = true`, `idle_temperature = [0, 180, 0]`, and `standby_temperature_delta = -10` are:
  - `M104 S210 T0 ; set nozzle temperature`
  - `M104 S180 T1 ; set nozzle temperature`
  - `M104 S230 T2 ; set nozzle temperature`
- Exact default-disabled ooze-prevention expected startup lines for the same temperatures and no `ooze_prevention` key are:
  - `M104 S210 T0 ; set nozzle temperature`
  - `M104 S230 T1 ; set nozzle temperature`
  - `M104 S240 T2 ; set nozzle temperature`
- Exact RepRapFirmware expected startup lines for `gcode_flavor = "reprapfirmware"`, `nozzle_diameter = [0.4, 0.6]`, `nozzle_temperature_initial_layer = [210, 230]`, `ooze_prevention = true`, `idle_temperature = [0, 0]`, and `standby_temperature_delta = -15` are:
  - `G10 S210 P0 ; set nozzle temperature`
  - `G10 S215 P1 ; set nozzle temperature`
- Exact short-vector fallback expected startup lines for `nozzle_diameter = [0.4, 0.6, 0.8]`, `nozzle_temperature_initial_layer = [210, 230]`, `ooze_prevention = true`, `idle_temperature = [0, 180]`, and `standby_temperature_delta = -10` are:
  - `M104 S210 T0 ; set nozzle temperature`
  - `M104 S180 T1 ; set nozzle temperature`
  - `M104 S200 T2 ; set nozzle temperature`
- Exact non-positive inactive temperature expected startup lines for `nozzle_diameter = [0.4, 0.6]`, `nozzle_temperature_initial_layer = [5, 5]`, `ooze_prevention = true`, `idle_temperature = [0, 0]`, and `standby_temperature_delta = -10` are:
  - `M104 S5 T0 ; set nozzle temperature`
  - no `T1` startup nozzle temperature command before `;LAYER_CHANGE`
- Signed delta tests include `standby_temperature_delta = -10` as valid and at least one invalid non-integer value that errors with `standby_temperature_delta` in the message.
- Adjacent temperature tests pass with `cargo nextest run -p ares-core nozzle_temperature_gcode other_layer_temperature_gcode`.
- Full verification passes:
  - `cargo fmt --check`
  - `cargo nextest run --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `git diff --check`
  - touched Rust LOC guard, keeping every touched Rust file at or below 400 LOC.

## Safety

The implementation stays inside `ares-core`, uses existing in-memory `SliceOptions` parsing, performs no file I/O, UI, terminal, OpenGL, or platform-specific operations, and remains compatible with WASM, Windows, macOS, and Linux.
