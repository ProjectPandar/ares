# G4 Manual-map bounded review

## Rewrite boundary and conclusion

Approved spec: `01799f5c1`; upstream source pinned to `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `PrintConfig.cpp:2489-2493`, `GCode/ToolOrdering.cpp:1275-1312`: four coherent Manual maps `[1,1]`, `[1,2]`, `[2,1]`, `[2,2]`, not automatic grouping or arbitrary vector domains.
- `GCode.cpp:5289-5308,5478-5494,5527-5546`: pre-logical-toolchange traditional timelapse, label-only completion and actual layer-tail retract owner.
- `GCode.cpp:2586-2592,5255-5258,7331-7346,7775-7779,8037-8041`: source-instance sorted label masks and nonempty filament-template M624/M625 consumers.

Rust destinations are `project_slice/gcode_emit/{tool_transition,layers,object,emit_helpers,motion/state/object_labels}` and the dedicated test-only `option_coverage/tool_*` modules. No speed/time/comparator changes, hardcoded masks, O8 guard removal, CLI flag additions or legacy fallback. By-object mask production and the other five G4 keys remain deferred/INCOMPLETE. Shared physical nozzle does not suppress logical filament switches.

## Actual execution receipts

Legal Native-backed production RED is fixed at `32597ade0`, `/tmp/worker2-g4-mapping-red-seeded.Ng65k6`; original singleton input remains `/tmp/worker2-g4-mapping-red-seeded.lzEc6h` (UB_ADJACENT), not replaced or repaired.

Four-map directed GREEN: `/tmp/worker2-g4-mapping-red-seeded.td2g9I`, original execution `65f4db41a`, load14.03, two threads, actual nextest1/1 PASS/exit0. Both fresh candidate full modes also execute and pass the four-map whole-output, mapped-R, FEATURE wall/infill extrusion, multilayer and bounded classification assertions.

Fresh engineering on `fa2c951069960d7ab2656427200fd4855c62553b` / main `b1b958e17`:
`/tmp/worker2-g4-engineering-fixed-fresh.vrLMXk`.
Default environment (all ARES_* removed): workspace8265/8265 PASS,5 existing skipped, exit0; strict workspace/all-targets/all-features clippy `-D warnings`, fmt, WASM core/vgcode/adapter all exit0.

The prior `/tmp/worker2-g4-engineering-fresh.xdCSGA` remains RED:8263/8264, one obsolete metadata nonpromotion assertion. Test-only `fa2c95106` preserves `filament_self_index` !legal/omissionSome and replaces only `filament_map` with equal-strength legal/omissionNone, literal four-map values/labels and exact source-cited G4 bounded metadata assertions. No assertion deletion or unbounded promotion.

Four fresh full legs, separate base/candidate targets, per-leg empty caches and preserved raw artifacts:
`/tmp/worker2-g4-full-pairs-fresh.ojrsfb`.

| Leg | Admission load1 / threads | Actual Summary | Exit |
|---|---|---|---|
| base-seeded |3.76 /2|893/893 PASS,0 skipped|0|
| candidate-seeded |2.87 /2|894/894 PASS,0 skipped|0|
| base-official |3.01 /2|892/893 PASS,1 FAIL,0 skipped|100|
| candidate-official |14.75 /2|893/894 PASS,1 FAIL,0 skipped|100|

The candidate-official load41.35 hold did not create a capture; only that pending leg resumed below30 with a fresh cache. Completed legs were not rerun. START/END/RELEASE are in the coordinator inbox. Interrupted early1909/1911 attempts are VOID, not evidence.

## Raw failure sets and explicit owner adjudication

Raw exact-name subset is **false**, not relabeled as a literal subset:
- Base official: `runner::tree_support::manual::vertical::manual_slim_nested_vertical_painting_preserves_complete_output`.
- Candidate official: `option_coverage::tree_activation::classic_tree_dependencies_activate_support_moves_and_preserve_complete_output`.

Owner explicitly adjudicated the latter as Native-side ND and authorized review without three directed repeats. The original failure stays visible. No comparator tolerance, output filtering or Native goldens were added.

Failing subcase `tree-activation/tree_support_branch_diameter/tree_slim/max`:
all input ZIP member names and member bytes agree between base and candidate; all four captures have identical Ares RAW SHA256 `2c09388e1b9e62ec811930d85ce7d1190e3d354b890c41ad14e20ef04ecfee07`.
The Native executable first difference is byte28666/line1129, X113.388/Y113.135 versus X113.244/Y113.297, followed by a swap of two existing internal-solid-infill paths/widths/extrusions. No layer-tail retract/label/M624/M625 difference; one nozzle, Auto For Flush map1 cannot enter the dual-nozzle gate. Native-only ordering changes did not alter Ares output across base/candidate.

Preserved diagnosis: `/tmp/worker2-g4-classic-tree-offline-diagnosis.md`; four-capture SHA/path/status inventory and full Native raw diff: `classic-tree-max-four-captures.json`, `classic-tree-max-base-candidate-orca.gcode.diff` under the full root. Baseline failure is retained as an independent residual, not silently excused.

## Raw stream inventory

`raw-input-stream-inventory.json` and its summary under the full root record every located Ares raw capture, distinct streams and repeated captures. Input identity hashes uncompressed ZIP member names+bytes, or STL plus supplied profile files; output SHA uses untouched RAW bytes.

Seeded:674 common serialized inputs,0 changed common Ares raw SHA sets; base704/candidate708 captures,608/612 distinct Ares streams,30 repeated-input captures each. Official:652 common serialized inputs,0 changed common raw SHA sets;686 captures each,591/592 distinct Ares streams,30 repeats each. Four candidate-only maps are explicit. Four official baseline-only tree-brim captures remain listed (candidate test abort), not treated as common.

88 additional captures have no serialized input beside the Ares stream and are explicitly inventoried as unidentifiable, not credited as common-input proof. This limitation does not turn them into changed streams or establish parity. No global/all-project/six-key completion claim is made.

## Review gate

Review only this bounded map/source-lifecycle slice. Existing APA bridges, bridge/filament-flow and G2 dispatch hooks remain. Review approval must bind the exact final hash; no push occurs before approval and latest permitted rebase. These receipt-only documentation changes carry the `fa2c95106` execution evidence, not a claim of fresh execution on a later docs hash.
