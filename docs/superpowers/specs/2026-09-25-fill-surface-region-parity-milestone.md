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

## Scope of this milestone

1. Trace the effective sparse `Internal` expolygon reaching
   `fill_entities` (which of: `staged.fill_surfaces` after
   `clipped_fill`, combine_infill `rewrite_layer`, or
   bridge_over_infill rewrites) and localize the +0.203mm:
   - dump `fill_expolygons` (infill_boundary output) for the case —
     if it already exceeds oracle, the bug is upstream in
     gap_extrusion/top_split remaining-set computation;
   - if `fill_expolygons` matches oracle, the over-large region
     comes from classification/rewrite paths downstream.
2. Port the missing upstream behavior into the owning stage
   (cited above), NOT a new ares-designed region.
3. Verify with `ARES_DUMP_TPMSREG`/`ARES_DUMP_SURF` vs oracle dumps
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
