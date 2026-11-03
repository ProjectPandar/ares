# SPEC: localize odd-order mesh connectivity to layer-geometry divergence

Status: docs-only SPEC-REVIEW-REQUEST after first-layer SPEC7975ff214 STOP.
Base main104d8497097dc7239ad63f7a69f3b418b699b3e8; approved mesh-edge
pairing is merged. No production or test assertion change accompanies this
proposal. First-layer predicate attribution is recorded separately in
first-layer-collection-state-preflight-stop.md (commit62f3f5527).

## Problem and proven limit

Odd-rotated has Native first-layer perimeter/fill collections0/0 in BOTH
modes, while Ares already has genuine perimeter/fill collections before
island wrapping. Native's first8 layers are empty and last2 populated;
Ares prints all10. Both implementations apply their existing export check
correctly to their own state. Adding a GCode guard cannot close this gap.

Input SHA25693a493cec3f419398d2cfc04401e0c5f53c373ffe38d2e526c885e876f6b5bc3;
input/generator are already committed under tests/parity/mesh-edge-pairing.
Native result-100/Unix156, stderr Errors/no G-code; Ares raw output SHA
99521ddc83fa146da43b50d88271afb586afc95f2c046f875089130fb6afc6b4,
API/CLI identical, including on main104d full6. The latter is a diagnostic
gap, never a strict parity PASS. Do not presume sorting is the cause from
face-order sensitivity alone.

## Owning upstream slice and observation seams

1. TriangleMesh.cpp::EdgeToFace/create_edge_map:520-563,
   its_face_edge_ids_impl:565-613 and wrappers:616-629 own indexed edge-use
   ordering and opposite-first/same-direction pairing. operator< compares
   endpoints only; std::sort equal-key ordering is an explicit risk.
2. TriangleMeshSlicer.cpp::slice_facet:156-321 and
   slice_facet_at_zs/slice_make_lines:476-562 own oriented intersections,
   integer XY points and endpoint identities. slice_mesh:1865-1902 obtains
   face_edge_ids, transforms vertices/planes and passes lines to make_loops.
3. TriangleMeshSlicer.cpp::chain_lines_by_triangle_connectivity:1060-1176,
   chain_open_polylines_exact:1179-1276, close_gaps:1278-1381 and
   make_loops:1383-1482 own polygon/open-polyline construction. make_loops
   runs connectivity, exact same/reversed passes (:1415-1432), then2mm gap
   same/reversed passes (:1450-1461). The disabled debug/alternative code is
   excluded; do not introduce an invented closure-repair policy.
4. Layer.hpp:101/207/295 and GCode.cpp:1771-1778 are downstream observation
   boundaries. They remain unchanged. If loops already match but collection
   generation differs later, STOP and identify the actual downstream owner;
   this SPEC does not authorize guessing a PerimeterGenerator patch.

## Rust destination and existing scaffold

Matching inspection targets are mesh_slicer/topology.rs
(sorted_edge_uses/index_mesh_edges); mesh_slicer.rs and intersection.rs
(plane dispatch and IntersectionLine); mesh_slicer/chaining.rs, exact.rs,
gaps.rs (connectivity, exact and gap passes); and project_slice/
raw_intersections.rs, chained_intersections.rs, looped_intersections.rs as
transport shells. These represent the cited upstream mesh/line/loop data,
not an independent Ares slicing pipeline.

This request authorizes a public RED plus read-only localization ONLY.
It does not yet name a production fix because the first divergent seam is
unproven. After localization, propose a narrow source-cited GREEN addendum
naming the exact owner/function and affected Rust files. Until that addendum
is approved, sorting, pairing, interpolation, chaining and transport are
all unchanged. Keep the already merged opposite-first pairing semantics.

## Hypotheses and discriminating evidence

- Import/transform hypothesis: if the actual indexed vertices, oriented
  triangles, effective transform or sample planes differ before edge IDs,
  first report that mismatch and stop for its owning import/model SPEC.
  Identical zipped inputs and facet multiset do not alone prove identical
  in-memory transformed meshes. Public load/selection contracts are deferred.
- Edge-order/pairing hypothesis: identical indexed input but different
  per-face edge IDs would locate an earlier divergence than slicing/loops.
  Read both actual outputs, including the high-valence edge's paired uses.
  Existing Rust face/local-edge tie order is not assumed Native-equivalent.
- Intersection hypothesis: matching edge IDs but differing ordered lines,
  endpoint references or coordinates at the same plane indicates the facet
  or line-dispatch boundary. Do not compare only unordered point sets.
- Chaining hypothesis: identical ordered raw lines but differing connected
  polygons/open polylines locates connectivity/exact/gap semantics. Observe
  each return before any cleanup so later repair cannot hide the first gap.

Stop at the earliest proven divergent seam. Read all owning regions/volumes
and retain source identities; no regions[0], normalization of ordering,
empty-path substitution or fixture-specific filtering.

## Native admission and public RED plan after approval

1. Use the exact odd-rotated input as a separately committed rejection RED:
   both Native modes must produce-100/Errors/no G-code; current public Ares
   API/CLI succeeds. Assert the desired public empty-first-layer typed error
   and no output, with its existing message/CLI exit1. No attempt to make
   Ares CLI numerically imitate Native's156/-100 adapter code is included.
2. Retain odd-reverse typed-error/no-output control, odd-original strict
   Native0 control and all13 admitted mesh-edge strict tests. Do not change
   the existing diagnostic collector to count this rejection as output parity.
   Retire an existing assertion only after a separate exact request.
3. In CPU12 read-only preflight, capture the actual mesh/planes, face-edge
   IDs, raw ordered lines, connectivity polygons/open polylines, exact-pass
   and gap-pass results for these same inputs. Work forward only until the
   earliest divergence; no independent C++ replay or code probes.
4. Compare actual producers. If std::sort is implicated, prove the relevant
   implementation/source identity rather than relying on ELF compiler
   comments or a convenient local header. A generic new sorting module or
   assumed stable tie policy is not authorized by this SPEC; unresolved
   implementation identity requires another STOP/addendum.
5. Report localized source witness and request the small GREEN addendum.
   Subsequent approved GREEN must naturally yield the correct first-layer
   state, future_gap=false, both-mode13/13 admitted raw-output controls and
   strict/fmt/wasm32, followed by review and allocated full.

## Included and deferred behavior

Included: exact current odd-order witness, first physical slice_z0.1 /
print_z0.2 plus remaining layer states as needed to distinguish semantics,
existing Native admission/error boundary, public API/CLI observation and
source-data identity across the cited mesh/line/loop seams.
Deferred: any production algorithm, general non-manifold repair, welding,
deduplication, new guard, change to2mm gap policy, mask/slab/cut-mesh variants,
import/load/selection API changes, CLI exit-code redesign, broader stochastic
oracle behavior or edits to unrelated print features. No feature flags or
legacy fallbacks. Tier1 platforms remain required for any later Rust rewrite.

## Durable evidence and execution rules

The preflight STOP document commits first-layer counts, outcomes, input and
output hashes, explains each void run, and references a144388-byte archive
outside Git at ${EVIDENCE_ROOT}/worker3/
first-layer-state-main104d-9y7stqky.tar.gz, SHA256
05d72fbd8d0f9da1fc74a2804056308316e1d0cf4a479eace3c202c8c5bbfc3b.
Native output stays outside Git; document new findings so /tmp cleanup cannot
remove the only record. Actual debugger exit is not a parity result.

All live runs: CPU12 outer taskset, jobs1/threads2 directed, fresh Native
cache/artifact roots, START/END inbox. No run during exclusive freeze;
terminate the whole driver/debugger/Native tree when instructed. No full or
workspace without allocation, no push, no implementation before addendum
approval. The submitted SPEC is docs-only.
