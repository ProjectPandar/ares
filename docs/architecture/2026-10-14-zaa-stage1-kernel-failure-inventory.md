# Approved ZAA PLAN90381524d: stage-one kernel failure inventory

Write before isolated kernel tests and production. Public RED6d2b6f37a already
proves the Native/application gap; these isolated tests do not replace it.
Coordinator allows Tree3f/IndexedMesh/contour now, not fill carrier migration.
Fill migration waits W5 zag/W6 Triangles stack main, gets its own mechanical
commit and both-mode evidence, and preserves Vec capacity/no per-point copies.

Source: AABBTreeIndirect.hpp Tree3f/build/QuickSelect/first-hit;
SLA/IndexedMesh.cpp:17-43,139-171; ContourZ.cpp:33-159.

Failure cases before implementation:

1. Empty tree query or no triangle hit incorrectly becomes a finite hit.
2. Behind-origin or coplanar triangles accepted despite positive-t/epsilon.
3. Nearest hit selection changes with face ordering, partition or CPU count.
4. Equal f32 distances incorrectly replace the first/left hit.
5. Back-facing triangle wrongly culled; source tests both determinant signs.
6. Source f32 mesh/normal/distance are replaced with double-only geometry.
7. Axis-parallel ray/bbox NaNs are mishandled relative to source slab tests.
8. Tree shape, longest-axis ties or median-of-three partition differ.
9. Contour clamp changes top/wall/ironing height bounds or wall-up exclusion.
10. No-hit or far-from-top points change the original path.
11.0.1mm sampling and collinear collapse lose an original path junction.
12. Per-point heap allocation is introduced in query/sample loops.
13. First physical layer, noneligible roles or disabled regions are contoured.
14. Object XY centering/ground frame or face winding changes normals.

Use small independent analytic triangles/planes for nearest-hit, backface,
no-hit, ties and height bounds, asserting source-defined outcomes rather
than reimplementing the algorithm in expected values. Rebuild the same tree
and compare complete nodes/results to catch unstable topology; construction
and recursion are serial with no available_parallelism. Public Native E2E
remains the eventual strict whole-output authority. No new guards/fallback,
assertion weakening, suppressions, full/workspace or pushes.
