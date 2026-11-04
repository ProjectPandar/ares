# G4 Manual-map bounded review

## Rewrite boundary and conclusion

Upstream source pinned to `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `PrintConfig.cpp:2489-2493`, `GCode/ToolOrdering.cpp:1275-1312`: four coherent Manual maps `[1,1]`, `[1,2]`, `[2,1]`, `[2,2]`, not automatic grouping or arbitrary vector domains.
- `GCode.cpp:5289-5308,5478-5494,5527-5546`: pre-logical-toolchange traditional timelapse, label-only completion and actual layer-tail retract owner.
- `GCode.cpp:2586-2592,5255-5258,7331-7346,7775-7779,8037-8041`: source-instance sorted label masks and nonempty filament-template M624/M625 consumers.

Rust destinations are `project_slice/gcode_emit/{tool_transition,layers,object,emit_helpers,motion/state/object_labels}` and the dedicated test-only `option_coverage/tool_*` modules. No speed/time/comparator changes, hardcoded masks, O8 guard removal, CLI flag additions or legacy fallback. By-object mask production and the other five G4 keys remain deferred/INCOMPLETE. Shared physical nozzle does not suppress logical filament switches.

## Verification

The legal Native-backed production RED is a test-only commit; the original
singleton input remains UB_ADJACENT, not replaced or repaired. The four-map
GREEN passes the whole-output, mapped-R, FEATURE wall/infill extrusion,
multilayer and bounded classification assertions in both modes.

The metadata test preserves `filament_self_index` !legal/omissionSome and
replaces only `filament_map` with equal-strength legal/omissionNone, literal
four-map values/labels and exact source-cited G4 bounded metadata assertions.
No assertion deletion or unbounded promotion.

## Native-side nondeterminism observed during verification

`option_coverage::tree_activation::classic_tree_dependencies_activate_support_moves_and_preserve_complete_output`
failed once in official mode and is classified as Native-side nondeterminism.
The original failure stays visible. No comparator tolerance, output filtering
or Native goldens were added.

Failing subcase `tree-activation/tree_support_branch_diameter/tree_slim/max`:
all input ZIP member names and member bytes agree between base and candidate; all four captures have identical Ares RAW SHA256 `2c09388e1b9e62ec811930d85ce7d1190e3d354b890c41ad14e20ef04ecfee07`.
The Native executable first difference is byte28666/line1129, X113.388/Y113.135 versus X113.244/Y113.297, followed by a swap of two existing internal-solid-infill paths/widths/extrusions. No layer-tail retract/label/M624/M625 difference; one nozzle, Auto For Flush map1 cannot enter the dual-nozzle gate. Native-only ordering changes did not alter Ares output across base/candidate.

Existing APA bridges, bridge/filament-flow and G2 dispatch hooks remain.
