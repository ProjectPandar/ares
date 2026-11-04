# Timelapse / printable-area retained-output verification

Source owner: GCodeProcessor.cpp:1866-1999, GCode.cpp:2166-2194,
OrcaSlicer.cpp:6209-6227; retained-output amendment e5ec88b5.
Rust boundary: the existing processor pass and shared core emission completion.
No CLI changes, filesystem behavior, second parse or output cloning.

## Commit chain and independent runtime RED

- Builds on the circle slice `3b6e0ebdd`.
- Original spec/RED: `aabee6e76` / `be26d99da`.
- Additional circle final-result runtime RED: `124cebaf`: Ares returned Ok, Native returned -102.
- Retained-output spec: `e5ec88b5`.
- Strengthened runtime RED: `2a28d455`: successful compilation,
  runtime failure because Ares still returned Ok. Only the minimal error enum
  declaration/Display scaffolds compilation; no construction points exist then.
- GREEN: `a501eac2`.

## Preserved per-vertex and retained-byte assertions

`circle_bed_per_vertex_accepts_inward_cylinder_then_rejects_export` uses the
unchanged second stock-circle input at center 50.9. It requires
`ToolpathOutsidePrintableArea { error_code: 4, gcode }`, the exact Native
-102 diagnostic, and complete emitted bytes equal to the committed Native
`plate_1.gcode`, excluding only the generator line. No M73/motion normalization.
The 167943-byte Native stream raw SHA256 is
`cabb272d646a6404e88db9af3c1e7dd8966173aff96afb60b856a744948e1f4a`.
Native final -102 remains rejection, not a successful parity stream.

The original timelapse witness independently asserts exact Native -102 and
core rejection for byte-only and processed entry points. Its input SHA256 is
`67a27bafcda838742db058b1edb812055c9470608e47dc83a314357a8e9b233d`.

Shared completion first finishes the processor's complete progress/statistics
stream, obtains the active result bits, then moves `processed.gcode` into the
error. This follows Native's completed temporary-file rename before the CLI
checks the result. Error-family -102 handling and writing retained bytes
are outside core; core returns owned data and has no native I/O side effects.

## Final-source gates

- Narrow O6 suite: **8/8 PASS**, zero skips.
- Fresh seeded (new Native cache): **647/647 PASS**, zero skips.
- Normal successful outputs: **541 vs 541 raw SHA256 multisets identical**
  to the 7a/circle seeded round; generator-only SHA256 multisets also identical.
- Workspace: **8037/8037 PASS**, nine existing skips.
- Clippy workspace/all targets `-D warnings`: **PASS**, zero warnings.
- Fmt check: **PASS**.
- WASM: **PASS**.
- Fresh official same-round detached base/candidate completed serially, separate
  new Native caches: base **634 PASS / 12 FAIL** of 646; candidate **637 PASS /
  10 FAIL** of 647; zero skips. Nextest exits100, not global PASS.
- Failure sets are **not identical**: candidate-only is empty; baseline-only is
  `change_filament_flush_blocks` (Native timeout on base, completed on candidate)
  and `organic_angle_and_normalized_grid_tip` (base Native M73 placement differs;
  candidate completes). Do not relabel incidental Native success as an Ares fix.
- Eight initialized/reference cases fail for missing producer environment
  (`NotPresent`) bilaterally; this is incomplete reference infrastructure, not
  new slicer parity. Three base X2D Native timeouts remain INCONCLUSIVE; candidate
  completes those Native runs, two X2D full-file assertions still diverge and
  retain their established UB_ADJACENT/non-PASS disposition. Causes changed and
  are not hidden behind matching/selected counts; no retry-to-PASS occurred.
- Persisted official normal bytes: base520 vs candidate523. All **520 common
  raw hashes/multiplicities are preserved**, no base-only output hash; the three
  additional outputs are `tree_support_tip_diameter/grid/{min,max,seeded}` after
  candidate progresses past the base's earlier organic-angle assertion.
  The complete 520-vs-523 sets are **not claimed equal**.
- The original Native -102 witness passes under the official candidate too.
  The official gate holds: candidate failure set is a strict
  subset of base (no candidate-only failure); baseline-only timeout is load-
  adjacent INCONCLUSIVE, and baseline-only Native M73 nondeterminism is outside
  this slice. X2D remains failure on both sides despite changed causes.
  Fresh seeded 647/647 and unchanged normal541 RAW bytes are authoritative.
