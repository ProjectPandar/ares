# Spec: KSR FDM v4 option-driven retract, wipe, and spiral lift

## Observable contract

Project G-code travel motion resolves retraction length/speeds, minimum travel, wipe enable/distance, retract-before-wipe, role-based wipe speed, z-hop type, z-hop distance, and travel slope from typed effective 3MF options. Eligible travel retracts along the prior extrusion path, emits `WIPE_START`/`WIPE_END`, performs the configured spiral lift, travels at raised Z, lowers, and deretracts. Travel below the minimum distance and non-perimeter travel wholly inside an internal surface skip retraction when `reduce_infill_retraction` is enabled.

Production behavior depends only on typed options, generated extrusion geometry, and source layer surfaces. It must not inspect fixture names, golden G-code, expected hashes, or fixture-specific coordinates.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:7358-7650`, `GCode.cpp:7652-7708`, `GCode/RetractWhenCrossingPerimeters.cpp:9-51`, and the spiral lift seam in `GCodeWriter.cpp`. The Rust destination is `project_slice::gcode_emit::motion` and its private `travel` module.

Included: minimum-travel gating, internal-surface gating, relative-E retract/deretract, reverse prior-path wipe distance, role feedrate selection, normal/spiral z-hop, and raised-Z travel. Deferred: avoid-crossing-perimeters route generation, overhang intersection selection for `Auto Lift`, tool changes, and wipe-path arc fitting.

## Acceptance

Focused tests prove effective KSR option resolution, inside/outside internal-surface gating, and emitted wipe/spiral-lift structure. The KSR CLI output contains option-derived wipe and spiral-lift blocks without fixture-specific branches; the next divergence remains generated path ordering and geometry.
