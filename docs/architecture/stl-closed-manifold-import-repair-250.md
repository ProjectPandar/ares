# Closed-manifold STL import repair

## Source boundary

This slice rewrites the closed, exact-neighbor branch of OrcaSlicer's STL
import repair, not a new Ares mesh-processing pipeline. Source references are
to v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

The owning call chain is `Format/STL.cpp::load_stl` →
`TriangleMesh::ReadSTLFile` → `TriangleMesh::from_stl` → the static
`trianglemesh_repair_on_import`. `TriangleMesh::repair()` inside that routine
is log text, not its declaration.

| Upstream source | Included behavior | Rust destination |
| --- | --- | --- |
| [`Format/STL.cpp:18–40`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/Format/STL.cpp#L18-L40), `load_stl`; [`TriangleMesh.cpp:197–235`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/TriangleMesh.cpp#L197-L235), `from_stl` / `ReadSTLFile` | Repair after reading facets and before generating shared vertices/model-volume geometry. | `ares-core/src/stl.rs::load`, reached by public `load_model` and existing STL project loaders. |
| [`TriangleMesh.cpp:95–202`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/TriangleMesh.cpp#L95-L202), `trianglemesh_repair_on_import` | Exact connectivity → normal directions → normal values → volume/reversal, in source order on inputs not needing nearby or facet-removal stages. | Private `ares-core/src/stl/repair.rs` and focused children under `stl/repair/`. |
| [`deps_src/admesh/stl.h:58–100`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/deps_src/admesh/stl.h#L58-L100), `stl_facet`, `stl_neighbors` | Authored normal and ordered vertices; three neighbor indices and encoded opposite-vertex/direction state. | Private STL facet and neighbor data, not a new public model or configuration API. |
| [`connect.cpp:42–88,182–292,432–471`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/deps_src/admesh/connect.cpp#L42-L471), `HashEdge::load_exact`, `HashTableEdges::insert_edge` / `record_neighbors`, `stl_check_facets_exact` | Exact endpoint matching, signed-zero key equivalence, local-edge orientation and reciprocal neighbor encoding; facet/local-edge visitation order. | `stl/repair/connectivity.rs`; only its nondegenerate, exactly paired branch. |
| [`normals.cpp:35–240`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/deps_src/admesh/normals.cpp#L35-L240), `reverse_facet`, `check_normal_vector`, `stl_fix_normal_directions`, `stl_fix_normal_values`, `stl_reverse_all_facets` | Seed-normal comparison, traversal, reversal and rollback, then normal recomputation. | `stl/repair/normals.rs`. |
| [`stl.h:392–401`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/deps_src/admesh/stl.h#L392-L401), `stl_calculate_normal`, `stl_normalize_vector`; [`util.cpp:248–297`](https://github.com/OrcaSlicer/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/deps_src/admesh/util.cpp#L248-L297), `get_area`, `get_volume`, `stl_calculate_volume` | Source arithmetic and whole-mesh negative-volume reversal. | `stl/repair/volume.rs`, sharing the private normal arithmetic. |

The existing vertex-only STL parser/indexer is a temporary compatibility shell
around this import boundary. Replace its loss of authored normals for this
repair path; do not add an alternative slicer or an output-only winding fix.

## Included mesh domain

The included branch has nondegenerate facets whose exact coordinate edges each
have two uses belonging to distinct facets. Matching uses compare endpoint
coordinates, not pre-existing shared vertex IDs. Winding may be inconsistent;
closed disconnected components are included. This is the edge-neighborhood
condition relevant to the source repair, not a new public assertion that every
input is an embedded, outward-oriented solid.

Retain facet order, vertex coordinates and the source's local vertex swap.
Do not sort facets, weld nearby vertices, deduplicate faces, translate geometry
or impose independent outward orientation on each connected component. Exact
edge keys treat negative and positive zero alike without rewriting vertex
coordinates. A hash table may use Rust storage/hashing: for the two-use domain,
pairing and visitation must retain the source neighbor semantics, independent
of table iteration order. Avoid quadratic all-facet neighbor searches.

Inputs requiring nearby matching, unconnected-facet removal or degenerate-face
repair are outside this branch. Their existing loading behavior remains an
explicitly unported boundary, not a retry/fallback implementation of those
upstream stages. Do not introduce a new public rejection or claim complete
repair for those inputs. Scope classification must precede mutation so an
excluded input cannot receive half of this repair sequence.

## Ordered repair semantics

1. **Exact neighbors.** Visit facets in input order and local edges 0, 1, 2.
   Preserve the source's canonical endpoint ordering and local-edge direction
   bit, reciprocal neighbor indices and `which_vertex_not` encoding. The
   repeated-vertex removal loop inside `stl_check_facets_exact` is deferred;
   it is not permission to silently remove or reorder facets in this slice.
2. **Normal directions.** Keep the authored normal from both ASCII and binary
   facets until the source algorithm has consumed it. Start with facet zero;
   when its work list empties, choose the first unvisited facet by index. Match
   the source's prepend/pop traversal, including repeated work-list entries.
   Check each component's seed normal rather than voting across its faces.
3. **Seed comparison.** Compute the vertex-derived cross product and normalized
   normal. `check_normal_vector` uses component-wise absolute differences with
   strict `< 0.001f`, then normalizes the authored normal and compares again,
   including its negation. An unknown direction is not a backwards direction.
   The first, already-within-tolerance match also replaces the stored normal
   even with `normal_fix_flag=0`; this is not a pure predicate.
   Normalization computes length in double precision, maps length `< 1e-12`
   to zero, and otherwise multiplies float components by a float reciprocal.
   Preserve these distinctions for zero, scaled and incorrect authored normals.
4. **Reversal and contradiction.** `reverse_facet` swaps vertices 0 and 1 and
   updates reciprocal direction/opposite-vertex state plus local neighbor slots.
   Track reversals in source order. If traversal would reverse an already fixed
   neighbor, undo the complete accumulated reversal list in reverse order and
   leave the direction pass. The list spans components; rollback is not limited
   to the component that exposed the contradiction. Continue with the following
   source stages, not a new input rejection or a second repair attempt.
5. **Normal values.** Run the source `check_normal_vector(..., normal_fix_flag=1)`
   pass after direction repair or its rollback. Do not replace authored normals
   before the seed-direction pass, which would erase information it uses.
6. **Volume.** Use `get_area`'s double coordinate products and three-term
   component sums, then convert each sum to its float vector component before
   the float dot/area calculation. Use the first facet's vertex zero as
   `get_volume`'s reference point. Height, area, `(area * height) / 3.0f`
   and accumulation are float-valued and processed in facet order. Only a
   strictly negative whole-mesh volume causes `stl_reverse_all_facets`, including
   its normal recomputation. Do not substitute a double signed-tetrahedron sum,
   change reduction order or reverse components independently.

`util.cpp::stl_verify_neighbors` observes connectivity and counts/logs backwards
edges; it does not repair geometry. Keep reversal bookkeeping consistent with
that source invariant without adding terminal logging or unused public stats
to core. The final degeneracy-triggered connectivity refresh belongs to the
deferred degeneracy slice.

## Rust integration boundary

Keep the existing public `load_model` result and `Model` / `Triangle` interfaces.
Private parser facets may retain normals until repair finishes, then transfer
the repaired ordered vertices into the existing model representation. No normal
field, repair option, feature flag or separate public repair API is needed.
Preserve existing input-format detection and malformed-input policy; do not
expand ASCII/binary grammar or add unrelated input validation.

All STL consumers must reach this shared load seam. Do not duplicate repair in
`project/domain/profile_input/stl.rs`, its vertex-indexing helper, plater batch
loading, WASM or JavaScript. Do not change centering, placement, ObjectID labels,
profile/config composition or instance/volume transforms. In particular, do not
copy multipart conversion into this slice or change the indexed 3MF constructor
into an STL repair caller.

Use owned per-import facet/neighbor/work-list storage, with no global mutable
state, native dependencies, filesystem/terminal access or unsafe code in core.
Keep changed Rust modules within 400 physical lines. The same memory-only
implementation must compile for browser WASM, Windows, macOS and Linux.

## Public regression and failure inventory

The primary observation seam is public `ares_core::load_model`: provide the
same complete synthetic STL bytes to Native import and Ares, then compare
ordered facet vertex triples. Native's actual STL import/export path can retain
its repaired result as an STL artifact (`OrcaSlicer.cpp:6398–6413` export action;
`Format/STL.cpp::store_stl`). Qualify that transfer's coordinate frame and facet
order before using it as an oracle. The test's Native-output decoder must read
raw facets without running Ares repair on the expected result. Do not sort,
normalize winding, cyclically rotate vertices or use an expected mesh generated
by the implementation under test. If export placement changes the comparison
boundary, stop and specify a source-bound import observation instead of hiding
that change behind normalization.

A downstream public profile-slicing control additionally checks that repaired
STL geometry reaches existing slicing. It is not a whole-G-code parity claim;
export-only geometry output must not be described as successful slicing.
A repair regression test must distinguish differing imported facet
geometry from compilation failures, Native premise rejection, placement
mismatches or unrelated configuration panics.

The behavior/control inventory is:

| Input/control | Failure it detects |
| --- | --- |
| Outward-wound asymmetric closed mesh; equivalent ASCII and binary encodings | Unconditional reversal, coordinate/order changes, format-path divergence. |
| Wholly inward winding, with consistent inward and deliberately wrong authored normals | Missing global reversal or discarding authored-normal semantics. |
| A closed component with a locally reversed facet, including the seed facet | Missing neighbor propagation, wrong local-edge remapping or seed handling. |
| Disconnected closed components of unequal volume, differing winding and zero/correct/opposite/scaled authored normals | Per-component outward heuristics, skipping component seeds, or a volume-only shortcut. |
| Near-opposite authored seed normals with orthogonal perturbations below and above the normalized 0.001 threshold | Non-source epsilon, inclusive comparison or replacement of normals too early. Use a smaller component so global volume reversal cannot mask the difference. |
| An exact-paired closed non-orientable facet neighborhood, with a prior component that already caused reversals | Missing contradiction rollback, wrong rollback order or incorrectly clearing its reversal list between components. Native import admission must be established; do not promise this input slices successfully. |
| Signed-zero endpoint encodings and permuted facet order | Incorrect exact matching, hash-order dependence or changed traversal/reduction order. |
| Translated/scaled closed inputs with nonzero volume and float-sensitive arithmetic | Wrong reference point, area cast/reduction order or double-precision substitution. |
| Existing open, degenerate and high-valence controls; existing 3MF loading | Accidental scope expansion, facet deletion, new rejection or repair applied to unrelated import paths. |

## Deferred behavior

- Nearby-edge matching, tolerance iterations, unconnected-facet removal,
  repeated-vertex/other degeneracy repair and their connectivity refresh.
- Hole filling: the import code places it under `#if 0`; it is not active behavior
  to port even in a later completeness claim without a different source owner.
- General non-manifold repair, self-intersection removal, remeshing, CAD/painting
  handling, arbitrary mesh-format repair and public repair-statistics APIs.
- Multipart conversion/placement, model identity allocation, configuration and
  profile defaults, GUI behavior, browser interaction and whole-output parity.

The acceptance boundary is the imported facet geometry and its unchanged
transfer to existing slicing, not complete STL repair, whole-G-code parity or
proof of browser/Windows/macOS execution from cross-compilation alone.
