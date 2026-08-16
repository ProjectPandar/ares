# Spec: Task 22o.119 layer cooling fan transition

## Observable contract

The KSR project emits `M106 S255` and `M106 P2 S178` immediately before the second `CHANGE_LAYER` block. The values come from `close_fan_the_first_x_layers`, `fan_max_speed`, `additional_cooling_fan_speed`, `auxiliary_fan`, and `part_cooling_fan_min_pwm` loaded from the 3MF.

## Upstream boundary

Rewrite `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:733-824`, limited to layer-index fan enablement, full-speed-layer ramping, PWM conversion, and command deduplication. Layer-time interpolation, cooling slowdown, and role fan markers remain deferred.
