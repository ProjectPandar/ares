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
