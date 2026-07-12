# ARD-0009: External perimeter paths before polygon offsets

## Status
Accepted for M10.

## Context
OrcaSlicer generates perimeters after slicing, using `PrintObject::make_perimeters`, `Layer::make_perimeters`, and `PerimeterGenerator`. The classic generator offsets surfaces inward, nests loops, emits gap fills, and prepares fill surfaces for infill.

Ares currently has deterministic simple contours but no polygon boolean or offset engine. Jumping directly to internal wall offsets would require a larger geometry milestone and risk hiding repair decisions inside perimeter generation.

## Decision
M10 introduces a perimeter artifact boundary by converting each valid contour into one external perimeter path. The pipeline and `slice` output expose those paths and counts. Internal offsets, wall-loop option typing, fill surfaces, gap fills, extrusion values, and Orca parity remain explicit later milestones.

## Consequences
- Future G-code and UI APIs can consume perimeter path artifacts without parsing contour diagnostics.
- Infill milestones get a stable boundary while offset/fill-surface semantics remain deferred.
- The core stays dependency-free, filesystem-free, and WASM-safe.
