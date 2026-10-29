# Proposed slice: high-valence mesh edge pairing

Status: docs-only SPEC review request. Worker3. Base is code-approved bottom
candidate `2fa5a5486d228a49f26ddb9512d8eac2cd190ed8`, awaiting stacked full.
The bottom branch/worktree stays frozen. This proposal lives separately in
/tmp/ares-worker3-mesh-edge-spec, branch pi/worker3-mesh-edge-pairing-spec.
Carry only this proposal onto the next broadcast main before implementation.
No Native, build, test, fixture or production change is included here.
Filament selectors remain reserved for W6; no skirt/counterbore overlap.

## Remaining difference and owning source

Current `mesh_slicer/topology.rs::index_mesh_edges` rejects any undirected
vertex-index edge with more than two uses at reject_non_manifold_groups:116-128.
Its accepted path assigns one ID to a one/two-use bucket (48-78).
Inventory row09 identified this guard but did not establish Native admission.

A narrower owner than a complete repair lifecycle is available in pinned
OrcaSlicer/src/libslic3r/TriangleMesh.cpp:

- EdgeToFace/create_edge_map:520-563: produce three oriented uses per selected
  face, sort vertex endpoints, retain signed local-edge direction, and sort
  the map by vertex_low/vertex_high.
- its_face_edge_ids_impl:565-613: visit each unconsumed use; search the rest of
  its equal-edge bucket for an unconsumed opposite direction (579-586), then
  an unconsumed equal direction (587-599); assign one new ID to the first use
  and its found partner (601-609), mark partner consumed, increment per pair
  or singleton. The comment591-593 explicitly anticipates edges used by more
  than two facets. It does not assign one ID to an entire high-valence bucket.
- its_face_edge_ids wrappers:616-629: all-face, cancellation and face-mask
  variants share this owner.
- TriangleMeshSlicer.cpp::slice_mesh:1865-1889 feeds this map into
  slice_make_lines. Its comment1876-1879 explicitly relies on a single ID
  belonging to at most two triangles. The single-plane face-mask variant
  calls the same owner at1982.
- Format/bbs_3mf.cpp:4975-5010 copies indexed triangles/vertices, validates
  indices and constructs TriangleMesh. Imported 3MF inputs can retain these
  shared vertex-index edges; reachability must still be demonstrated at runtime.

This is a libslic3r mesh-edge/slicer rewrite slice. TriangleMesh::repair and
its admesh neighborhood operations remain outside its implementation boundary.

## Rust target and scaffold

Keep the platform-neutral leaf in ares-core/mesh_slicer/topology.rs, preserving
MeshTopology's face_edge_ids and edge_count consumers. Replace the one-ID bucket
scaffold with source per-use consumption and pair/singleton ID assignment;
remove the >2-use guard only after the complete pairing consumer has public RED.
Reuse existing checked edge-ID range/resource checks and intersection/loop
assembly. Do not deduplicate facets, collapse all uses into one edge, perform
geometry repair, introduce another mesh representation or add a replacement
Unsupported guard. Keep every Rust file <=400 LOC; split this leaf only if needed.

The current Rust sort includes face/local-edge tie breaks; C++ comparator only
orders endpoints and std::sort is not stable. Pairing order is part of the
parity proof. Do not assume these tie orders are equivalent. Test face-order
permutations and a larger repeated bucket. If equal-key sort ordering affects
public output and requires porting a platform-library sort implementation,
stop for a SPEC addendum rather than silently inventing a tie rule or expanding
this slice. Invalid vertex/resource boundaries stay unchanged.

## Included, deferred and preflight

Proposed initial tracer: one 3MF model-part mesh containing two closed prisms
that touch along a vertical edge and explicitly share its two vertex indices,
creating four oriented uses. Fixed-height by-layer, one tool/object/instance,
ordinary Classic walls, no support/brim/skirt/tower/painting. Compare with an
input having identical world coordinates but distinct indices on that edge.

After SPEC approval and an open CPU12 directed window, require official/seeded
Native exit0/result0 on both inputs, then public core/CLI strict output and typed
mesh_topology rejection on the shared-edge input. Admission is a hypothesis,
not a claimed result in this doc. If Native rejects, repairs it differently,
produces unstable output or the expected public RED cannot be established,
retain roots and stop before production. Do not change the oracle or reproduce
a Native crash/assertion as a portable guard.

Then cover mixed/opposite and equal-orientation pairing, three-use odd leftovers
using a nonzero-area appended face, four/six-use groups, face-order permutations,
exact repeated facets and same-geometry split-index controls. Include one
Arachne output control. An isolated collapsed facet is only a diagnostic
admission question; it must not be silently admitted as a new repaired mesh.
Retain original manifold/open-boundary public controls and raw output bytes.

Defer hole filling, nearby-vertex welding, normal repair, vertex deduplication,
non-index-sharing geometric collisions, arbitrary mesh repair, painting/face
property reindexing, cuts and new face-mask public APIs. The existing Rust
slicer dispatch mode is unchanged; don't add a new public input contract.

## RED, retirement and verification

Inputs are deterministic generators/provenance only. Commit public Native-backed
RED separately and report it before GREEN. All runtime executions outer taskset
CPU12, jobs1/threads2, fresh Native cache/artifact root per run, START/END inbox.
Record original input SHA, Native exit/result, full G-code SHA, API/CLI equality
and first strict mismatch. Only existing generator-line comparison is permitted;
no geometric normalization, probes, retries, ignored tests or Native products
in git. If a private isolated test is essential, first enumerate its failure
modes and obtain the agreed seam; public E2E is the default proof.

Existing negative tests in mesh_slicer/tests/topology.rs and dispatch.rs expect
this blanket guard. Inventory their exact assertions after RED, including the
collapsed-facet expectation, and request separate precise retirement approval;
do not delete their unrelated budget/dispatch/manifold assertions automatically.
GREEN must prove the complete source pairing consumer, not just guard removal.
Run directed both modes, retained affected tests and strict/fmt/wasm before
REVIEW. Workspace/full and push require coordinator allocation.
