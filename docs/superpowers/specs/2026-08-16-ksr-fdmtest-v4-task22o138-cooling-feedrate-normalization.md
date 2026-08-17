# Spec: Task 220.138 cooling-buffer redundant feedrates

## Observable contract

Each generated layer is passed through feedrate normalization. For G0/G1 commands containing an `F` word, the decimal value is compared using the source integer-feedrate semantics against the current feedrate within that layer. A repeated standalone feedrate command is removed; a repeated feedrate attached to a motion command loses only its `F` word. A changed feedrate is retained and becomes current.

In the first KSR dynamic inner-wall block, `G1 F15791.926` appears once rather than once per fully supported segment. The rounded `G1 F15780` transition and all extrusion moves remain.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode/CoolingBuffer.cpp:822-823,899-963` resets `current_feedrate` for a processed layer and removes redundant feedrate-only lines or words while rewriting cooling output. Fan-marker resolution and minimum-layer-time slowdown are deferred.
