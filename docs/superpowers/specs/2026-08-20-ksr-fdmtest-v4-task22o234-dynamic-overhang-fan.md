# Spec: KSR FDM Test V4 dynamic overhang fan transitions

## Observable contract

Variable-speed perimeter segments enable the configured overhang fan when both segment endpoints satisfy `overhang_fan_threshold`, and restore the current layer part-fan speed when the segment leaves that region. The emitted PWM values come from `overhang_fan_speed`, the part-fan layer ramp, and the loaded 3MF options.

## Upstream boundary

Port the role-marker decisions in OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6845-6911, 7123-7150` and the processed overlap value in `src/libslic3r/GCode/ExtrusionProcessor.hpp:445-458` into `project_slice::gcode_emit::motion`. Cooling-buffer marker internals are not retained; Ares emits their final `M106` result at the same motion seam.

## Acceptance

The KSR project output contains an overhang-fan transition inside the first variable-speed inner-wall region, uses the project-configured 100% PWM, and restores the current layer fan speed at the next transition. Production code does not inspect fixture names or reference G-code.
