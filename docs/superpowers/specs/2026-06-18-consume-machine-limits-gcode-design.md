# Consume Machine Limits G-code Design

## Goal

Implement a source-cited OrcaSlicer rewrite slice that consumes Ares' existing machine limit options and emits Orca-style machine envelope G-code during startup. This slice must move beyond option metadata by making `emit_machine_limits_to_gcode` and the `machine_max_*` values affect generated G-code.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1247-1274` declares `MachineEnvelopeConfig` options for `emit_machine_limits_to_gcode`, axis max accelerations, axis max speeds, role max accelerations, axis jerk limits, junction deviation, and minimum rates.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4326-4332` registers `emit_machine_limits_to_gcode`, defaults it to `true`, and documents that Klipper ignores it.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4386-4514` registers machine envelope default vectors: max speeds, max accelerations, max jerks, junction deviation, minimum rates, and role accelerations.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2784` calls `GCode::print_machine_envelope(...)` before fan shutdown and startup custom G-code.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3883-3933` emits machine envelope commands when flavor is Marlin Legacy, Marlin Firmware, or RepRapFirmware and `emit_machine_limits_to_gcode` is `true`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:352-365` emits `M205 J...` only for Marlin Firmware when machine max junction deviation and requested junction deviation are both positive, clamping to the configured maximum.

## Ares Destination Boundary

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs` and `crates/ares-core/src/options/registry/definitions/table/late.rs` already store metadata/defaults for the machine envelope keys. This slice will consume those option keys at runtime without adding more metadata-only milestones.
- `crates/ares-core/src/options.rs` is already at the 400 LOC threshold, so runtime parsing will live in a new focused options module instead of expanding the monolithic file.
- `crates/ares-core/src/gcode.rs` owns startup G-code ordering. It will call a new machine-envelope formatter immediately after `writer.preamble()`, matching the current Ares startup structure and the upstream pre-start-script placement.
- A new focused G-code formatter module will own machine envelope command formatting. It will not introduce file I/O, terminal behavior, OpenGL, UI behavior, or a new Ares-owned slicing pipeline.

## Included Behavior

- Missing `emit_machine_limits_to_gcode` defaults to `true`.
- `emit_machine_limits_to_gcode = false` suppresses all machine envelope output.
- Only `GCodeFlavor::MarlinLegacy`, `GCodeFlavor::MarlinFirmware`, and `GCodeFlavor::RepRapFirmware` emit envelope output. `Klipper` and `Repetier` emit nothing.
- Machine limit numeric options accept JSON numbers, numeric strings, numeric arrays, and semicolon/comma-separated numeric strings through the existing numeric vector parser.
- The first value of each vector is consumed, matching upstream `.values.front()`.
- Missing machine envelope options use Orca defaults already recorded in the Ares registry:
  - `machine_max_acceleration_x/y/z/e`: `1000/1000/500/5000`
  - `machine_max_speed_x/y/z/e`: `500/500/12/120`
  - `machine_max_acceleration_extruding/retracting/travel`: `1500/1500/0`
  - `machine_max_jerk_x/y/z/e`: `10/10/0.2/2.5`
  - `machine_max_junction_deviation`: `0.01`
- Marlin Legacy and Marlin Firmware emit speeds and jerks in `mm/sec`; RepRapFirmware multiplies speed and jerk values by `60` and labels jerk as `mm/min`, matching upstream `factor`.
- `M201` rounds acceleration values as `int(value + 0.5)`.
- `M203` rounds speed values as `int(value * factor + 0.5)`.
- `M204` flavor handling follows upstream:
  - Marlin Legacy: `M204 P... R... T...` with travel acceleration equal to extruding acceleration.
  - Marlin Firmware: `M204 P... R... T... ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2`.
  - RepRapFirmware: `M204 P... T... ; sets acceleration (P, T), mm/sec^2`.
- Jerk output follows upstream:
  - Marlin Legacy and Marlin Firmware: `M205 X... Y... Z... E... ; sets the jerk limits, mm/sec`.
  - RepRapFirmware: `M566 X... Y... Z... E... ; sets the jerk limits, mm/min`.
- `machine_max_junction_deviation` emits `M205 J...` only for Marlin Firmware and only when the first value is greater than `0`.
- Invalid machine limit values return `SliceError::InvalidInput` before any G-code bytes are returned.

## Deferred Behavior

- Input shaping override commands after the junction deviation line remain out of scope because their option model and writer behavior are not part of this machine-envelope slice.
- `machine_min_travel_rate` and `machine_min_extruding_rate` runtime output remains deferred because upstream `print_machine_envelope()` does not emit `M205 T` or `M205 S` in this path.
- Time-estimator integration is out of scope; Ares currently formats in-memory G-code and has no ported estimator hook for this envelope block.
- Additional firmware flavors beyond active Ares parsing remain out of scope.
- No new crates, dependencies, candidate workspace members, profile composition rules, or Ares-owned pipeline behavior are introduced.

## Acceptance Criteria

- A default Marlin Legacy slice emits machine envelope commands after the writer preamble and before startup temperature/custom G-code.
- A Marlin Legacy slice with custom machine limit values emits `M201`, `M203`, `M204`, and `M205` using first vector entries and upstream rounding.
- A Marlin Firmware slice emits the Marlin2 `M204 P/R/T` comment form and emits `M205 J...` when `machine_max_junction_deviation > 0`.
- A RepRapFirmware slice emits `M203` and `M566` speed/jerk values multiplied by `60` and uses the RRF `M204 P/T` form.
- Klipper and Repetier slices emit no `M201`, `M203`, machine-envelope `M204`, `M205`, or `M566` block from this feature.
- `emit_machine_limits_to_gcode = false` suppresses machine envelope output for otherwise supported flavors.
- Numeric vector parsing accepts array/string forms and rejects empty, negative, non-finite, non-numeric, boolean, null, and object values for machine envelope fields.
- No Rust file under `crates/` exceeds 400 LOC.
