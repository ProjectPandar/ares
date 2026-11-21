# Layer labels before a wipe-tower event

Native at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` owns this ordering in
`GCode.cpp:7507–7558` (`GCode::process_layer`). The tower branch runs the
traditional timelapse insert before the first eligible tower call, not
necessarily before an actual filament change. Test for a non-empty call
using `WipeTowerIntegration::is_empty_wipe_tower_gcode` (`:2142–2166`): no
tower layer means no call, the special first wall qualifies, otherwise a
needed change or layer finish must have a result that isn't skipped.
Combined-away sparse layers (`WipeTower.cpp:77–86`, queried at
`GCode.cpp:2148`) are not represented by the Ares cursor and remain deferred.

In `gcode_emit/layers/drive.rs`, reuse the pre-change timelapse logic from
`tool_transition.rs` for that boundary. Synchronize the inline insertion
flag, and skip if already inserted or deferred to the inline photo head.
For two nozzles, use the physical photo head predicate (inverted for case
B); for other counts Native's tower branch defaults to inserting. Retract,
flush both pending object labels, then insert the layer wrapper even if
the timelapse template is empty (`GCode.cpp:6499–6539`). Mark the insert so
it cannot repeat at layer end. After that check, run the existing wrapping
detection at most once per layer, before the same eligible tower call. The
non-tower path keeps its real dual-nozzle change requirement.

Compare both derived X2D inputs against Native through the API and CLI:
enable the tower, then also test enable plus 33 mm tower width. Compare the
ordered first entry between the model wipe and tower start, including the
object stop, layer wrapper, acceleration and travel. Verify the layer
wrapper doesn't repeat at the end. A strict whole-stream comparison remains
a diagnostic if unrelated output differs. Tower paths, the compacted tower
planner, and print-time estimation are outside this slice.
