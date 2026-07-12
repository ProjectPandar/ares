# Input Shaping G-code Runtime Design

## Goal

Consume the already-registered `input_shaping_*` MachineEnvelopeConfig options in generated G-code, before adding more option metadata. This is a source-cited Rust rewrite slice of OrcaSlicer's machine-envelope input-shaping output, not a new Ares pipeline feature.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:365-379` defines `InputShaperType` values: `Default`, `MZV`, `ZV`, `ZVD`, `ZVDD`, `ZVDDD`, `EI`, `EI2`, `TwoHumpEI`, `EI3`, `ThreeHumpEI`, `DAA`, and `Disable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:503-518` maps serialized option strings to those enum values. The runtime strings accepted by this slice are exactly `Default`, `MZV`, `ZV`, `ZVD`, `ZVDD`, `ZVDDD`, `EI`, `EI2`, `2HUMP_EI`, `EI3`, `3HUMP_EI`, `DAA`, and `Disable`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1281-1287` places `input_shaping_emit`, `input_shaping_type`, `input_shaping_freq_x`, `input_shaping_freq_y`, `input_shaping_damp_x`, and `input_shaping_damp_y` inside `MachineEnvelopeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4541-4585` defines defaults and bounds: emit defaults false, type defaults `Default`, frequencies default 0 with min 0 and max 1000, damping defaults 0.1 with min 0 and max 1.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3883-3944` emits machine-envelope G-code for MarlinLegacy, MarlinFirmware, and RepRapFirmware when `emit_machine_limits_to_gcode` is true, and then writes input-shaping overrides only when `input_shaping_emit` is true and the flavor is not MarlinLegacy.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:395-477` formats `set_input_shaping`: `Disable` forces all-axis zero frequency/damping, RepRapFirmware emits `M593`, MarlinFirmware emits `M593`, and Klipper formatting exists in the writer but is outside the normal `GCode::print_machine_envelope` branch for this slice.

## Rust Destination Boundary

- Add runtime option parsing under `crates/ares-core/src/options/` for a small `InputShapingConfig`.
- Add generated G-code formatting under a new `crates/ares-core/src/gcode_input_shaping.rs` module to keep `gcode_writer.rs` below the 400 LOC project guard.
- Wire the formatter from `crates/ares-core/src/gcode.rs` immediately after `gcode_machine_limits::format_machine_envelope`, matching the upstream machine-envelope placement before startup temperature and custom start G-code.
- Add focused integration tests in `crates/ares-core/src/tests/input_shaping_gcode.rs`.

## Included Behavior

- Default `input_shaping_emit = false` emits no input-shaping G-code.
- Disabled `emit_machine_limits_to_gcode` suppresses the input-shaping machine-envelope output for this ordinary slicing path.
- MarlinLegacy suppresses input-shaping output even when `input_shaping_emit` is true.
- MarlinFirmware (`gcode_flavor = "marlin2"`) emits X and Y `M593` commands when enabled and not disabled:
  - X always includes the `X` axis parameter and the comment.
  - Y always includes the `Y` axis parameter and the comment.
  - Frequency is emitted as `F<value>` only when the axis frequency is greater than 0.
  - Damping is emitted as `D<value>` only when the axis damping ratio is greater than 0.
  - When an enabled non-disable axis has zero frequency and zero damping, the upstream output is still an axis-only line such as `M593 X ; Override input shaping`.
  - `input_shaping_type` does not appear in MarlinFirmware output, matching upstream `GCodeWriter::set_input_shaping`.
- RepRapFirmware emits at most one `M593` command from the X values and does not emit a separate Y command in the ordinary machine-envelope path.
- RepRapFirmware includes `P"<type>"` when the type is neither `Default` nor `DAA`, `F<value>` only when `input_shaping_freq_x` is greater than 0, and `S<value>` only when `input_shaping_damp_x` is greater than 0.
- RepRapFirmware emits no input-shaping line for enabled non-disable configs where the type is `Default` or `DAA` and both X frequency and X damping are zero.
- RepRapFirmware and MarlinFirmware parsing accepts serialized profile strings only. In particular, `2HUMP_EI` and `3HUMP_EI` are accepted, while C++ identifier spellings `TwoHumpEI` and `ThreeHumpEI` are rejected as invalid profile values.
- `input_shaping_type = "Disable"` emits an all-axis disable command with zero frequency and damping:
  - MarlinFirmware: `M593 F0.00 D0.000 ; Override input shaping`
  - RepRapFirmware: `M593 F0.00 S0.000 ; Override input shaping`
- Values must be finite. Frequencies must be in `[0, 1000]`; damping must be in `[0, 1]`.
- Unknown input shaper type strings are rejected with `SliceError::InvalidInput`.

## Deferred Behavior

- Klipper input-shaping startup commands are deferred because the cited ordinary `GCode::print_machine_envelope` branch does not call `set_input_shaping` for Klipper.
- Calibration-mode input-shaping output in `OrcaSlicer/src/libslic3r/GCode.cpp:4585-4611` is deferred.
- Per-flavor UI filtering from `get_shaper_type_values_for_flavor` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:95-111` is deferred; this slice validates the enum values that already exist in Ares metadata.
- New option registry metadata, new public API, time estimation, viewer data, input-shaping physics, and firmware probing are out of scope.

## Existing Ares Scaffolding

Ares already records the `InputShaperType` enum and `input_shaping_*` MachineEnvelopeConfig option metadata, and those metadata tests explicitly defer runtime behavior. This slice consumes those existing option keys in generated G-code without adding more option metadata.

## Acceptance Criteria

- Focused tests prove default and disabled cases produce no input-shaping G-code.
- Focused tests prove MarlinFirmware emits X/Y `M593` commands after machine limits and before startup temperature/custom start output.
- Focused tests prove MarlinFirmware enabled non-disable zero frequency/damping still emits axis-only X/Y commands.
- Focused tests prove RepRapFirmware emits a single `M593` from X values when at least one parameter is present, omits a separate Y command, and emits no `M593` for enabled non-disable `Default`/`DAA` with zero X frequency and damping.
- Focused tests prove `Disable` emits the all-axis zero override.
- Focused tests prove serialized enum values `2HUMP_EI` and `3HUMP_EI` are accepted where relevant, and `TwoHumpEI` / `ThreeHumpEI` are rejected before output bytes are returned.
- Focused tests prove invalid frequency, damping, boolean, and enum values are rejected before output bytes are returned.
- Verification uses `cargo nextest run` rather than `cargo test`, plus `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust 400 LOC guard.
