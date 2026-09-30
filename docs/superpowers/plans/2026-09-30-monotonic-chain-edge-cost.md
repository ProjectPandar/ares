# Monotonic chain edge-cost plan

Source: `Fill/FillRectilinear.cpp::AntPathMatrix::operator()` and
`chain_monotonic_regions`. Destination: `fill/rectilinear/path_matrix.rs`.
Included/deferred boundaries and temporary-shell constraints are defined by
ARD-0041 and the matching spec.

1. Reduce the captured KSR concave contour to a thin solid-fill CLI fixture.
   Verify a complete-output RED caused by the same path-choice behavior.
2. Preserve upstream edge costs with the smallest implementation change.
   Remove obsolete implementation-pinning coverage only where superseded.
3. Verify complete-output GREEN, KSR local restoration, workspace nextest,
   fmt/clippy and WASM/Windows/macOS compilation.
4. Independent six-axis review; repair and re-review; commit/push code and
   documents together. Do not claim overall printer/option parity.

Diagnostic evidence: `/tmp/ares-ksr-next.ATDjdd/matrix-counterfactual.gcode`
restores 975 consecutive official lines after changing one observed edge
from the contour branch to the existing straight-distance branch. Region
boundaries/internal costs match; only the middle region's chain direction
previously differed. Independent review validated this local causal result.
This was the pre-repair causal evidence; production repair is recorded below.

## Fixture reduction history

The draft CLI test is registered as
`orca_parity_concave_monotonic_fill_preserves_complete_output`, with a
separate static, triangulated STL fixture. It checks effective process
options and compares complete output. These initial attempts were not yet
sensitive regressions; the final reduction below is sensitive.

- Original 20-point reduction, 0.4 mm height: full comparison RED only on
  Y112.477 versus Y112.478 (`/tmp/ares-monotonic-red.qJqSDD`). This is not the
  target path-choice failure. Changing sparse infill direction did not
  affect solid fill and is not a valid reduction knob.
- Automatic official arrangement rotates this model about 100 degrees.
  Explicit solid direction and `align_infill_direction_to_model=1` restore
  relative orientation, but the initial model still has one region per
  layer (`aligned-regions.log`).
- A 0.21 mm outward-expanded outline restores three first-layer regions,
  but full output already PASSes without repair
  (`/tmp/ares-monotonic-expanded.a0RkBs`). Its last region ends at scanline
  11 with even parity; KSR ends at 10 with odd parity. Independent review
  suggests trimming the remote edge rather than altering the notch.
- Whole-model 0.95 scaling produces another unrelated skirt-coordinate
  RED (`/tmp/ares-monotonic-scaled.EEvVYn`), not useful causal evidence.
- An intermediate draft clips the remote edge in the fill frame by 0.37708 mm,
  removes the skirt, and uses one 0.2 mm layer. Complete output PASSes
  (`/tmp/ares-monotonic-clipped.F7qRMd`, 0.770 seconds). This also does not
  establish defect sensitivity; keep reducing/characterizing before the fix.

GDB probes must use the unoptimized dev CLI. Nextest builds an optimized
test-profile CLI at the same target path. The first reduced-region probe
stopped on an optimized-out variable, despite batch exit 0; its absence of
data is not evidence. `cargo build -p ares-cli` restored the dev binary and
`reduced-regions-dev.log`/`aligned-regions.log` then exited normally with
actual captures. No original KSR fixture or production slicing source changed.

At that stage only fmt/clippy had passed; workspace/Tier-1 and completion
review were deferred until a sensitive RED and repair.

## Sensitive regression and implementation

The missing reduction control was first-layer elephant-foot compensation.
With the original outline, model-relative solid direction and compensation
disabled, the reduced model reproduces the same three-region chain and
split path. Simplifying outline coordinates to four decimal places and
using one 0.2 mm layer removes unrelated near-rounding differences while
retaining the failure. Three-decimal rounding did not retain sensitivity.

Authoritative RED:
`/tmp/ares-monotonic-rounded4-red.dXLwCz/case-nCXSXv` and
`/tmp/ares-monotonic-rounded4-red.log` (nextest exit 100). Output differs
in the contour connection, associated M73 placement and used filament.
The process-only straight-cost counterfactual produces complete matching
output except the allowed generator identity/time line:
`/tmp/ares-ksr-next.ATDjdd/rounded4-counterfactual.gcode`.

Production `path_matrix.rs` now computes straight endpoint distance for
ordered regions, preserving the upstream f32 norm/unscale boundary.
The obsolete matrix orientation bit-pattern assertion is removed; the
unrelated pheromone reset test remains. No official oracle or comparator
change is made.

Main-tree GREEN: `/tmp/ares-monotonic-green.liU7Ok`,
`/tmp/ares-monotonic-green.log`, 1 test PASS in 0.778 seconds.
Clippy completed successfully (`/tmp/ares-monotonic-fix-clippy.log`).
The isolated worktree `/tmp/ares-monotonic-isolated.4qwT2E`, based on
a20f0e91, contains only this slice. Its live test/workspace/fmt/clippy chain
completed with exit 0. Live complete-output regression: 1 PASS (0.782 s).
Workspace: 7375 PASS, 9 skipped (265.318 s). Fmt passed; Clippy passed with
248 existing warnings. Logs: `/tmp/ares-monotonic-isolated-live.log`,
`/tmp/ares-monotonic-isolated-workspace.log`,
`/tmp/ares-monotonic-isolated-fmt.log`,
`/tmp/ares-monotonic-isolated-clippy.log`.
Global printer/option completion and KSR full output remain unproven.

Production KSR rerun (isolated optimized CLI, exit 0):
`/tmp/ares-monotonic-isolated-live.shWZAt/ksr-production.gcode`.
Compared with `/tmp/ares-ksr-next.ATDjdd/plate_1.gcode`, the target travel
`G1 X115.984 Y94.535 F60000` starts at official line 102025 and Rust line
102021. The following 975 lines match exactly, restoring the split path.
The next difference is official `M73 P37 R67` versus a Rust motion line.
The first whole-file difference after generator metadata is total estimated
time: 1h49m0s versus 1h49m1s. No normalization or golden replacement is made.
Independent review reproduced the complete reduced-fixture PASS and these
KSR local/global results.

Tier-1 compiler checks completed successfully in the isolated worktree:
`/tmp/ares-monotonic-isolated-{wasm,windows,macos}.log`.
These are compiler checks, not runtime parity tests on the three targets.

The supplementary official edge-return debugger (session 25841) was
explicitly retired after independent reduced-output and production-KSR
verification superseded it. It was terminated (exit 137), not completed,
and provides no accepted evidence. Its original log is retained.

Independent read-only six-axis review approved the nine-file slice after
checking requirements, logic, boundaries, quality, coverage and actual
terminal results. No remaining repair items. Approval covers this slice
only, not global KSR or all-printer/all-option parity.
