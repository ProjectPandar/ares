# Spec: Task 220.137 rounded dynamic-overhang base speed

## Observable contract

Every speed produced by dynamic overhang evaluation is rounded to an integral millimetres-per-second value, including the fully supported branch that selects the role/volumetric-limited original speed. The emitter may subsequently restore the precise G-code feedrate when the rounded value is within the source one-millimetre-per-second threshold.

For the first KSR dynamic inner wall, the initial feedrate commands are `G1 F15780` followed by `G1 F15791.926`.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode/ExtrusionProcessor.hpp:426-448` applies `round(final_speed)` after every interpolation branch and clamps it to `original_speed`. `src/libslic3r/GCode.cpp:7111-7124,7202-7210` emits the rounded initial speed and precise feedrate restoration.
