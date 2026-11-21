# Type1 compacted tower exit

Port the exit of `WipeTowerIntegration::append_tcr` in OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode.cpp:1640–1660`). The phony XY move and last-position
update happen at every exit. When the tower result Z differs from the
entry writer Z by the existing `is_approx` test, the writer retracts,
returns to the entry Z, and unretracts, without setting a future wipe path.
Only the same-Z arm replaces the wipe path with the transformed tower
points and calls the generator retract. Both arms still request external
travel planning.

The compacted Z already comes from non-sparse tower layers, as in
`GCode/WipeTower.cpp:53–75` and `GCode.cpp:2114–2124`. Keep the current
planning, deferred descent, and Z comparison. In Ares,
`project_slice/gcode_emit/wipe_tower/type1/emit.rs` owns the branch;
`project_slice/gcode_emit/motion/tower.rs` provides the writer and
wipe bookkeeping operations. Separate the shared position update from the
wipe update so the branch can invoke only the upstream operations it needs.
Do not change Type2, the pressure advance and toolchange templates, or
progress-time calculation.

Use a synthetic two-object, two-slot X1C project with skipped sparse tower
layers. Require the Native output to exhibit a retract/Z return/unretract
and compare the following object-entry retract and travel through the next
feature marker through public slice output. Also compare whole Native and
Ares streams in official and seeded modes using only generator-line
normalization; a progress mismatch remains a whole-stream difference.
