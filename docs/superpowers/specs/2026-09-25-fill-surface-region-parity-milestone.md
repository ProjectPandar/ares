# Milestone: `fill_surfaces` region-extent parity (stInternal inset)

## Upstream boundary (source-cited)

The sparse/`stInternal` fill REGION is produced inside
`PerimeterGenerator` — not by `detect_surfaces_type`:

- `PerimeterGenerator::process_classic` → island loop
  (`OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1237-1690`):
  `not_filled_exp = union_ex(inner_pp)` (remaining area after all
  wall loops + gap fill), then
  `infill_exp = offset2_ex(not_filled_exp, -(inset + min_perimeter_infill_spacing/2),
  +min_perimeter_infill_spacing/2)` collapsed-narrow, plus
  `top_infill_exp = intersection_ex(fill_clip, offset_ex(top_fills, ext_perimeter_spacing/2))`
  unioned in; `fill_surfaces->append(infill_exp, stInternal)` (:1670).
- `fill_clip` second form (:654):
  `offset_ex(orig_polygons, ext_perimeter_spacing/2 - scale_(infill_spacing_unscaled/2))`
  (≈ orig − ~0.015mm; the clip only matters for the one-wall-top
  path).
- `split_top_surfaces` (:574-660) feeds `top_fills`/`fill_clip`
  (only_one_wall_top path).
- The detected `stInternal` classification from
  `PrintObject::detect_surfaces_type` (`PrintObject.cpp:1520`) is
  OVERWRITTEN by `make_perimeters` output
  (`Layer.cpp:224-225`: `fill_surfaces.surfaces.clear()` +
  `make_perimeters` + `fill_expolygons = to_expolygons(fill_surfaces)`).

## Ares destination boundary

- `perimeters/classic/infill_boundary` already ports the
  `not_filled_exp → infill_exp` computation
  (`stage_surface`, `geometry.rs`), emitting `fill_surfaces`
  (stInternal) + `fill_no_overlap` per island surface.
- `prepare_infill/surface_type_detection` currently REPLACES those
  fill surfaces with a classification-derived set
  (`stage.rs` `classify_slices` → `internal(previous − top_bottom)`,
  then `clipped_fill(slices, output.fill_expolygons)` intersects the
  classified kinds with the boundary set). The `Internal` kind
  therefore carries the detect_surfaces extent clipped to
  `fill_expolygons`, which is correct in shape only if
  `fill_expolygons` itself matches upstream `infill_exp` — oracle
  measurement (tpmsd case) shows ares's effective sparse region is
  ~0.203mm/side LARGER.

## Oracle evidence

`ORCA_DUMP_TPMSD` (oracle `FillTpmsD.cpp:112` bbox) vs
`ARES_DUMP_TPMSREG` (ares `fill_tpmsd_surface` rotated contour) on
`sparse_infill_pattern/tpmsd` (10mm cube, 45°, 15%): oracle
`±4019688`, ares `±4223228` (+0.203mm/side). tpms inputs
(angle/density/multiline/z/npts) all match — only the region extent
differs.

## Forensic findings (2026-09-25, tpmsd/cube10/15% case)

Oracle probe `ORCA_DUMP_FK` (patched `Fill.cpp:1354`
`fill_surface_extrusion`) vs ares `ARES_DUMP_KINDS` (coalesce,
kind+bbox per surface, region+object+layer tagged):

| layer | oracle surfaces | ares surfaces |
|---|---|---|
| z1.2–8.4 | stInternal ±4223228 | Internal ±4223228 |
| z8.6 (l42) | stInternal ±3816139, stInternalSolid ±4223238 | Internal ±3816159, InternalSolid ±4223238 |
| z8.8 (l43) | stInternalBridge ±4223228 | InternalBridge ±4223228 |

→ **surface sets and kinds match upstream**; the 0.203mm error is NOT
a fill-region misclassification.

**Resolution (2026-09-26)**: the tpmsd parity defect was stale evidence
— the case artifact dir `case-CM7Svk/ares.gcode` was from 2026-09-24
and had not been regenerated. Current-build tpmsd G-code is
byte-identical to oracle (only the generator banner differs). The
region-boundary work is CONFIRMED COMPLETE.

**Follow-on residuals discovered** (post-fix sweep):

1. `sparse_infill_pattern=concentric`: InternalSolid narrow-split
   (`Fill.cpp:597 split_solid_surface`, line-based path) leaves a
   ~0.13mm `ConcentricInternal` VariableWidth sliver inside the solid
   infill at layer 1 that oracle does not emit. Ares port lives in
   `project_slice/group_fills/narrow/{split,filter,trace}.rs` — the
   divergence is inside the ported reconstruction, not in dispatch.
2. M73 placement residual on concentric: after fixing the `e_only`
   cache eligibility (`estimate.rs`, upstream :574 unconditional push),
   concentric still shows a ~2-g1-line elapsed-time drift inside the
   time model (M73 lands on the next eligible g1 instead of the
   boundary g1). All other previously-failing M73 domains
   (nozzle_temperature, spiral_mode, support_angle, top_surface_pattern)
   now pass byte-parity.

`Fill::fill_surface` (`FillBase.cpp:104-118`) shrinks the surface by
`scale_(overlap − spacing/2)` (overlap=0 → −0.2035mm for
spacing≈0.407): oracle gcode sparse reach = boundary±0.203mm
(X114.02); ares emits at the UNSHRUNK edge (X114.223). Verified via
`ARES_DUMP_TPMSOUT` that ares's tpmsd polylines DO stay within the
shrunk ±4019688 region — so the 114.223 extrusions are emitted by a
**different entity inside the same `;TYPE:Sparse infill` block**
(connect/anchor/approach entity reaching the raw ±4223228 surface),
not by the tpmsd fill paths. Oracle's corresponding connect path is
clipped at ±4019688 too. Next step: bisect the entity→islands→gcode
emit path (dump per-collection extents at `extrusion_islands` and the
`start_travel`/`approach`/connect-emit sites) to find which component
adds the unshrunk-boundary segments.

## Scope of this milestone

1. Localize the 0.203mm sparse-boundary overshoot (findings above:
   surface parity confirmed; overreach originates in the emit/connect
   path, not the region).
2. Port the missing upstream behavior into the owning stage
   (cited above), NOT a new ares-designed region.
3. Verify with `ARES_DUMP_TPMSREG`/`ARES_DUMP_SURF`/oracle `FK` dumps
   on the tpmsd case and the ksr fixture (crosshatch sparse infill).
4. Regression test: sparse `Internal` surface bbox/area parity for a
   unit cube case inside the existing prepare_infill test harnesses.

## Deferred

- The `stInternalSolid`/`stTop`/`stBottom` boundary refinements
  already handled by other staged ports; multi-region
  `make_perimeters` grouping (`Layer.cpp:236-275`); arachne variant
  (`process_arachne` uses the same append sites :1899-1910, :2519).
- The `fill_no_overlap` path exists and is ported; only the
  `fill_surfaces[stInternal]` extent is in scope.
