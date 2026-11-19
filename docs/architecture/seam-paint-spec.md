# 3MF seam facet annotations

## Source and boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):
`Format/bbs_3mf.cpp:357,4041,5389–5390,7773–7780` reads, restores and
writes `ModelVolume::seam_facets`. Ares imports `paint_seam` through the model
XML attribute whitelist and retains a distinct `FacetsAnnotation` in
`ProjectMesh`, reusing `TriangleCodes`/`TriangleSelector` (also used for support
and fuzzy paint). No export change is part of this slice.

`GCode/SeamPlacer.cpp:1077–1156` collects the ordinary enforcer/blocker leaves
for distance queries and strict leaves for contour ends in the sliced object
frame. Seam paint reuses the shared tree decoder in
`project/facets_annotation/selector.rs` (`TriangleSelector.cpp:1717–1799,
1813–1949`): distance uses the ordinary leaves (`:1474–1493`), whereas
contours use the strict traversal (`:1540–1666`), which subdivides selected
leaves along neighboring T-junctions so interior edges cancel by vertex
identity. `GCode/SeamPlacer.cpp:1114–1133` builds these two views separately
for both enforcers and blockers; neither changes the sliced model mesh.
`GCode/SeamPlacer.cpp:1103–1156` consumes only model parts and negative
volumes. `project_slice/seam_placement/paint.rs::SeamPaint::from_object`
applies `trafo_sliced * volume_matrix` to each volume's leaves. The strict
contour is extracted within each volume before appending its edges; edges
from separate volumes must not cancel. The ordinary distance triangles are
combined by paint state. Residual annotations on helpers and ordinary
modifiers do not participate in validation or collection.
`GCode/SeamPlacer.cpp:601–667,672–979` projects the boundary crossings onto the perimeter,
adds exact-end samples (including blocker guards) or samples green by radius
when ends are unavailable, resolves red over green, and selects a central
candidate from the longest enforced patch. The Rust destinations are
`project_slice/seam_placement` and `project_slice/seam_candidates`; the existing
unpainted candidate path is the compatibility shell for this source boundary.

`SeamComparator` (`:1176–1275`) prioritizes central enforcers in aligned modes,
then Enforced > Neutral > Blocked, including string matching; equal blocked
points still use ordinary geometric priorities. `place_on_loop` (`:1498–1627`)
bounds fitting to allowed arcs, with allowed endpoints available to the solver.
The string fit (`:2005–2083`) gives enforcers additional weight and retains the
corner-dependent pull before projection.

### Rear and Nearest

`calculate_overhangs_and_layer_embedding` (`:1683–1726`) fills geometric
context independently of paint; it does not overwrite the classification
from `:850–860`. `SeamComparator` (`:1176–1222`) compares the paint state
before overhang, embedding, Rear Y or Nearest distance. Its central-enforcer
preference applies only to Aligned and AlignedBack, not to Rear or Nearest.

- **Rear (`back`):** `pick_seam_point` (`:1339–1350,2176–2194`) scans candidates,
  then `align_seam_points` (`:2197–2204`) uses the paint-aware comparison
  (`:1227–1268`) and existing constrained fit, including the Rear length
  multiplier (`:2027–2075`). In `alignment.rs` and `alignment/paint_rules.rs`,
  limit the comparator's central preference to the aligned strategies. The
  separate proximity shortcut for a selected central enforcer in the next
  layer (`:1802–1818`) still applies.
- **Nearest (`nearest`):** initialization skips picking and alignment
  (`:2161–2204`). `pick_nearest_seam_point_index` (`:1352–1363`) is called
  by `place_seam` using the current nozzle position (`:2371–2383`). Carry
  the sampled paint states through `seam_placement.rs::nearest_seam_data`
  into the existing `runtime/nearest.rs` comparison, retaining the loop
  association, candidate order and inner-wall projection (`:2345–2423`).
  Equal paint states use the existing geometric ranking in both strategies.

## Included and deferred

- Included: neutral, enforcer and blocker leaves, with or without selector
  subdivision, on model parts and negative volumes with Aligned, Nearest or Rear,
  including unpainted controls and both Classic and Arachne walls. Reuse the
  transformed object coordinate frame already used by seam visibility,
  without involving machine-space offsets. Negative paint can affect the
  perimeter of the hole cut by that volume.
- Deferred: extended palette values on participating volumes, spiral mode,
  AlignedBack and Random seam strategies, Precise Seam modifiers, and
  full-green normalization (`:557–595`). The existing full-coverage rejection still applies to each
  painted volume; partial paint also retains the perimeter coverage guard. These
  combinations must return a seam-specific unsupported input error before
  emitting G-code. No silent Neutral fallback for deferred input. In particular
  Random (`:1366–1437`) restricts viable outgoing edges.

## Public behavior

Compare strict complete output (only generator line normalized) with pinned
Native in Official and seed-0 modes. Each of X1C, K1 and MK4 has the same box
with no paint, front-only enforcer, rear-only blocker, both front and rear
paint, and a side-painted control. Compare Native control and painted wall
approaches to prove activation separately from byte parity; side paint must
retain the unpainted output. For each printer, also compare a locally subdivided
front patch with a T-junction against a control with no paint and against a
locally subdivided side patch that must preserve unpainted Native output.
For each printer, also exercise two separately transformed model parts and a
painted negative volume cutting a hole. Compare each to its unpainted Native
control and check that changes reach the intended walls. Residual paint on
an ordinary modifier and a helper must preserve complete output. Include
public rejection cases for deferred paint.

Reuse the box geometry and the five paint variants for each printer with
Nearest and Rear configured in the 3MF. Verify each against both Native
producers and compare Native wall approaches to the plain run for the same
printer and strategy to distinguish active paint from non-changing controls.
