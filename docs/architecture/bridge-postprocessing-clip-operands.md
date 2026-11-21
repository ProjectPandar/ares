# Bridge postprocessing clip operands

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/PrintObject.cpp:3995–4011`, smooths the reconstructed bridge,
then skips the final three clips when it is empty. Otherwise it computes one
bounding box of the smoothed polygons and inflates it by `SCALED_EPSILON`
(`libslic3r.h`, 0.0001 mm). `ClipperUtils.cpp:60–136` discards clip polygon
vertices lying with both neighbors outside the same box side, retaining path
order and winding. Native uses that box for all three **clip operands**:
intersect with the limiting area, intersect with total fill, subtract top.
The box is not recomputed after each cut. These booleans do not apply a safety
offset (`ClipperUtils.hpp`, `ApplySafetyOffset::No`).

Port only those three prefilters and the nonempty-subject guard into
`prepare_infill/bridge_over_infill/candidate_bridge_postprocessing.rs`.
The current whole-layer operand handoff is an incomplete port of this block.
Reuse `geometry/bbox_clip.rs::clip_clipper_polygons_with_subject_bbox` and
`BoundingBox::from_polygons`; do not change the helper or Clipper itself.

## Adjacent upstream uses

| Native `PrintObject.cpp` | Ares boundary and disposition |
|---|---|
| `:2146`, prefilter the tops by island before top expansion | Already present in `prepare_infill/surface_type_detection/top_expansion.rs::apply`, the bbox around `island` and `clip_clipper_expolygons_with_subject_bbox(&tops, bounds)` before `intersection_polygons_ex`. |
| `:2969`, prefilter `object_volume` for the tiny-shell visibility difference; nearby `:2975–2981` restrict the protection difference by overlap | Not present: `prepare_infill/vertical_shell_filtering/filter.rs` passes the full volume in both differences. Deferred; the witness's post-shell typed geometry agrees on the target and neighboring layers. |
| `:3858–3863`, `split_near` used at `:4013–4017` for subtracting a completed bridge from `expansion_area` | Not present: `candidate_bridge_postprocessing.rs` differences the full expansion area. Deferred; the two measured layers have one candidate each, so this update cannot affect a later candidate on those layers. The differing bridge output precedes this subtraction. |
| `:3869–3883`, prefilter deep infill by expanded candidate box; prefilter unsupported area by each component box | Already present in `candidate_bridge_area.rs::prepare_candidate_bridge_area_using`, `clipped_deep` and `clipped_unsupported` before the intersections. |

Do not change candidate selection, anchors, angle, smoothing, recomposition,
ironing or arc fitting. The isolated upstream Clipper2 replay reproduces the
upper-layer clip mismatch by changing only its clip operand. At Z=6.4, both
slicers supply the same subject; Native passes 41 total-fill vertices after
prefiltering versus Ares's 475. The changed upper cut propagates to Z=6.2 via
`PrintObject.cpp:4051–4064,4089–4093` (the ensuring band), while the current
bridge cut agrees. Replaying either upper cut produces that slicer's solids
at Z=6.2. This is not a tolerance issue on identical inputs.

## Public witness

Use `tests/parity/concentric-ironing-order/project.3mf` unchanged as the
strict Top control. Derive the AllSolid input at test time using the existing
`rewrite_archive` test helper to change only the embedded `ironing_type`
from `top` to `solid`; do not store a second 3MF.
Through the existing Native double replay / public Ares API and CLI seam,
require whole ordered G-code equality after the single guarded generator-line
normalization. Qualify the test-only failure independently in official and
seeded modes before the fix. Require both tests to pass in both modes after
the fix, and no pass-to-fail changes in the paired ordinary internal solid,
bridge and concentric-ironing inventory. No point or arc tolerance is added.
