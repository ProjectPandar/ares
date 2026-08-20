# Spec: KSR FDM Test V4 fitted-circle center quantization

## Observable contract

Arc fitting computes circle centers in the slicer's scaled-coordinate domain and converts them back without rounding across an integer coordinate boundary. The resulting `G2`/`G3` `I` and `J` offsets remain stable at six-decimal source geometry precision.

## Upstream boundary

Port the scaled `Point` construction semantics used by OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:15-108` and `src/libslic3r/ArcFitter.cpp:276-313` into `project_slice::gcode_emit::motion::arc::circle_from_three`. Adjacent arc fitting heuristics remain unchanged.

## Acceptance

A center whose scaled Y value is just below an integer remains below that boundary after conversion, focused arc tests pass, and the KSR slice emits the corrected fitted-arc offset without fixture-specific production branches.
