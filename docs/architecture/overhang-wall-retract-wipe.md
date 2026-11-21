# Wipe path lifetime at wall segment boundaries

OrcaSlicer 2.5.0 emits each constituent wall path through `GCode::_extrude`, then publishes the wipe geometry for the complete entity (`GCode.cpp:8733–8768, 8854–8864`). Between constituent paths, `travel_to` uses the previously stored wipe path if the travel needs a retraction and clears it otherwise (`GCode.cpp:10568–10623, 10714–10716`). The retract sequence only considers wiping when that path is present (`GCode.cpp:10838–10865`, with distance limiting at `:538–602`).

In `project_slice/gcode_emit/motion`, keep a constituent path's wipe geometry separate from the stored path until the enclosing loop or multipath publishes it. Preserve the existing geometry, ordering and direction of the completed wipe path, and leave standalone extrusion, travel routing, retract amounts and wipe speed unchanged.

For a synthetic MK4 overhang tower, the transition from an overhang wall to the next wall can require a retract even though the preceding wall segment has not published a wipe path. Compare complete public API and CLI output against OrcaSlicer for both a zero travel minimum and wall-crossing avoidance, each independently applied to the stock project.
