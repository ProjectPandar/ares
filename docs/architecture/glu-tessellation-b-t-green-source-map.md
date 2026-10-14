# B-T GREEN: original GLU source map

Approved RED7942e79ad uses a runtime original glu-libtess supplier. This
production port replaces the missing private geometry operation; it does not
change Support preparation, octree inputs, bridge-candidate ownership or
public load/selection contracts. B-S remains the real pipeline consumer.

## Source and Rust ownership

All destinations below are under
crates/ares-core/src/geometry/tessellation/ unless otherwise named.

| Original source operation | Rust destination |
|---|---|
| Tesselate.cpp:23–56/100–170/192–195, contour/holes/z/flip | geometry/tessellation.rs::triangulate_expolygon_3d and render.rs::Emitter |
| tess.c:337–457/500–579, vertex collection/cache/polygon pipeline | kernel.rs::tessellate/add_vertex |
| mesh.h:110–190 and mesh.c:58–483, half-edge navigation and topology | mesh.rs::Mesh and edge/vertex/face records |
| normal.c:76–172/200–257, automatic normal/projection/orientation | projection.rs::project/normal |
| geom.c:47–264, ordered predicates and stable intersection interpolation | predicates.rs::leq/eq/eval/sign/intersect |
| priorityq.c:89–261, source quicksort/insertion order and event merge | queue.rs::Queue |
| priorityq-heap.c:94–256, dynamic event heap and deletion handles | queue.rs::up/down/insert/delete/extract_heap |
| dict.c:59–111, ordered circular-list dictionary | regions.rs::insert_before/search/delete_region |
| sweep.c:59–280, active-region winding and left/right topology | regions.rs::Tess |
| sweep.c:287–456/672–737, splice repair and dirty-region traversal | splice.rs::Tess |
| sweep.c:318–365/464–660, exact intersection/combine payload | intersection.rs::Tess |
| sweep.c:743–1115/1190–1361, sweep events/sentinels/degenerates | sweep.rs::Tess |
| tessmono.c:59–149, monotone subdivision triangulation | monotone.rs::triangulate_interior/triangulate_face |
| render.c:64–322/371–502, maximum fan/strip/triangle grouping and cache rendering | render.rs::Tess/cache |

Indices replace client/internal pointers while preserving linked-list order,
paired half-edge order and source insertion/splice/delete order. Removed records
leave the live lists; their per-call storage is released together by Rust.
There is no unsafe, allocator/address logic, sorting by object addresses,
ear clipping, external triangulator, TS or runtime Native dependency.
The original sorted priority queue preserves equal-key swaps and insertion
cutoff. Its deterministic seed uses wrapping u64 to match the canonical LP64
Native build on all Ares targets. Heap-created events retain source handle reuse.
Intersection coordinates preserve GLfloat weight storage before GLdouble
accumulation. Callback expansion preserves complete ordered triples and z.
No contour/hole normalization or tolerance-based comparison is introduced.

The default GluTessWrapper fixes the represented GLU configuration: automatic
normal, odd winding, no boundary-only/edge-flag/mesh callback. Generic GLU
property setters, other winding modes, client-error callbacks and arbitrary
state transitions are outside this wrapper slice. Rust owns allocation/drop;
C allocation-failure longjmp paths are not compatibility fallbacks. Existing
integer coordinates cannot exceed GLU_TESS_MAX_COORD after unscale.
SGI Free Software License B notices and the complete original license are
retained in tessellation/LICENSE and source module headers.

## Verification and staging

A rapid driver compiled the actual kernel source on CPU12, then regenerated
Native expectations from the original C kernel for every authored input:
18/18 ordered coordinate-bit matches, including holes, combine crossings and
101-vertex cache flush. Root /tmp/worker3-glu-kernel-green-f6zag709 retains
requests and both runtime outputs. This is diagnostic evidence; the approved
19 nextest comparisons and exact-head gates are still required.

The independent B-T operation has no production caller until B-S. Coordinator
ruling is requested on stacking separately committed/reviewed B-T and B-S
rather than adding a dummy consumer, exposing unrelated internals, adding a
cfg exclusion or suppressing dead_code. No B-S implementation is authorized
by this B-T GREEN. Non-test strict gates remain pending that boundary ruling.

CI self-hosted native and parity jobs now check gcc/g++, and sparse checkouts
include original deps_src/glu-libtess. Both canonical-mode CI parity jobs run
the 19 core comparisons with their mandatory Native environment before CLI
parity, so the runtime C supplier build is actually exercised. The canonical
local full script is unchanged. No workspace/full/push is run by this worker.
