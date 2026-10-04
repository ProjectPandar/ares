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

ARD-0105 records actual metadata consumers. Do not delete width/rib/depth-profile
state: mesh-bottom, fake-tower, brim exclusion and chamber-hold consumers remain
in scope. In particular, the current chamber-hold placeholder is hard-coded false.

Before unloading refactors, add native full-output witnesses with ramming enabled,
two cooling moves, stamp distance 3, and tube lengths 4 and 10 to activate both
stamping branches. Verify applied machine/filament settings and emitted phases.

The narrower result/preview-metadata boundary still needs coordinator confirmation:
`is_finish_first` is consumed by Type1, result elapsed time has no downstream read,
and extrusion tool colors are consumed by the GUI preview adapter. No such fields
or their tests have been removed. `Writer::tool` is independently required by
RepRap `M572 D... S0`; all G-code timing and filament statistics stay required.

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

Latest rebased history has not been force-pushed over the approved remote branch.
A new remote ref or local-only continuation needs confirmation, as does the
unused result/GUI metadata boundary. No final REVIEW-REQUEST, approval or main
integration is claimed here.
