# Consume default_junction_deviation G-code Design

## Scope

Consume the existing OrcaSlicer `default_junction_deviation` option in concrete Ares G-code output. This is a narrow `libslic3r` rewrite slice: when the configured flavor is Marlin Firmware and the configured junction deviation is positive, Ares must emit Orca-style `M205 J...` on the first layer before generated toolpath movement commands.

This is not a new option-registration milestone and does not add a new Ares motion model. It wires an already registered Orca option into the existing G-code writer path.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1052-1060` declares the jerk option group and `default_junction_deviation` as `ConfigOptionFloat` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3178-3186` defines `default_junction_deviation` with label `Junction Deviation`, units `mm`, minimum `0`, maximum `0.3`, and default `0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4628-4640` consumes `default_junction_deviation` on the first layer only, after first-layer acceleration and jerk handling, and only when `GCodeWriter` is using `gcfMarlinFirmware`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:53-57` initializes the writer's max junction deviation from `machine_max_junction_deviation` when machine limits are available.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:352-367` emits `M205 J{value}` only for Marlin Firmware, only when the writer max junction deviation and requested junction deviation are positive, clamps requested deviation to the machine maximum, formats with three fixed decimals, and appends `; Junction Deviation` when full G-code comments are enabled.

## Current Ares State

- `crates/ares-core/src/options/acceleration.rs` parses `default_jerk` and role jerk options, but not `default_junction_deviation`.
- `crates/ares-core/src/options/machine_limits.rs` already parses `machine_max_junction_deviation` into `MachineLimits::max_junction_deviation`, with default `0.01`.
- `crates/ares-core/src/gcode_machine_limits.rs` already emits machine-limit `M205 J...` for Marlin Firmware when machine limits are emitted. This slice must not change that machine envelope behavior.
- `crates/ares-core/src/gcode_writer.rs` emits dynamic XY jerk as `M205 X... Y...`, but has no writer method for default junction deviation.
- `crates/ares-core/src/gcode.rs` already has the first-layer loop, but the file is near the repository 400 LOC limit.
- `crates/ares-core/src/gcode_layer_markers.rs` is a small first-layer hook point already called before generated movement commands.

## Required Behavior

1. Add a runtime parser for `default_junction_deviation`.
2. Missing `default_junction_deviation` defaults to `0`.
3. Accept JSON numbers and numeric strings.
4. Reject non-numeric, negative, non-finite, and values above Orca's max `0.3` with `SliceError::InvalidInput` mentioning `default_junction_deviation`.
5. Add a `GCodeWriter` method for junction deviation that:
   - emits only for `GCodeFlavor::MarlinFirmware`;
   - emits only when `machine_max_junction_deviation > 0` and requested `default_junction_deviation > 0`;
   - clamps the requested value to `machine_max_junction_deviation`;
   - formats the emitted value with exactly three decimals, matching Orca `std::fixed << std::setprecision(3)`;
   - appends `; Junction Deviation` only when `gcode_comments` is true.
6. Emit the first-layer default junction deviation once per generated G-code, after layer markers/custom layer hooks and before the first generated toolpath movement command in the layer move loop. Existing layer Z travel such as `G1 Z...` may still appear earlier because Ares emits it before the layer marker hook.
7. Preserve existing dynamic XY jerk behavior and existing machine envelope `M205 J...` behavior.
8. Preserve non-Marlin Firmware behavior: no `default_junction_deviation` output for every other Ares `GCodeFlavor` variant: Marlin Legacy, RepRapFirmware, Klipper, Repetier, RepRap Sprinter, Teacup, MakerWare, Sailfish, Mach3, Machinekit, Smoothie, and NoExtrusion.

## Destination Boundary

- `crates/ares-core/src/options/acceleration.rs`: add the runtime parser and an inherent `SliceOptions::default_junction_deviation()` accessor; do not expand `crates/ares-core/src/options.rs` unless it is split first.
- `crates/ares-core/src/gcode_writer/junction_deviation.rs`: add the writer method for clamped Marlin `M205 J...`; `crates/ares-core/src/gcode_writer.rs` should only register the child module.
- `crates/ares-core/src/gcode_junction_deviation.rs`: own the first-layer command helper that parses the needed Ares options and calls `GCodeWriter`.
- `crates/ares-core/src/gcode_layer_markers.rs`: call the helper from the existing per-layer marker hook, after the existing layer marker commands and before the layer move loop emits generated toolpath commands, so `crates/ares-core/src/gcode.rs` does not grow past the 400 LOC limit.
- `crates/ares-core/src/lib.rs`: register `mod gcode_junction_deviation;` next to the other `gcode_*` modules.
- `crates/ares-core/src/tests/jerk_gcode.rs` or a focused new test file: add runtime G-code tests for the behavior.
- `crates/ares-core/src/options/tests/jerk.rs` and a focused `crates/ares-core/src/gcode_writer/tests/junction_deviation.rs`: add parser/writer tests without pushing touched Rust files past 400 LOC.

## Writer Contract

Add `GCodeWriter::set_junction_deviation(&self, junction_deviation: f64, max_junction_deviation: f64, comments_enabled: bool) -> String`. The writer does not own new machine-limit state in this slice; the caller passes the requested `default_junction_deviation`, parsed `machine_max_junction_deviation`, and parsed `gcode_comments` flag. The method returns an empty string for unsupported flavor or non-positive inputs, clamps the requested value to the passed machine maximum, emits `M205 J{value:.3}\n`, and appends ` ; Junction Deviation` before the newline when `comments_enabled` is true.

## Non-Goals

- Do not change the option registry.
- Do not add machine junction-deviation state to `GCodeWriter`; pass the parsed machine maximum into the writer method.
- Do not change the existing machine envelope `M205 J...` output from `emit_machine_limits_to_gcode`.
- Do not port Orca calibration-mode cornering logic from `GCode.cpp:4614-4623`.
- Do not port combined `set_accel_and_jerk`, Klipper square-corner-velocity behavior, Repetier-specific jerk behavior, BBL Z/E suffixes, or short-travel special cases.
- Do not add dependencies, public API, UI behavior, filesystem access, terminal behavior, or platform-specific behavior.

## Docs Impact

No roadmap or architecture docs update is required because this is a narrow source-cited runtime wiring slice that consumes an already registered Orca option without changing milestones, public API, crate boundaries, or architecture decisions.

## Acceptance Criteria

- Focused parser tests prove default `0`, numeric and numeric-string parsing, rejection of invalid values, and rejection above `0.3`.
- Focused writer tests prove Marlin Firmware emits `M205 J...`, clamps to the parsed machine maximum, suppresses unsupported flavors, suppresses zero machine maximum, suppresses zero requested value, and appends `; Junction Deviation` only when comments are enabled.
- A focused G-code runtime test proves `default_junction_deviation` reaches generated Marlin Firmware G-code before the first generated toolpath movement command after the `;MOVE:...` marker, while allowing earlier layer setup movement such as `G1 Z...`.
- A focused G-code runtime test proves the output is clamped by `machine_max_junction_deviation`.
- A focused G-code runtime test proves non-Marlin Firmware output does not contain the dynamic `default_junction_deviation` line.
- Existing XY jerk tests continue to pass.
- Verification uses `cargo nextest run`, not `cargo test`.
- `cargo fmt --check`, targeted nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC checks pass before commit.
