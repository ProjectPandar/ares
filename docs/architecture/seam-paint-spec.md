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

### Random

`GCode/SeamPlacer.cpp::pick_random_seam_point` (`:1366–1444`) uses
`SeamComparator::is_first_not_much_worse` (`:1227–1257`) in both directions
for viability: Enforced outranks Neutral, which outranks Blocked. Enforced
points are equivalent without checking overhang or embedding; the other
states retain those checks. Random does not prefer the central enforcer.

In `project_slice/seam_placement/alignment/random.rs`, replace the
unpainted eligibility rules with the paint-aware comparator and outgoing
edge rules. Use the candidate types and independent exact enforced/blocked
end flags produced by `seam_candidates.rs` and `paint/samples.rs` from the
same perimeter's paint intervals (`:739–860`). With exact enforced ends,
an Enforced candidate cannot sample the edge to a non-Enforced successor.
With exact blocked ends, a non-Blocked candidate cannot sample the edge to
a Blocked successor. Keep such edges at zero length, including at the loop
wrap, and retain the source's bounded residual and zero-length placement.
The first candidate still supplies the seed for one draw before the scan;
paint can change viable lengths, not the draw order. Random is finalized by
its picker and does not enter string alignment (`:2181–2200`).

### Extended selector palette

`Format/bbs_3mf.cpp:4041,5389–5390,5586–5587,5917` and
`Format/3mf.cpp:1793,2221–2222` populate `ModelVolume::seam_facets`.
`FacetsAnnotation::set_triangle_from_string` (`Model.cpp:4138–4164`) reads
hexadecimal nibbles in reverse order. With the default `ExtruderMax = 32`
(`TriangleSelector.hpp:24–66,410–414`), `TriangleSelector::deserialize`
(`TriangleSelector.cpp:1855–1886`) preserves states 3–17 via one extension
nibble, and 18–32 via the escape nibble `F` followed by `state - 18`.
`project/facets_annotation.rs` and `project/facets_annotation/selector.rs`
validate and decode this shared leaf encoding without changing the mesh.

`SeamPlacer.cpp:1114–1133` exports ordinary and strict facets only for exact
states 1 (enforcer) and 2 (blocker); the selector tests equality, not a state
range (`TriangleSelector.cpp:1492–1510`). At seam candidates, the default is
Neutral unless a green or red query hits (`SeamPlacer.cpp:829–859`). Thus
3–32 are inert paint, even on a model part or negative volume containing
active paint. They are not rewritten to state 0: Native's separate full-green
coverage check uses exact `NONE` plus blockers (`:1135–1153`). In
`seam_placement/paint.rs`, admit the extended states without changing those
exact-state queries or lifting the full-green guard for active enforcers.
An annotation without enforcers or blockers needs no seam paint surfaces.

## Included and deferred

- Included: neutral, enforcer, blocker and extended-palette leaves, with or
  without selector subdivision, on model parts and negative volumes with Aligned, AlignedBack,
  Nearest, Rear or Random, including unpainted controls and both Classic and Arachne walls. Reuse the
  transformed object coordinate frame already used by seam visibility,
  without involving machine-space offsets. Negative paint can affect the
  perimeter of the hole cut by that volume.
- Deferred: spiral mode, Precise Seam modifiers, and full-green normalization
  (`:557–595`). The existing full-green rejection still applies to each volume
  with enforcers and no exact NONE or blocker surface; partial paint also retains
  the perimeter coverage guard. These combinations
  must return a seam-specific unsupported input error before emitting G-code.
  No silent Neutral fallback for deferred input.

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
AlignedBack, Nearest, Rear and Random configured in the 3MF. Verify each against both
Native producers and compare Native wall approaches to the plain run for the
same printer and strategy to distinguish active paint from non-changing controls.

For AlignedBack, Nearest and Random, replace front or rear paint with states
3, 4, 15, 17, 18 and 32. On each printer compare complete Native output with
the corresponding plain control; all six states, including a subdivided face
and a box entirely annotated with state 32, must leave output unchanged.
A real front enforcer must change an inner-wall approach, and adding
state 32 on the rear must leave the front-enforced output unchanged. Check
strict Native/API/CLI parity for these variants in both replay modes.
