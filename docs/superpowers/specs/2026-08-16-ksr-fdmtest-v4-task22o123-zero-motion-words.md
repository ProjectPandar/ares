# Spec: Task 22o.123 valid zero-valued motion words

## Observable contract

Every numeric word emitted by the Ares motion writer contains a numeric value. Exact zero is emitted as `0`; relative extrusion and arc offsets retain Orca's omitted leading zero for nonzero fractional values. KSR spiral lifts therefore emit forms such as `I0 J-1.217`, never bare `I` or `J` words.

## Upstream boundary

This ports the zero handling of the numeric formatting used by `OrcaSlicer/src/libslic3r/GCodeWriter.cpp`. It applies only to generated motion; custom 3MF G-code remains byte-preserved after template rendering.
