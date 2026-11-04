# GREEN addendum: create_edge_map endpoint-only reference sort

Follows the localization SPEC and the public odd-order admission RED.

## Bounded localization result and RED

Localization stopped at the first proven data divergence;
no downstream raw-line/loop investigation or implementation was started.

RED official/seeded each:2 controls PASS,1 expected FAIL. Odd-rotated Native
result-100/Unix156, stderr Errors, no G-code; Ares API/CLI emits SHA
99521ddc83fa146da43b50d88271afb586afc95f2c046f875089130fb6afc6b4.
Odd-reverse typed-error/no-output and odd-original strict Native0 controls
PASS. strict/fmt/wasm32 all exit0 on the RED head. These are new
admission assertions; existing diagnostic/13 admitted assertions stay intact.

Odd-rotated input SHA256:
93a493cec3f419398d2cfc04401e0c5f53c373ffe38d2e526c885e876f6b5bc3.
Input fixtures and deterministic generator are committed under
 tests/parity/mesh-edge-pairing; no Native output is committed.

## Earliest proven source seam

TriangleMesh.cpp::create_edge_map:535-560 builds three EdgeToFace uses per
face, then std::sort at558. operator<:531 compares vertex_low/high ONLY.
Rust mesh_slicer/topology.rs::sorted_edge_uses currently adds face/local_edge
to the key and uses Rust sort_unstable_by_key. Its reference tie operation
is therefore different even though opposite-first pairing:565-613 was ported.

Read-only runtime snapshots establish, for odd-rotated:

- Both Native producers and Ares have identical13 oriented triangles in the
  SAME indexed order,9 vertices in equivalent effective slicing coordinates
  and10 identical float32 planes. Native mesh coordinates are unscaled with
  Z-1 and params translation Z+1; transform_mesh_vertices_for_slicing
  (TriangleMeshSlicer.cpp:1840-1861) scales XY by1e6. Applying the captured
  matrix yields exactly the captured Ares float32 vertices. No import or
  physical-plane difference is hidden by comparing only unordered geometry.
- Actual Native pre-sort39 EdgeToFace uses match each other and the Rust
  construction derived from its captured triangle order and the cited loop.
  The latter is source-derived, not a claim of a Rust pre-sort breakpoint.
- Actual Native POST-sort39 uses match across official/seeded. Ares post-sort
  uses were read at topology.rs:51, before partner consumption. Both Native
  reorder the high-valence group while Rust keeps its total secondary key.

Edge(3,7), current face numbering and signed1-based local edge:

| Stage | Ordered (face,signed edge) uses |
|---|---|
| Native pre-sort / Rust construction | (0,-3),(4,+1),(5,+1) |
| Native official+seeded post-sort | (5,+1),(4,+1),(0,-3) |
| Ares post-sort | (0,-3),(4,+1),(5,+1) |

Face0 is cube triangle(3,6,7); face4 is cube triangle(3,7,5);
face5 is the extra triangle(3,7,8). Both Native pair face5/local0 with
face0/local2 as edgeID12 and leave face4/local0 singleton13. Ares pairs
face0/local2 with face4/local0 as12 and leaves face5/local0 singleton13.
This is a different connectivity graph, not harmless ID renumbering. The
remaining per-face edge IDs agree. Odd-original positive control preserves
high-edge order(7,-3),(11,+1),(12,+1) in both Native and Ares; full pair ID
matrices match. All observations used existing binaries and read-only GDB.

This locates the first divergence at create_edge_map sorting. The previously
observed genuine collection difference precedes the export predicate:
Native first8 layers empty/last2 populated; Ares all10 populated. No claim
of a missing GCode validation or Native nondeterminism is made.

## Concrete minimal production change

Only modify crates/ares-core/src/mesh_slicer/topology.rs:

1. Reuse the EXISTING private geometry::fixed_gcc_sort_by helper.
2. Replace the total-key Rust sort with endpoint-only strict comparison
   `left.key() < right.key()`, matching TriangleMesh.cpp:531.
3. Retain EdgeUse construction, orientation, face/local metadata, consumed
   partner logic, distinct pair/singleton IDs and range checks unchanged.

No new generic sorter, stdlib module, fallback, platform cfg or option is
introduced. The existing helper in geometry/clipper/ordering/gcc.rs remains
untouched. It implements the fixed GNU introsort/final-insertion operation
already used elsewhere in main: threshold16, depth2*floor(log2(n)), median
pivot, right recursion/left iteration, heap fallback and insertion passes.
Use that deterministic platform-neutral operation, not host std::sort or a
stable secondary tie-break. Do not add guards or force an error downstream.

## Supplier evidence and its explicit limit

Both actual EdgeToFace call sites invoke GNU-style instantiated
__introsort_loop followed by __final_insertion_sort (full symbols/disassembly
retained). Their complete39-use permutations match for both examined inputs.
The source comparator is endpoint-only; this is more than ELF compiler-comment
attribution. It does NOT pin an exact per-TU GCC/header release or establish
that every Native build/standard library has identical tie order.

This addendum explicitly selects the existing fixed GNU helper as
compatibility supplier for the CURRENT pinned canonical producers. The
implementation must demonstrate complete sorted-use and per-face-ID agreement
with BOTH actual producers for the saved witnesses. If the existing helper
fails, STOP; do not edit it, add a new sorter, sample permutations or silently
switch algorithms. Further supplier/source changes need another SPEC.

## Required GREEN evidence and deferred scope

Keep production in its own commit, then verify:

- Both-mode public rejection gates become GREEN: odd-rotated produces the
  existing InvalidInput empty-first-layer error and CLI1/no G-code naturally.
  Keep odd-reverse and odd-original controls; all13 admitted mesh tests must
  remain strict whole-output generator-header-only parity with raw API/CLI
  equality. Neither rejected input counts as output parity PASS.
- Read-only snapshots confirm full39-use order and per-face IDs now match
  Native, not just the high-edge pair. Confirm genuine first-layer source
  collections are empty, not a rejected nonempty geometry state. The existing
  diagnostic collector must report future_gap=false for both rejected inputs.
- If sort/pair snapshots match but admission or collection state still differs,
  STOP with that residual witness; do not expand into chaining here.
- Retained topology/intersection/chaining controls and strict/fmt/wasm32 pass
  on the candidate; Rust files<=400LOC. No assertion weakening or retirement
  is proposed.

Deferred: changes to pairing policy, facet interpolation, connectivity/exact/
gap passes,2mm closing threshold, import/load/selection, perimeter or infill
algorithms, error mapping and GCode first-layer predicate. Do not retag layers,
drop actual extrusions or emit a fixture-specific rejection.

Native error comparison: GCode.cpp:1771-1778 observes no object/support
extrusions on its first pair and throws SlicingError with source text
"One object has an empty first layer and can't be printed. Please Cut the
bottom or enable supports." GCode.cpp:1848-1866 aggregates that error;
Exception.hpp:44 labels the aggregate Errors, which is the ACTUAL stderr.
OrcaSlicer.cpp:6286-6291 records-100. The proposed geometry port lets the
already implemented Rust schedule.rs:219-246 check fire; no new error guard.

## Probe corrections

Rust sort-helper entry inspection attempts encountered breakpoint expression
errors or an inlined helper with no standalone address. Ares legs with those
inspection failures are not used as sorted/ID witnesses. Native legs in the
same roots remain separately valid. The final Rust source-line breakpoint
reads the live `uses` vector; no noinline/build-flag/source-probe change.
Unsuccessful probe attempts are classified, not hidden by retries.
