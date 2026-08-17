# Spec: Task 220.139 reverse extrusion-path wipe

## Observable contract

A wipe starts at the final emitted extrusion coordinate and follows the just-printed path backwards. It must never jump to the second point of a forward-ordered path. Wipe-distance clipping and retraction allocation operate on that reversed path.

At the end of KSR layer zero, the final extrusion at `X108.329 Y93.811` wipes one millimetre backward toward the preceding `X109.102 Y94.584` endpoint, producing `G1 X109.036 Y94.518 E-.28571`.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6110-6124` stores the emitted extrusion polyline and reverses it before later retraction. `GCode.cpp:343-350` prepends the last position and clips the backward path to `wipe_distance`.
