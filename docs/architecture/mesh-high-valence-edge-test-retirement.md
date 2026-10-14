# Mesh-edge guard test retirement request

Status: proposed exact deletion; approval required before changing tests.
Public RED commit: d5d5bdcc1, approved SPEC49c31802f/addendum8b4162da3.
Production and all existing assertions remain unchanged at this request.

RED evidence: /tmp/worker3-mesh-edge-red-eco_8wg9/RED-receipt.json
and evidence.json. Official/seeded each: Native13 admitted, strict3 controls
PASS +10 mesh_topology guard RED. Two Native-rejected ordered inputs each
produce the same Ares guard, CLI1 and no G-code; diagnostic-only collection
is not a parity PASS. RED-REPORT was posted before any production change.

## Upstream conflict

TriangleMesh.cpp::its_face_edge_ids_impl:565-613 does not reject an indexed
edge with more than two uses. It connects each unconsumed use to an opposite
partner first (:578-587), then an equally oriented partner (:589-599), and
assigns a distinct edge ID per pair or unpaired use (:600-609). The comment
at :591-593 explicitly discusses more than two touching facets.
TriangleMeshSlicer.cpp:1876-1880 consumes those pair IDs.

Public RED requires complete Native result0 output for ten high-valence
inputs; the three manifold controls keep strict output comparison. The
Native-admitted collapsed diagnostic is retained separately in
/tmp/worker3-edge-native-order-risk-xz5652uy/evidence.json; both modes result0.
It is not an additional claim of mesh repair parity. Native-rejected odd
permutations are diagnostic-only and do not authorize an Ares rejection.

## Exact proposed deletions, separate test commit

1. In crates/ares-core/src/mesh_slicer/tests/topology.rs, delete only the
   whole `task22b_topology_rejects_more_than_two_uses_before_intersection`
   test: its local expected closure and two assert_eq statements expecting
   UnsupportedProjectFeature("mesh_topology") for the three-use edge and
   collapsed indexed triangle. No import removal: SliceError is still used
   by the preserved edge-ID range test.
2. In crates/ares-core/src/mesh_slicer/tests/dispatch.rs, within
   `task22b_multi_plane_slicing_is_repeatably_deterministic`, delete only
   the trailing guard-only block beginning `let nonmanifold_vertices`:
   its nonmanifold vertices/triangles/planes, untouched_budget, attempts,
   output setup, the dispatch assertion expecting mesh_topology rejection,
   and its three consequences (zero retained lines, empty attempts, empty
   output). Preserve the function name and every earlier statement and
   assertion proving multi-plane output/count determinism.

All opposite-orientation, same-orientation, boundary, deterministic ID,
edge-range, intersection budget and positive dispatch assertions stay
byte-for-byte unchanged. Do not replace these rejection assertions with
weaker success assertions or suppress them. The independently committed
public Native RED remains the new feature gate.

After exact approval, place these deletions in an independent test commit
citing TriangleMesh.cpp:565-613 and RED d5d5bdcc1, separate from GREEN.
No full/workspace or push is authorized by this proposal.
