# Spec: Task 22o.120 per-layer timelapse G-code

## Observable contract

The KSR project renders its 3MF `time_lapse_gcode` after every one of 460 printed layers. Runtime placeholders produce `M9711 M0 E1 X133 Y193 Z{layer_z + 0.4} S11 C10 O0 T3000`; the first and final values are `Z0.6` and `Z92.4`.

## Upstream boundary

Rewrite `OrcaSlicer/src/libslic3r/GCode.cpp:5145-5178` and the single-object, by-layer position selection from `GCode/TimelapsePosPicker.cpp:335-451,487-526`. The safe position derives from transformed model bounds, printable area, and `extruder_clearance_radius`. By-object collision routing, multi-object aggregation, wipe-tower exclusion, and smooth-timelapse all-layer placement remain deferred.
