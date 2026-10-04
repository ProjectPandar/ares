# Plan: re3D common travel boundary

[Spec](../specs/2026-10-04-re3d-common-travel-boundary.md),
[ARD](../../architecture/ard-re3d-common-travel-boundary.md).
Branch pi/re3d-travel-ordering; baseline931691eb; isolated target and jobs2.

1. Reproduce the three immutable stock cases through public replay. Completed:
   `/tmp/ares-re3d-red`,0/3, ordered_bytes_generator_only.
2. Distinguish geometry/order from time-estimation symptoms. Ranked hypotheses:
   post-wipe detour points, motion-time estimation, extrusion path ordering.
   Diagnostic-only removal of M73/time comments exposes extra final-layer travel
   points; all other motion bytes agree. Acceptance retains those lines.
3. External debugger comparison: `/tmp/ares-re3d-{native,rust}.jsonl` and scripts.
   Native function-entry capture63 and Rust router capture25 share exact
   start(-1529690,-1277833), end(4050000,4050000), outer contour±2471903 and
   inner hole±824713. Geometry hypothesis is eliminated for this route. Source
   cpp190-216 scans end candidates against all start contour IDs; Rust tests
   only start_lines.first(). The nearest start hole is not near the end, but
   the outer contour is reachable from both endpoints. Captured before production edits.
4. Port just the common-boundary selection with a membership set and end-first
   scan. Preserve the existing replacement paths and all assertions.
5. Strict target/fresh-original checks,68+823 and extra-overhang controls, four
   gates, then push REVIEW-REQUEST. Retain failures. Rebase latest main and
   revalidate after approval before ff integration.

Six Flashforge/Raise3D random-coordinate cases are recorded in the vendor triage
plan and explicitly not repaired. Next after re3D: Prusa terminal wipe, then
Creality solid-infill cooling.

## Final validation on931691eb

The source selection change restores3/3 stock targets; all three newly sliced
original Orca2.4.2 CPU0 references also PASS. Entire inventories use
`ordered_bytes_generator_only`: targets3/3, fresh3/3, full68/68, controls823/823,
extra-overhang stock2/2 and active step2/2, official8/8. No skipped/dropped cases.
Reports: `/tmp/ares-re3d-final-*/replay-summary.json`.

`/tmp/ares-re3d-final-exits.txt`: all exit0. Workspace7595/7595 PASS (9 existing
skips), fmt/defaultClippy/actual core+adapter WASM build PASS. Clippy remains219
pre-existing warnings. Logs share `/tmp/ares-re3d-final-` prefix.
One production file changed; no tests/assertions removed, probes, suppressions,
new fallback or coordinate/timing normalization. Await coordinator approval,
then latest-main rebase and full revalidation before ff-only integration.
