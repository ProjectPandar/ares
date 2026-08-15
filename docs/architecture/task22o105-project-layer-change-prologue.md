# Task 22O.105: Project layer-change prologue

## Upstream boundary

This slice ports the executable prologue around `OrcaSlicer/src/libslic3r/GCode.cpp`'s
initial filament-start handling and layer loop, together with `GCodeWriter.cpp`
`GCodeWriter::preamble()`. The Ares destination is
`project_slice::gcode_emit`, after typed project configuration has been resolved.

## Included

- Emit the source-cited relative-E/preamble commands for the initial Bambu-style
  project start.
- Emit layer height markers and evaluate typed `layer_change_gcode` placeholders
  for each resolved layer.
- Preserve the project placeholder parser as the only configuration source.

## Deferred

- Orca's `GCodeProcessor` timing/M73 insertion and motion feedrate state.
- Full `GCodeWriter` travel, retract, acceleration, role comments, and machine
  end handling.
- Exact first-layer convex-hull translation for adaptive bed leveling.

The existing Ares emitter remains a temporary compatibility shell until these
upstream G-code boundaries are ported; no fixture-specific output is introduced.
