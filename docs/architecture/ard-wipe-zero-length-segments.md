# ARD: retain source wipe segments

Owner: `GCode.cpp:440-488 Wipe::wipe`, `Polyline.hpp:89-95` iterator append,
`Polyline.cpp:22-31 lines`, `GCodeWriter.cpp:933-952 extrude_to_xy`. Destination:
`gcode_emit/motion/travel/wipe_path.rs::wipe_moves`. Replace the segment-filter
approximation, not the motion pipeline. Retain every clipped adjacent pair,
including zero-length terminal segments. The existing writer emits XY without
E for zero retraction; do not synthesize printer-specific commands.

Match source append-junction dedup (only the first appended equal point) before
clipping, and source EPSILON=1e-4 denominator floor for a shortened zero-length
path. No clipping arithmetic, writer, wipe-tower, cooling, RNG or shared bridge
changes. Those adjacent behaviors remain deferred.

Public full-output RED before implementation; frozen evidence read-only. Keep
all comparisons generator-line-only, no deleted assertions/probes/suppressions.
