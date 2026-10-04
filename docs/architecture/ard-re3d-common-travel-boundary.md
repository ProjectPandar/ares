# ARD: common travel boundary selection

Owner: OrcaSlicer v2.4.2 `GCode/AvoidCrossingPerimeters.cpp:190-216`,
`extend_for_closest_lines::endpoints_close_to_same_boundary`. Destination:
`gcode_emit/motion/path/avoid_crossing/router/closest_lines.rs`.

Collect every start candidate contour, scan end candidates in source order,
then choose the first matching start candidate. This is an upstream rewrite
slice replacing the first-start-only compatibility approximation, not a new
travel planner. Preserve source membership-set complexity and candidate order.
Do not change geometry, spacing, intersection arithmetic, wipe, timing, shared
bridge code or fallback branches. Those adjacent behaviors are deferred.

Frozen stock remains read-only. Public complete G-code RED before implementation;
strict generator-line-only comparison, no timing filtering for acceptance, no
assertion removal or probes. Six random startup-template cases remain deferred.
