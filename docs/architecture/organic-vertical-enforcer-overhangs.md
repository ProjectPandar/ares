# Organic tree vertical paint enforcer overhangs

## Upstream boundary

Port OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`Support/TreeSupport3D.cpp:3527–3557`, specifically the append at lines
3542–3556 of `generate_support_areas`. The enabled smart-overhang route first
copies detected overhang polygons, then appends circles for vertical painted
enforcers before precalculation (lines 3561–3562).

- Use `zs_from_layers` (`Layer.hpp:357–363`): object `slice_z` values in `f32`,
  not print Z or support-plane Z. Lower-bound by centroid Z, accepting the first
  layer whose plane is at least that height only when `0 < layer_nr < layer_count`.
- Append the translated polygon at `layer_nr + num_raft_layers`. Its center is
  the centroid XY in scaled coordinates, truncated to `coord_t`. Its radius is
  0.5 mm, with `SUPPORT_TREE_CIRCLE_RESOLUTION = 25`
  (`Support/TreeSupportCommon.hpp:40`). In `Polygon.cpp:789–805` that argument
  is a chord error in scaled units, not a vertex count.
- The facet normal is read but unused upstream; do not offset or otherwise
  modify the circle using it.

## Point provenance and Rust destination

`Support/TreeSupport.cpp:1174–1178` collects `m_vertical_enforcer_points` via
`PrintObject::project_and_append_custom_facets`. `PrintObject.cpp:5494–5519`
selects strict support-painted facets of model parts and uses the object slicing
transform composed with the volume matrix. `TriangleMeshSlicer.cpp:1973–1997,
2253–2278` scales XY but not Z, classifies vertical faces by the integer XY
cross product with mirror sign, excludes coincident transformed vertices, and
stores each remaining vertical triangle's centroid and normal.

Ares already collects the same points: `project_slice/support_material/project.rs:98–119`
keeps volume annotations separate from the call for strict enforcer paint (state
1). `facet_projection.rs:30–91` selects painted model parts, applies the centered
object and volume transforms, scales XY only, and retains the vertical results
only for enforcer paint. `mesh_slicer/slabs.rs:87–105,147–165` implements the
same orientation classification and centroid/normal calculation. Do not change
this collection or its non-Organic consumer.

Pass the existing points and object layer planes to the Organic `TreeInput`.
Append in `tree/organic/generation.rs::detect` after the existing
`SmartOverhangs` polygons, before `generate` computes the highest overhang for
precalculation. The existing smart-overhang adapter remains the compatibility
boundary for this upstream route. No changes to paint decode, projected volume
annotations, blockers, sharp-tail contacts, non-Organic support geometry, or G-code
comparison are included.

## Verification

Start from `8b2829635`: the same painted Slim project already has strict parity
there, and the discovery comparisons showed no improvement from stacking the
extra-wall, interlaced-angle or bottom-contact changes. No prerequisite is
expected. Exercise the public project-to-G-code API and CLI against pinned Orca
using a synthetic T-shaped project with two painted vertical triangles and
manual tree support. Explicit Organic and Tree Default must print support and
match the complete Native output. Slim is the control for the Organic change:
it can print support through its separate contact path and must continue to
match Native output on the identical mesh and paint. Compare base and tip across
the support inventory with both the official and seeded oracles.
