# Requested addendum: resolve mesh edge ordering before RED/GREEN

Status: STOP, docs-only SPEC supplement for coordinator review.
Original approved SPEC49c31802f, rebased doc357786fc7 on main
`de1211be5067d0eb6f709602a1299a0854c89145`, range-diff1/1 '='.
No public RED, existing assertion change, Ares implementation or push occurred.
All Native batches ended; CPU12 only, fresh cache/artifact roots, no retries.

## Observations and exact input revisions

First root: /tmp/worker3-edge-native-preflight-t8_h596m/evidence.json.
One-volume pair of closed prisms touching along a vertical edge, shared vertex
indices ->4 uses. Compare identical world coordinates with split indices, and
shared input face lists reversed/interleaved/rotated. Five inputs per Native
mode, all exit0/result0. All ten outputs share generator-only SHA
6480e4c1a1ec8b2cc275cd1b090bcf415d6a7ddfc5957f265264aed0b0927aec.
This proves admission for the proposed first tracer, not all equal-key pairing.

Second root: /tmp/worker3-edge-native-order-risk-xz5652uy/evidence.json.
Input revisions deliberately exercise risks listed in the approved SPEC:

- Cube control: one prism with2 uses per ordinary edge.
- Odd-use inputs: that cube plus one nonzero-area face on its vertical edge,
  giving3 uses. Original/reversed/rotated face order keep vertices and the
  oriented triangle multiset exactly the same; verified in
  triangle-multiset-audit.json (all SHA330216f23d9abd53fc505c21b68756767131e237e750233fcf69dd12901e6e74).
- Six-use inputs: duplicate the first prism's faces in the original two-prism
  mesh. This increases valence without changing vertex coordinates; use
  original/reversed/rotated face order.
- Collapsed-facet diagnostic: append (0,0,0) to the first tracer. It remains
  diagnostic, not a claimed newly supported repair case.

Each mode: six exit0/result0 and two deterministic admission failures.
Odd-original returns0 (generator-only SHA92de92462c3bde6deacb42f3c6e78d4f3512f6963f24905a33ec299820945e31);
odd-reverse and odd-rotated return -100 in result.json, process156 on Unix,
"Failed slicing the model. Please verify the slicing of all plates on Orca
Slicer before uploading." No G-code exists for these two. Six-use and collapsed
cases match the first tracer's generator-only SHA. These are per-input
observations, no retries and no normalization except the permitted generator
line. The two failures are diagnostic admission results, never parity PASS.
No root/input was overwritten or discarded.

## Why STOP applies

TriangleMesh.cpp:520-563 EdgeToFace compares only vertex_low/vertex_high,
then calls std::sort; its_face_edge_ids_impl:565-613 consumes neighbors in that
result order. Current Rust sort additionally orders face/local-edge indices.
Public Native sensitivity to face order is now demonstrated for an odd bucket,
but this does not isolate std::sort as the cause: source triangle traversal and
TriangleMeshSlicer.cpp::make_loops:1383-1482 also depend on connectivity/order.
Do not declare a Native instability, invent a tie break, add an odd-valence
Unsupported guard or reproduce -100 by a new admission rejection.

The approved SPEC says to stop for an addendum when equal-key ordering affects
public output and the source order cannot safely be established. That condition
is unresolved, so production remains stopped.

## Proposed E0: isolated source-order diagnostic, approval requested

Before any Rust change, resolve create_edge_map's ordering owner with a bounded
diagnostic seam. This adds no slicer API or public capability.

1. Read-only pin each canonical producer's compiled EdgeToFace/std::sort
   instantiation through symbols/disassembly and available build provenance.
   ELF comment/dynamic snapshots are retained in the second root. Official
   comment includes Ubuntu GCC14.2/clang18.1.3; seeded includes GCC15.3/clang21.1.8;
   both need libstdc++. Comments do not prove the exact per-TU header version.
2. Audit available GNU15.3 headers (source identity/hash must be recorded):
   bits/stl_algo.h:88 __move_median_to_first;1806-1822 threshold16/final insertion;
   1851-1858 pivot;1876-1908 introsort/depth/heap fallback and final sort.
   Locate/prove the official producer's corresponding implementation before
   asserting a shared policy. Runtime libstdc++ alone cannot pin this inlined
   template, and no canonical wrapper/library may be changed.
3. If approved, use a standalone source replay of TriangleMesh.cpp:520-613,
   with original four-int EdgeToFace/comparator and std::sort, to write ordered
   uses and face-edge-ID receipts for authored input triangles. This is an
   isolated diagnostic reference, NOT a call to the canonical Native slicer
   and NOT a Native parity PASS. Replay outputs remain /tmp; no production
   probe, Native binary patch, Rust implementation or modified slicer build.
4. Compare replay mapping and source-policy variants, retaining differences.
   Only proceed to a separate production addendum if the canonical ordering
   can be pinned and both producer policies agree or their public differences
   are explicitly adjudicated. Otherwise keep STOP and report the missing fact.

The isolated seam is necessary because whole G-code/error channels cannot
identify which mapping/order caused this failure. Failure modes to enumerate
before replay code: wrong stdlib/header/instantiation; accidental stable-sort
or face-ID tie break; threshold/median/equal-swap/depth/heap differences;
consumed partner reuse; same-direction fallback selected too early; odd-use
singleton numbering; duplicate/degenerate same-face uses; index conversion;
source triangle winding or index preprocessing changed before the replay.
Use opposite/equal directions, repeated keys, permutations, odd/even buckets
and threshold/depth boundaries. Preserve cancellation/face filtering scope;
all-face replay must not be passed off as an audit of the face-mask API.

## Future production boundary, not approved by this supplement

If E0 shows a library-sort rewrite is necessary, propose private
mesh_slicer/topology/sort.rs only for EdgeToFace uses, anchored to the
TriangleMesh.cpp create_edge_map call. Include actual canonical source lines,
policy and preserved invariants before approval. No generic sorting utility,
new public contract or independently designed Ares pipeline.

After that approval, public RED still comes first and independently compares
accepted inputs against both canonical Native modes. The two Native -100 cases
stay recorded as diagnostics unless their admission/error boundary receives
separate review; don't remove a committed failing parity case (none yet exists).
If matching accepted cases exposes a new make_loops/error-channel owner, stop
for that source-cited boundary as well. Negative unit retirement still requires
its separate exact request after public RED. No implementation before ruling.
