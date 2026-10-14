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
| tess.c:345–579, vertex collection/cache/polygon pipeline | kernel.rs::tessellate/add_vertex |
| mesh.h:110–190 and mesh.c:69–543/603–649, half-edge navigation and topology | mesh.rs::Mesh and edge/vertex/face records |
| normal.c:76–169/198–257, automatic normal/projection/orientation | projection.rs::project/normal |
| geom.c:47–264, ordered predicates and stable intersection interpolation | predicates.rs::leq/eq/eval/sign/intersect |
| priorityq.c:92–261, source quicksort/insertion order and event merge | queue.rs::Queue |
| priorityq-heap.c:96–256, dynamic event heap and deletion handles | queue.rs::up/down/insert/delete/extract_heap |
| dict.c:73–111, ordered circular-list dictionary | regions.rs::insert_before/search/delete_region |
| sweep.c:107–409, active-region winding and left/right topology | regions.rs::Tess |
| sweep.c:438–452/499–603/760–844, splice repair and dirty-region traversal | splice.rs::Tess |
| sweep.c:454–497/606–758, exact intersection/combine payload | intersection.rs::Tess |
| sweep.c:847–1196/1199–1361, sweep events/sentinels/degenerates | sweep.rs::Tess |
| tessmono.c:72–147, monotone subdivision triangulation | monotone.rs::triangulate_interior/triangulate_face |
| render.c:82–330/362–502, maximum fan/strip/triangle grouping and cache rendering | render.rs::Tess/cache |

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
has approved stacking separately committed/reviewed B-T and B-S
rather than adding a dummy consumer, exposing unrelated internals, adding a
cfg exclusion or suppressing dead_code. No B-S implementation is authorized
by this B-T GREEN. Non-test strict gates remain deferred until the real B-S consumer joins this stack.

CI self-hosted native and parity jobs now check gcc/g++, and sparse checkouts
include original deps_src/glu-libtess. Both canonical-mode CI parity jobs run
the 19 core comparisons with their mandatory Native environment before CLI
parity, so the runtime C supplier build is actually exercised. The canonical
local full script is unchanged. No workspace/full/push is run by this worker.

The first formal GREEN at bd759a6c2 passed official19/19 and seeded19/19
runtime-Native nextest comparisons (21 Native executions per leg, including
A–B–A). Root /tmp/worker3-glu-formal-green-3hnnfvx8 retains compiler/raw/bit
artifacts. Actual-kernel WASM check passed. Strict kernel lint found a manual
swap; it is replaced with Vec::swap in a following commit, preserving the
source swap and RNG pivot exactly. No assertion/input/oracle changes.
Kernel and core gates remain distinguished: isolated kernel strict/WASM do
not claim whole-core strict while the approved B-S consumer is absent.
