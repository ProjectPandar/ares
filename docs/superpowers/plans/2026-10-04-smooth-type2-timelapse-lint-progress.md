# Type2 lint and consumer-audit continuation

Continues [the earlier progress record](2026-10-04-smooth-type2-timelapse-progress.md)
without growing that file beyond 400 lines. ARD-0105 remains authoritative.

## Purge ownership and geometry-emission lint checkpoint

- Owning boundaries: `ToolOrdering.cpp:1620–1826::WipingExtrusions`,
  `GCode/PrintExtents.cpp`, `WipeTower2` cone/interface emission, and its
  `add_wipe_point` callers. Private helpers preserve pass/object/copy/region
  order and original collection IDs; support maps remain object-scoped.
- Collection volume subtraction remains f32; support subtraction retains its
  f64 intermediate before conversion back to f32. No partial-collection purge,
  dummy reads, visibility-based suppression, or `allow`/`expect` attributes.
- `Planned::generate` receives the used-tool list and final `LayerTools` as
  one borrowed tool-ordering projection. This keeps the narrow input seam and
  test fixture without adding a public constructor or exposing private fields.
- An intermediate signature change missed one unit-test caller and failed
  compilation (`E0061`), recorded in
  `/tmp/ares-pi-tower-lint-ownership-signature-compile-failure.log`.
  This was a local compile failure, not a behavioral RED or the known wasm
  main failure. The fixture now passes the same data through the new signature.
- Current-tree `/tmp/ares-pi-tower-lint-ownership-green-*` evidence:
  **323 focused PASS**, official **8/8 PASS**, seeded **8/8 PASS**, WASM,
  fmt and diff checks PASS. Full workspace has not been rerun for this slice.
- Clippy completes with **160 warnings** (204 before cleanup). Six introduced
  Type2 diagnostic sites remain: three unload nesting sites, one live-geometry
  metadata warning, one result-metadata warning and one preview-tool warning.
  Being below main's reported 179 is not zero-new-warning clearance.

## Remaining source audit

ARD-0105 records actual metadata consumers. Preserve width/rib/depth-profile
state: mesh-bottom, fake-tower conflict checking and chamber-hold consumers remain
in scope. The chamber-hold placeholder is still hard-coded false. The fresh Type2
path does not populate `FakeWipeTower::outer_wall`; `Brim.cpp:651` requires that
map to be nonempty, so its mask must not be invented for Type2. First-layer
support/raft fallback, actual print-object membership and brim boxes still need
coverage.

## Approved metadata boundary and unloading checkpoint

The coordinator authorized deferring Type1-only finish metadata, GUI extrusion
colors and unread result/writer timing bookkeeping. Removed only those fields
and their metadata assertions, with source citations in ARD-0105 and the spec.
`Writer::tool` remains; a new regression checks `M572 D2 S0` then `M572 D1 S0`
for both RepRap flavors. Geometry, filament accounting and processed timing stay.

Before changing unloading, added official full-output witnesses with ramming
enabled, two cooling moves, stamp distance 3, and tube lengths 4 and 10. Both
machine and filament applied settings and emitted ramming/cooling/load phases
are asserted. `/tmp/ares-pi-tower-unload-witness-official.log` passes the tool
matrix (now nine cases); this is a refactoring baseline, not a behavioral RED.
`/tmp/ares-pi-tower-deferred-unit.log` passes 324 focused tests.

Extracted private ramming-first-line, cooling and stamping phases from
`WipeTower2::toolchange_Unload`, preserving arithmetic and command ordering.
`/tmp/ares-pi-tower-unload-green-*` gates: **324 focused PASS**, official
**8/8 PASS**, seeded **8/8 PASS**, WASM/fmt/diff PASS. Full workspace has not
been rerun for this checkpoint. Clippy **160 → 158 → 155**: the five metadata
and unload sites are gone; the live-geometry field diagnostic remains. These
counts are not zero-new-diagnostic clearance; identity comparison is still due.

## Integration status

Published branch: `pi/smooth-type2-timelapse-tower-rebased-95e23008`, without force,
through `3788024a`. After the coordinator's latest request, all 15 feature patches
rebased without conflicts onto repaired main `944c0459`, producing `a6a54e4d`.
Range-diff marks all patches identical:
`/tmp/ares-pi-tower-rebase-944c0459-range-diff.log`. The prior history remains at
`checkpoint/tower-before-main-944c0459`; all required main ancestors are retained.

Fresh `/tmp/ares-pi-tower-rebase-944c0459-*` gates: **7640 workspace PASS,
9 skipped**, official **8/8 PASS**, seeded **8/8 PASS**, WASM/fmt/diff PASS.
Clippy completes with **160 warnings**; the six introduced sites above remain.
The previously reported `no_unapproved_dynamic_values` failure is fixed upstream
and that exact policy test passed in this workspace run; no local workaround.

With coordinator authorization, `8354b87c` was published without force to
`pi/smooth-type2-timelapse-tower-rebased-944c0459` and local upstream updated;
the older published history remains intact. The coordinator subsequently reported
main `f10c3b17` (region ironing speed). Before integration, rebase the then-latest
`origin/main` and revalidate; the 944-based gates above are not final integration
evidence. Never force-push the published ref. No final REVIEW-REQUEST, approval
or main integration is claimed here.

## f10c3b17 rebase and active geometry worktree (not GREEN)

Rebased all 18 commits without conflicts onto `f10c3b17`, yielding
`58bbbe7a47b68c397e2e0425106c05b37479e203`. Range-diff is identical throughout:
`/tmp/ares-pi-tower-rebase-f10c3b17-range-diff.log`; required main ancestors
remain reachable. Backup: `checkpoint/tower-before-main-f10c3b17`. No force push.
The new implementation and witnesses below are uncommitted.

- Chamber RED: `/tmp/ares-pi-tower-chamber-red-official.log`, expected
  `; FLAT_CHAMBER true`, actual `false`; disabled control passed. Ported the
  `construct_mesh` bottom and chamber-area consumer. Rectangle/rib and disabled
  controls, including the end-template flag, pass full output in
  `/tmp/ares-pi-tower-chamber-matrix-official.log`.
- Conflict witness first failed because `StageError::Display` omits native
  stdout. This fixture failure is not the behavioral RED. The corrected witness
  verifies native `CLI_GCODE_PATH_CONFLICTS=-101` (Unix exit155), preserves native
  captures and Ares output, and fails because Ares accepts the input:
  `/tmp/ares-pi-tower-conflict-red-confirmed-official.log`, artifact
  `/tmp/ares-pi-tower-conflict-red-confirmed-official/tower-conflict-YyC4w3`.
- Connected Type2 fake layers and depth profile to source conflict checking.
  The initial `E0282` closure annotation failure is retained in
  `/tmp/ares-pi-tower-conflict-clippy.log`; it is not a behavioral RED.
  `/tmp/ares-pi-tower-conflict-green-official.log`: all five then-existing Type2
  tests pass, including both new consumers. Clippy reports 154 warnings in
  `/tmp/ares-pi-tower-conflict-fixed-clippy.log`; the tracked geometry-field
  diagnostic is gone, but a fresh main diagnostic-identity comparison is due.
- Added a further membership witness: one source object, two actual instances,
  second instance scaled 2x in X. Official output proves positive extrusion at
  both copies and source center `112.5,110`. Ares rejects it at the pre-existing
  `raw_intersections::validate_centering` gate before tower generation:
  `/tmp/ares-pi-tower-membership-red-official.log`, native/output evidence
  `/tmp/ares-pi-tower-membership-red-official/case-FBHkaL`.

Current membership RED is retained, not ignored or weakened. ARD-0105 records
`PrintObject.cpp:85–114` as the shared centering prerequisite; merely deleting
its current guard is not acceptable. Support/raft fallback and brim ownership
remain unfinished, as do complete seeded/workspace/WASM gates and final lint
identity comparison on this tree. No final REVIEW-REQUEST is warranted.

## Coordinator scope decision and first-layer fallback checkpoint

Coordinator assigned general multi-transform Print/PrintApply centering to
pi-tip after RNG integration. The membership case and its helper changes have
been removed from the tower suite as explicitly directed, with the original
RED, helper patch, source and artifact references handed off at
`/tmp/ares-tower-to-tip-centering/`. ARD-0105 records the deferred dependency;
`raw_intersections::validate_centering` is unchanged.

- No-skirt raft public-output RED:
  `/tmp/ares-pi-tower-raft-hull-red-official.log`. Native bounds are
  `RAFT_MIN 11.7993,100.871` / `RAFT_MAX 119.129,153.824`; old Ares used body-only
  `11.7993,105` / `115,153.824`.
- Ported `Print::first_layer_islands` normal spacing/tree contour/raft coverage
  and qualifying-brim support bounds. Source float spacing, scaled epsilon and
  original-copy shifts are retained. The full four-case matrix is GREEN in
  `/tmp/ares-pi-tower-raft-hull-green-official.log`.
- The initial private-import `E0603` failure is retained in
  `/tmp/ares-pi-tower-islands-clippy.log`; it is not a behavioral RED.
- Extra complete-output normal/tree support witnesses pass in
  `/tmp/ares-pi-tower-islands-official.log`. Focused core/raft tests: 327 pass in
  `/tmp/ares-pi-tower-geometry-unit.log`, including ownership, support threshold,
  endpoint and negative-grid conflict regressions.
- First exact f10c3b17 diagnostic audit: 157 base / 154 branch, 152 inherited,
  five removed, **two introduced** at `extrusion_islands::prepare`. They have
  been replaced with equivalent lazy `bool::then` capture, not suppressed.
  Fresh final-baseline comparison is still required; lower totals were not used
  as clearance. Evidence: `/tmp/ares-pi-tower-f10c3b17-diagnostic-identities.json`.

Coordinator then advanced main to `7ec6cd05`; fetched full ref
`7ec6cd0541a9c05c6551a5120478b532971ac08a`. Final rebase and complete gates follow.
