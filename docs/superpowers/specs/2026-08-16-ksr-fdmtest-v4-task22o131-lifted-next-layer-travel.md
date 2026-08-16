# Spec: Task 220.131 lifted next-layer travel

## Observable contract

When layer-end retraction leaves the nozzle lifted, the first XY travel of the next layer stays at that already-safe physical Z. It emits the configured travel feedrate without recomputing `next_layer_z + z_hop`; deretraction then lowers to the new layer Z. For KSR layer two this is an XY-only `F60000` travel followed by `G1 Z.4`.

The move derives from emitter lift state, generated seam position, project layer height, travel speed, and Z-hop options. No fixture identity or reference coordinates enter production.

## Upstream boundary

Port the lifted travel behavior of OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6345-7047` and `src/libslic3r/GCodeWriter.cpp:587-617` into `project_slice::gcode_emit::motion::emit_points`. Layer-end lift has already established the travel Z; this slice changes only the subsequent XY move and retains the existing explicit lowering move.
