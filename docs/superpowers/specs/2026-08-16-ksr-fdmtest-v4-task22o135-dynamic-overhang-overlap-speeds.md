# Spec: Task 220.135 dynamic overhang overlap speeds

## Observable contract

When `enable_overhang_speed` is active above the first layer, perimeter and bridge paths are evaluated against the preceding layer boundary. Boundary crossings and source segmentation points split a path; each segment resolves its speed by interpolating the loaded 90/75/50/25/13/0-percent overlap bands, rounding to whole millimetres per second, and clamping to the option- and volumetric-limited base speed.

For the first KSR overhang transition, Ares emits the source sequence ending the inner-wall path with `G1 F3000`, `G1 X114.789 Y81.637 E.02836`, `G1 F1980`, and `G1 X114.989 Y81.637 E.00663`; the following inner-wall segment also starts at `G1 F1980`. Values come from the previous 3MF layer geometry and effective `outer_wall_speed`, `inner_wall_speed`, `bridge_speed`, `overhang_1_4_speed` through `overhang_4_4_speed`, `filament_max_volumetric_speed`, and `enable_overhang_speed` options.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode/ExtrusionProcessor.hpp:37-217,315-459` owns signed previous-boundary distance sampling, boundary intersections, path segmentation, speed-band interpolation, rounding, and overlap result production. `src/libslic3r/GCode.cpp:6654-6715,7111-7210` constructs option-derived speed bands and emits variable-speed linear segments. Curled-line proximity adjustment is deferred because the KSR 3MF has `slowdown_for_curled_perimeters = 0`; fan-marker post-processing remains a later slice.
