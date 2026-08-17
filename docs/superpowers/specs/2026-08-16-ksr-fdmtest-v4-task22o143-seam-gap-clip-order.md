# Spec: Task 22O.143 seam-gap clip ordering

## Observable contract

When loop clipping and arc fitting are enabled, Ares clips the unsimplified split loop by the 3MF-resolved absolute `seam_gap` before fitting or simplifying its emitted paths. The first KSR inner wall ends exactly at `G1 X140.174 Y102.761 E.02841`; its start travel and the following outer-wall travel remain exact.

The endpoint derives from generated loop geometry and typed `seam_gap`, nozzle diameter, and `enable_arc_fitting` options. Production code does not inspect fixture identity or reference G-code.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:5793-5805`, `src/libslic3r/ExtrusionEntity.cpp:317-331`, and `src/libslic3r/Polyline.cpp:683-723`. `ExtrusionLoop::clip_end` runs before each clipped path enters `_extrude`, where arc fitting and simplification occur. Arc clipping metadata, scarf seams, and remaining G-code differences remain deferred where the KSR option path does not exercise them.
