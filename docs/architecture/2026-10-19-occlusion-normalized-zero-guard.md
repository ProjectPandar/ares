# Occlusion visibility float semantics: Eigen zero-guard normalization

Follow-up evaluation of the four upstream float-semantic deltas deferred
from the decimation GREEN (73a3a0766), plus one new divergence found
during the search with a frozen-able witness.

## Part 1 — New divergence: `Vec3::normalized()` NaN on zero vectors

### Upstream boundary

`SeamPlacer` occlusion visibility uses `normalized()` on vectors that can
be exactly zero. Eigen (5.0.1, `Eigen/src/Core/Dot.h:92-102`) defines:

```cpp
RealScalar z = n.squaredNorm();
if (z > RealScalar(0))
  return n / numext::sqrt(z);
else
  return n;            // zero vector returned unchanged, NOT NaN
```

Reachable upstream sites that rely on this guard on decimated meshes:

- `TriangleSetSampling.cpp:67` — `result.normals[s_idx] =
  ((B - A).cross(C - B)).normalized()`. `its_short_edge_collpase`
  extraction filters only index-degenerate faces
  (`ShortEdgeCollapse.cpp:171-186`); faces with three distinct indices at
  coincident positions survive with an exactly-zero cross product, so the
  sample normal is the zero vector, and Eigen keeps it `(0,0,0)`.
- `SeamPlacer.cpp:74-77` — `Frame::set_from_z` then propagates the zero
  normal (`mZ = z.normalized()` stays zero; `mY = (tmpZ.cross(tmpX)`
  `.normalized())` stays zero; `mX = mY.cross(mZ)` stays zero), so the 25
  ray directions from such a sample are `(0,0,0)` and `intersect_ray_first_hit`
  is called with a zero direction.
- `TriangleMesh.hpp:329-332` `its_face_normal` on zero-area faces
  (double `.normalized()` of the same guarded routine).

### Ares boundary

`crates/ares-core/src/project_slice/seam_placement/mesh.rs`
`Vec3::normalized()` computes `self / self.norm()` — IEEE `0.0/0.0 = NaN`
for zero vectors. With NaN sample normals, `GlobalVisibility::at`
(seam_placement/visibility.rs) returns NaN weights, the affected corner
penalties become NaN, and the cross-layer seam champion flips to the
opposite diagonal corner.

### Witness (RED outcome 2026-10-19)

Fixture family: the subdivided-plate generator from the decimation slice
at grid n=72 (21888 triangles > 16000) with slight X asymmetry
(w=30.03mm, e.g. `cand72a.3mf`,
sha256 7da16537d55b7f82ed42a8810bac4f3a080913790d25737eecf31fbfa7dc592a,
local build only until frozen). The decimated mesh retains 153 exact
zero-area faces (three distinct indices, coincident positions).

- Native (official 2.4.2): 0 NaN-normal samples of 30000; picks the
  asymmetric corner; per-corner visibility layer 1:
  `[0.899170, 0.855406, 0.896867, 0.856266]`, seam = candidate i=1.
- Ares at 73a3a0766: 342/30000 samples have NaN normals
  (`plane_distance=NaN`, `weight=NaN` in `GlobalVisibility::at`), the two
  left corners score NaN, and the layer champion flips to the diagonally
  opposite corner — G-code diverges at the layer-1 travel
  (`G1 X95.615 Y124.37` native vs `G1 X124.385 Y95.63` ares), ~2980
  diff lines. Stable across 3 native re-runs.
- With only the zero-guard fix applied to `Vec3::normalized()`
  (`if z > 0 { self / z.sqrt() } else { self }`), ares is byte-identical
  to native on this fixture (0 non-header diff lines), and the frozen
  `target-decim.3mf` leg still passes (crosshatch_ordering 6/6, official).

Included: change `Vec3::normalized()` in seam_placement/mesh.rs to the
Eigen-guarded form. Deferred: none — this is a single guarded branch with
a G-code witness; other `normalized()` call sites only change behavior on
zero vectors, which currently produce NaN.

## Part 2 — The four ablated deltas: no G-code witness found

Search space (all sliced against the official 2.4.2 oracle; criterion:
stable byte divergence of `ares-base` where a single one of the four
fixes restores parity):

1. Grid density sweep n∈{65..84, 90..102} × plate symmetry
   {30.0, 30.03, 30.05, 30.1, 29.97, 30.06} — including the exact-tie
   symmetric square and fine asymmetry steps (0.002–0.02) around
   symmetry at n∈{70,72,74}.
2. Flipped duplicate wall faces (0–200 duplicated triangles) to force
   equal-`t` first-hit ties between opposite-orientation coincident
   faces — rejected by the native slicer itself (repair failure), not a
   legal witness.

Result: ~100 candidates, all byte-identical to native at 73a3a0766
(after Part 1's zero-guard is accounted for). The four deltas remain
observable only at the per-sample visibility level (14/30000 sample
values on the frozen `target-decim.3mf`, vanishing with the fixes,
dcprobe-verified), i.e. real upstream-semantic differences whose
decision margin never crosses a G-code observable in this fixture
family. Per the no-witness-no-change rule they stay deferred:

- BVH split centroid `f32(1/3) * (v1+v2+v3)` (AABBTreeIndirect.hpp:706)
- first-hit `t = e2·q × (1/det)` multiply (AABBTreeIndirect.hpp:318-340)
- equal-distance tie keeps left child (AABBTreeIndirect.hpp:427-440)
- `its_face_normal` double normalization (TriangleMesh.hpp:329-332)

Any future witness must satisfy: native-vs-ares byte divergence at a
frozen canonical fixture that the single corresponding fix restores.
