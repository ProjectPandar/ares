# Consume Num Extruders Placeholder Design

## Goal

Consume the existing Orca `num_extruders` custom G-code placeholder as concrete Ares `machine_start_gcode` output, using the effective `nozzle_diameter` vector length that Ares already parses.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/GCode.cpp`, in the print start G-code preparation block after `m_placeholder_parser_integration.init(m_writer)`, registers global custom G-code placeholders before start G-code rendering, including `total_layer_count`, `num_extruders`, and `retract_length`.
- `OrcaSlicer/src/libslic3r/GCode.cpp` sets `num_extruders` with the exact statement `this->placeholder_parser().set("num_extruders", int(print.config().nozzle_diameter.values.size()));`.
- `OrcaSlicer/src/libslic3r/GCode.cpp` processes and writes `machine_start_gcode` with `placeholder_parser_process("machine_start_gcode", print.config().machine_start_gcode.value, initial_extruder_id)` before `file.writeln(machine_start_gcode)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp`, in the custom G-code placeholder definitions, defines `num_extruders` as a read-only `coInt` placeholder with label "Number of extruders" and tooltip "Total number of extruders, regardless of whether they are used in the current print."
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp`, in `s_CustomGcodeSpecificPlaceholders`, gives `machine_start_gcode` an empty scope-specific placeholder set, so it can use the global custom placeholder set.

## Ares Destination Boundary

- `crates/ares-core/src/gcode.rs` already resolves `hardware_options = options.hardware_options()?`; it will pass `hardware_options.nozzle_diameters().len()` into the start G-code command.
- `crates/ares-core/src/gcode_start_custom.rs` will carry `num_extruders` through `StartGCodeCommand`.
- `crates/ares-core/src/gcode_adaptive_bed_mesh.rs` will forward the value to machine-start placeholder rendering while preserving adaptive bed mesh rendering.
- `crates/ares-core/src/gcode_placeholders.rs` will replace `[num_extruders]` only in `machine_start_gcode`.
- `crates/ares-core/src/tests/num_extruders_gcode.rs` will cover the runtime G-code behavior with `cargo nextest run`.

## Included Behavior

- `machine_start_gcode` containing `[num_extruders]` renders the effective nozzle count before the first `;LAYER_CHANGE`.
- The effective count is `HardwareOptions::nozzle_diameters().len()`, matching Orca's source boundary of `nozzle_diameter.values.size()`.
- Missing `nozzle_diameter` uses Ares' existing Orca-compatible default hardware options and renders `1`.
- `[num_extruders]` composes with the existing `[total_layer_count]` placeholder in `machine_start_gcode`.
- `[num_extruders]` remains literal in `layer_change_gcode`; this slice does not widen custom G-code scope.

## Deferred Behavior

- `initial_tool`, `initial_extruder`, `current_extruder`, hotend mapping, support/non-support initial tools, `is_extruder_used`, `retract_length`, wipe tower, tool changes, multi-extruder scheduling, and multi-material priming are deferred.
- Bracket indexing, expression parsing, full Orca `PlaceholderParser` parity, and expansion in non-machine-start scopes are deferred.
- A new Ares-owned `num_extruders` input option is not added; this slice follows the upstream `nozzle_diameter` vector length boundary.

## Acceptance Criteria

- With `machine_start_gcode = ";EXTRUDERS [num_extruders]"` and `nozzle_diameter = ["0.4", "0.6", "0.8"]`, slicing emits `;EXTRUDERS 3` before the first layer marker.
- With `machine_start_gcode = ";START [num_extruders] [total_layer_count]"` and two nozzle diameters, slicing emits `;START 2 2` for the square-pyramid fixture.
- With no explicit `nozzle_diameter`, slicing emits `;DEFAULT-EXTRUDERS 1`.
- With `layer_change_gcode = ";LC [num_extruders] [layer_num]"`, slicing keeps `[num_extruders]` literal while still rendering `[layer_num]`.

## Verification Plan

- Write failing runtime tests first in `crates/ares-core/src/tests/num_extruders_gcode.rs`.
- Run `cargo nextest run -p ares-core num_extruders` and confirm the machine-start cases fail because `[num_extruders]` is not yet replaced.
- Implement the minimal value propagation and replacement.
- Run `cargo nextest run -p ares-core num_extruders` and confirm the new tests pass.
- Before completion, run `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the touched Rust LOC guard.

## Docs Impact

- Update `docs/roadmap.md` with a short runtime-slice note after implementation.
- No user-facing usage documentation is required because this consumes an existing Orca custom G-code placeholder inside existing `machine_start_gcode` behavior.

## Safety

The change stays inside `ares-core`'s existing in-memory slicing and G-code formatting path. It adds no file I/O, terminal behavior, UI behavior, OpenGL behavior, dependencies, feature flags, or compatibility fallback.
