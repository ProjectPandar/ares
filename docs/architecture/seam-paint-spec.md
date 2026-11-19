# 3MF seam facet annotations for aligned placement

## Source and boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):
`Format/bbs_3mf.cpp:357,4041,5389–5390,7773–7780` reads, restores and
writes `ModelVolume::seam_facets`. Ares imports `paint_seam` through the model
XML attribute whitelist and retains a distinct `FacetsAnnotation` in
`ProjectMesh`, reusing `TriangleCodes`/`TriangleSelector` (also used for support
and fuzzy paint). No export change is part of this slice.

`GCode/SeamPlacer.cpp:1077–1156` collects the ordinary enforcer/blocker leaves
for distance queries and strict leaves for contour ends in the sliced object
frame. `:601–667,672–979` projects the boundary crossings onto the perimeter,
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
corner-dependent pull before projection. These rules apply to the included
Aligned mode, not merely to initial corner selection.

## Included and deferred

- Included: neutral, enforcer and blocker leaves on ordinary single-part
  objects, with the Aligned strategy, including unpainted controls and both
  Classic and Arachne walls. Reuse the transformed object coordinate frame
  already used by seam visibility, without involving machine-space offsets.
- Deferred: split selector trees, extended palette values, paint on or in
  composition with other volume types, spiral mode, other seam strategies,
  precise seam modifiers, and full-green normalization (`:557–595`). These
  combinations must return a seam-specific unsupported input error before
  emitting G-code. No silent Neutral fallback for deferred input. In particular
  Random (`:1366–1437`) restricts viable outgoing edges, and Nearest also
  requires its emit-time comparator and projection to consume paint.

## Public behavior

Compare strict complete output (only generator line normalized) with pinned
Native in Official and seed-0 modes. Each of X1C, K1 and MK4 has the same box
with no paint, front-only enforcer, rear-only blocker, both front and rear
paint, and a side-painted control. Compare Native control and painted wall
approaches to prove activation separately from byte parity; side paint must
retain the unpainted output. Include a public rejection case for deferred paint.
