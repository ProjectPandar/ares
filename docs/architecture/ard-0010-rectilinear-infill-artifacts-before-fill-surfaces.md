# ARD-0010: Rectilinear infill artifacts before fill surfaces

## Decision
Ares will add sparse rectilinear infill as deterministic path artifacts clipped to current simple contours before porting OrcaSlicer fill surfaces, shell classification, offsets, and extrusion planning.

## Context
OrcaSlicer routes infill through `libslic3r/Fill/*`, `Surface`, `ExPolygon`, and region-specific extrusion roles. Ares currently has STL import, layer planning, XY segments, stitched contours, pipeline diagnostics, and external perimeter artifacts, but not offset polygons or fill-surface classification.

## Consequences
- M11 can validate API shape, option typing, diagnostics, and deterministic path output without pretending to match full Orca infill semantics.
- The implementation must not classify solid/top/bottom infill or generate extrusion E values.
- Later milestones must replace or extend the artifact generator when fill surfaces, offsets, holes, and extrusion roles are ported.
