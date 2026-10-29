# Diagnostic: mirrored-instance triangle winding at the slicing boundary

## Witness

`/tmp/w5-x/target-mirror.3mf` (not yet frozen): the crosshatch-ordering RED
target with the second build item's transform mirrored in X
(`transform="-1 0 0 0 1 0 0 0 1 170 110 0.5"`), first item unchanged.
OrcaSlicer 2.4.2 official vs Ares `5df60edc0`:

- 206 diff lines; line counts differ (native 5088, ares 5098) — a structural
  divergence, not just ordering.
- First body divergence: the mirrored object's FIRST inner wall
  (`;TYPE:Inner wall`, right after `printing object cube10.stl id:0 copy 0`).
  Native starts the loop at `X181.63 Y104.37` and runs
  `181.63→172.37 (bottom, leftwards) → 115.63 → 181.63` — ares starts at
  `X172.37 Y104.79` and runs the loop in the OPPOSITE direction with a
  different closing seam (`E.28453/.34599` vs `.33308/.28453/.34599/.01168`).
  The outer wall, wipe path, and all downstream travels follow.
- The unmirrored first object and every translation/rotation-only variant
  stay byte-identical (existing legs PASS).

## Root cause (source-cited)

`PrintObjectSlice.cpp:86-92` (`slice_volume_slicing_...` per-volume mesh
preparation):

```cpp
params2.trafo = params2.trafo * volume.get_matrix();
if (params2.trafo.rotation().determinant() < 0.)
    its_flip_triangles(its);
layers = slice_mesh_ex(its, zs, params2, ...);
```

When the combined slicing trafo (object trafo × volume matrix) is
left-handed (negative 3×3 determinant — any mirrored instance), upstream
SWAPS the triangle indices before slicing so slice contours keep their
orientation and the perimeter/seam machinery sees the same winding as an
unmirrored object.

Ares' counterpart (`crates/ares-core/src/project_slice/raw_intersections.rs:
181-197`) computes `slicing_transform = centered_object_transform
.then(source_volume.transform()).prescaled_xy(scale.factor())`, transforms
the VERTICES, and passes `source_volume.mesh().triangles()` through
UNCHANGED — no determinant check, no index swap. A mirrored instance
therefore slices with inverted winding: slice contour orientation flips,
which reverses perimeter loop direction and displaces the seam/closing
point, exactly the witness.

## Fix boundary (GREEN proposal)

- Upstream owner: `PrintObjectSlice.cpp:88-90` (determinant test +
  `its_flip_triangles`), with `TriangleMesh.cpp:337-347`
  `TriangleMesh::transform`'s `fix_left_handed` as the same convention.
- Ares destination: `project_slice/raw_intersections.rs` — after building
  `slicing_transform`, compute its linear-part determinant (exact sign,
  mirroring upstream `rotation().determinant() < 0.`); when negative, swap
  each triangle's first two indices for the slicing input. The flip applies
  to BOTH slice entry points fed from there (`ranges::slice_volume` and
  `mesh_slicer::slice_mesh_on_planes`).
- No epsilon/tolerance: upstream compares the raw double determinant sign
  of the rotation part; ares must derive the same sign from its
  `Transform3d` linear block (the `prescaled_xy` positive-uniform XY scale
  does not change the sign).

## RED proposal

Freeze `target-mirror.3mf` as `tests/parity/crosshatch-ordering/
target-mirror.3mf` with a leg mirroring the existing witnesses: pin the
native first inner-wall order (body coordinates: the loop's first segment
`G1 X172.37 Y104.37 E.28453` follows the `X181.63 Y104.37` travel, i.e.
leftwards along the bottom edge first) plus the whole-output strict compare
(DIVERGENT until GREEN). Controls: the existing four legs stay PASS.

## Amendment (approved 2026-10-18): owner corrected to SeamPlacer occlusion mesh

The GREEN round falsified the slicing-boundary premise as the *observable*
owner: flipping triangles at `raw_intersections.rs` fires (det=-1e12 on the
mirrored volume) but ares output stays byte-identical, because BOTH slicers'
post-union contours are winding-blind — the general-case intersection line
endpoints are vertex-id normalized (upstream `TriangleMeshSlicer.cpp`
`slice_facet`, "Sort the edge to give a consistent answer"; ares
`mesh_slicer/intersection.rs` does the same), and the subsequent polygon
union rebuilds contour orientation regardless of line direction.

Native probe (env-gated `ORCA_SEAMDUMP` in `SeamPlacer.cpp place_seam`,
official-identical gcode body) proved the divergence enters exclusively
through seam-candidate VISIBILITY: obj1 and obj2 have identical loop vertex
sequences and identical candidate positions/angles, but different
visibility values (e.g. corner 0: 0.918239 vs 0.836138) → different
`pick_seam_point` choice (idx1 vs idx2) → the witness walls.

Corrected upstream owner: `SeamPlacer.cpp compute_global_occlusion`, TWO
`fix_left_handed` flip sites:

- `SeamPlacer.cpp:638-640`: `its_transform(model_its,
  model_volume->get_matrix(), true)` — "Mirrored transforms flip winding,
  keep normals outward" (per-volume matrix, applied before merge).
- `SeamPlacer.cpp:659-660`: `its_transform(triangle_set, obj_transform,
  true)` — "Mirroring flips normals, keep them outward for visibility
  sampling" (whole merged set, applied after merge).

Flip semantics: the two sites use DIFFERENT swap conventions and the
distinction is observable through barycentric visibility sampling:
- SeamPlacer sites use `its_transform(..., fix_left_handed=true)`
  (admesh `stl.h:362-367`), which swaps indices 0 and 1 — verified by
  GREEN: swapping (1,2) instead yields visibility values that diverge from
  the native probe, swapping (0,1) reproduces the native mirrored-object
  visibility exactly (e.g. 0.918239, 0.797810, 0.830911, 0.838843).
- The slicing flip uses `its_flip_triangles`
  (`TriangleMesh.cpp:737-740`), which swaps face(1),face(2).

Ares gap: `seam_placement/mesh.rs:158-199 TriangleMesh::from_object`
transforms vertices only and never flips triangle indices.

### Corrected fix boundary (GREEN)

- `from_object` keeps the two-site upstream structure: flip each volume's
  triangles when `det(volume.transform()) < 0`; after merging all volumes,
flip the whole set again when `det(object_transform) < 0`. Both flips swap
  indices 0 and 1 (`its_transform` convention). Net parity is
  `sign(det(volume) · det(object_transform))`, but the structure mirrors
  upstream site-for-site.
- The `raw_intersections.rs` flip (PrintObjectSlice.cpp:88-90) stays:
  upstream-required even though unobservable in this witness, because the
  witness geometry is X-symmetric (mirror = identical squares) and
  post-union slice contours are winding-blind on both sides; other
  geometries can surface it.

## Included / deferred (amended)

- Included: the winding flip at the slicing boundary, the mirror witness
  leg, regressions (crosshatch suite both modes, brim/infill/crosshatch
  official, clippy, fmt, wasm, file-size budget).
- Deferred: seam painting interplay on mirrored geometry (unchanged by this
  slice — the flip restores the same winding seam placement consumes
  upstream), support/ironing (already byte-identical on two-object sharing
  probes, 0 non-header diffs, verified 2026-10-18).
- Deferred (amendment, checked 2026-10-18): `from_object` does not build the
  `NEGATIVE_VOLUME` occlusion set (upstream gathers
  `ModelVolumeType::NEGATIVE_VOLUME` meshes separately,
  SeamPlacer.cpp:633-646) and does not run `its_short_edge_collpase`
  decimation to `fast_decimation_triangle_count_target`
  (SeamPlacer.cpp:650-653); ares has no negative-volume or decimation
  handling elsewhere in `seam_placement/` (grep 2026-10-18). Both are
  pre-existing gaps independent of winding and out of this slice.
