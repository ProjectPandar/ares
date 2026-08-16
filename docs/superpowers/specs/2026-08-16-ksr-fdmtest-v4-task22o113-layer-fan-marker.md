# Spec: Task 22o.113 layer-change fan-speed marker

## Observable contract

For every emitted project layer, Ares appends `;_SET_FAN_SPEED_CHANGING_LAYER` immediately after the rendered `layer_change_gcode` template and before the layer's motion acceleration. The template receives its configured trailing newline and Orca's additional separator newline before the marker. The KSR fixture emits the marker exactly 460 times.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:4633-4688`: `GCode::process_layer` renders the typed `layer_change_gcode`, appends the source separator newline, and then emits the Bambu layer-time fan-speed marker. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit.rs::append_layer_change`.

## Included behavior

- Preserve the 3MF-provided layer-change template output.
- Append the source separator newline only when a template was rendered.
- Emit the fan-speed marker for every layer, including an empty-template project.

## Deferred behavior

Calibration-mode layer G-code and downstream fan-time rewriting remain separate source-cited slices.
