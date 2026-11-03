# Negative-volume occlusion for seam visibility

## Status and upstream rewrite boundary

SPEC only; implementation requires separate public RED review and GREEN approval.
Independent base `0266b34949a6f9503002a3420e1c526cb745462b`.
OrcaSlicer2.4.2 source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This is a source-cited libslic3r negative-occlusion rewrite slice, not a new Ares
visibility/Boolean pipeline. No code from abandoned cursor candidate dd8a8ca02
or its initial-cursor SPEC489c86f17/RED66c553811 is inherited.

Owning upstream files/functions/data:
- `GCode/SeamPlacer.cpp:628-704::compute_global_occlusion`: collect MODEL_PART
  and NEGATIVE_VOLUME into separate indexed sets, transform each with winding
  fix, decimate both separately, store negative_volumes_start_index after positive
  decimation, append negative set, transform combined object, sample and build
  sample KD tree/radius and triangle AABB tree, pass split index to visibility.
- `TriangleSetSampling.cpp:9-70::sample_its_uniform_parallel` and
  `TriangleSetSamples::triangle_indices`: retain sampled triangle provenance
  alongside positions/normals, using the same area/CDF/RNG/barycentric order.
- `GCode/SeamPlacer.cpp:128-233::raycast_visibility`: for negative-bearing meshes,
  identify negative samples by triangle ID, invert ray direction and origin side,
  query all intersections, traverse hits in reverse and apply signed positive/
  negative face-normal counter; decrement visibility under source counter rule.
- `AABBTreeIndirect.hpp:755-787::intersect_ray_all_hits` and its detail
  RayIntersectorHits traversal: BVH traversal, source ray/triangle intersection,
  float hit parameters and ascending-t sorting, not first-hit substitution.
- Existing visibility interpolation and nearest consumer remain consumers:
  `SeamPlacer.cpp:783-795,930-941,1500-1563`. Do not compensate in those scores.

## Exact Rust destination and temporary scaffold

Under `crates/ares-core/src/project_slice/seam_placement/`:
- `mesh.rs::TriangleMesh::from_object` currently filters ModelPart only, with
  negative volumes explicitly deferred. Port positive/negative bucket collection,
  separate decimation, merge and split metadata at this upstream boundary.
- `short_edge_collapse/mod.rs::IndexedMesh/short_edge_collapse`: reuse existing
  source-owned decimator separately per bucket before merge; no new decimator.
- `sampling.rs::TriangleSamples/sample_uniform` currently lacks triangle IDs.
  Port source sample provenance without reseeding/reordering existing samples.
- `visibility.rs::GlobalVisibility::from_mesh/sample_visibility` currently has
  positive first-hit behavior only. Port the negative branch and preserve the
  source no-negative branch; the latter is upstream behavior, not legacy fallback.
- `spatial/bvh.rs::TriangleBvh`: add source-equivalent all-hits traversal/query;
  split into a source-aligned sibling file if additions would reach400LOC.

The positive-only scaffold is incomplete for the cited operation. Replace this
omission, not emitted G-code, negative slicing geometry, cursor state or seam
candidate coefficients. No public slicing API, UI, filesystem or terminal code
belongs in this core rewrite; browserWASM/Windows/macOS/Linux stay Tier1.

## Nonnegotiable semantics and performance

1. Preserve the negative split after separate source decimation; sampled face
   identity, not guessed normals/coordinates, determines negative membership.
2. Include negative surfaces in the source sampling distribution. Do not pre-
   Boolean the occlusion mesh, skip negatives, treat them as positives or design
   a replacement CSG policy. Preserve the upstream volume-order TODO semantics.
3. Implement all-hits through the existing BVH: build once per occlusion mesh,
   prune nonintersecting nodes, test only reached leaves, retain hit IDs and t.
   NO linear scan over every triangle per ray and NO linear fallback; do not
   rebuild a tree per query. Sorting collected hits by t matches source behavior.
4. Reuse hit storage across rays at the source work-unit/worker boundary, as the
   C++ blocked-range loop does. Avoid allocating a fresh hit vector per ray or
   copying entire meshes per query. Preserve source epsilon and arithmetic;
   do not add a face-ID tie-break or numeric tolerance as a parity workaround.
5. Preserve existing non-negative branch and weighting/candidate consumers. No
   new Unsupported guard, mask, feature flag, admission-only success or fallback.
6. Port the first producer completely. A later whole-output residual means STOP,
   not adjustment to nearest score, region slicing, travel or processor time.

## Included and deferred upstream behavior

Included: ordinary and negative mesh buckets, existing source transforms and
winding rules, separate decimation and split index, combined surface samples
with triangle provenance, negative origin/direction rules, BVH all-hits query,
reverse sign-counter visibility and existing interpolation into candidate data.
This is the complete missing negative branch, not merely appending its triangles
while retaining first-hit visibility.

Deferred/frozen: initial generator cursor, routing/lift/retraction, W4 print_order,
actual negative diff_ex/region graph, perimeter topology/order, seam candidate
positions/scoring/kdtree selection, painting, fan/skirt/brim types, processor/time,
and broader multi-tool behavior. The separately frozen20e41 volume-affine precision
slice is not cherry-picked or widened here. The accepted integral-transform input
keeps this boundary separable; integration remains coordinator-owned.

## Qualified runtime witness and corrected attribution

Accepted existing GUI normal-part to negative-part pair from
`validation/modifier-wall-loops-classic.3mf`, common X1 Carbon0.4/Classic profile,
nearest seam, no arc/skirt/brim/EFC/scarf/ironing/painting. Freeze original Native
input-only exports; no generated Native G-code or expected output constants.
The GUI type change also derives six source_* provenance omissions through
bbs_3mf.cpp2214-2220/7862-7869, explicitly audited and coordinator-accepted.

Whole-output target first difference is NON-M73 byte78/line3: Native modeltime
3m45s/Core3m44s,total11m1s; first unequal motion109/raw1337 NativeX127.25 vs
CoreX123.75,Y121.75. PrintZ0.2, priorZ0.6 lift. These are discovery observations,
NOT public-test expected coordinates/time values.

Live candidate audit: cursors/preferred(-128,-128), definedfalse, ordered loop
and five candidate3D positions/range[0,5)/angles/overhang/embedded agree after the
abandoned cursor port. Output stayed byte-identical to RED: it was not this
witness's sufficient cause and has no independent output RED; do not inherit it.
First different nearest input is visibility/base penalty. Example candidate0:
Native visibility live0.6687729; Rust combinedbase live1.5264784; commonangleterm
host0.5264784 gives Nativebase host1.1952513. Live chosen candidate points uniquely
identify Nativeindex3/Coreindex2; raw index variables optimized out and are not
claimed captured. Float32 host replay predicts the same selection; its penalty
arithmetic is not claimed as captured Native expf bit identity.

Native live raycast input has16vertices/24triangles, negative split12. Rust live
positive decimator input has8vertices/12triangles,target16000. Stages are explicit:
source adds no triangles during decimation and has no later negative merge, so
missing-negative semantics are source reasoning, not a fabricated finalBVH probe.
Native gathers negative surfaces/all-hits; Rust filters them out and lacks that
branch. Normal/direct-offset controls do not exercise this missing operation;
earlier six direct case-mode comparisons PASS, not universal cursor parity.

## Public RED and subsequent approval gates

After this SPEC, add tests/input+SHA only using the approved existing
`runner::native_stream_case::check -> native_api_cli::NativeCase` seam. Dynamic
Native2 FIRST, actual validated byte API and real CLI, unchanged whole generator-
only ordered comparison and actual raw equality. Normal control must completePASS;
negative target remains authenticRED. No private/new unit tests, Native expected
visibility/coordinates/time constants, inverted assertions, skips or helper change.

Keep all M73, time, comments, motion, metadata/footer comparisons. RED canonical
API panic precedes final raw assertion/adapter update; independently audit actual
retained API/CLI bytes without claiming those assertions executed. Freeze the
same accepted input bytes and document export-derived bookkeeping, not a false
physical one-field-only archive claim.

Run official/seeded targeted nextest sequentially on CPU15/jobs1/threads1,
independent initially empty caches, retain complete Native/API/CLI artifacts.
Run rustfmt and strict workspace/all-target clippy; changed text files stay<400.
Commit docs-only SPEC then tests/input-only RED and send RR. GREEN, adjacency,
wasm32 compilation/performance evidence and exact-tip FULL require separate
coordinator approval; no browser-runtime or FULL credit at this phase.
Future performance review must verify BVH pruning/all-hits storage reuse rather
than accepting a semantically passing but per-ray linear mesh scan.

## Evidence and archive boundaries

Accepted read-only audit:
`${EVIDENCE_ROOT}/w6-candidate-nearest-audit-ioic_az3/`
`NEAREST-INPUT-AUDIT-REVIEW-REQUEST.md`, `verification.json`, `verify-audit.py`,
complete valid/invalid probes and host/live distinction. Fourteen normal-exit
output-preserving observers:8valid/6excluded; no inferior calls/explicit writes.
Initial cursor chain remains audit-only, not a dependency of this rewrite.

New independent SPEC/RED evidence:
`${EVIDENCE_ROOT}/w6-negative-occlusion-spec-red-j92rt3s0/`.
No claim that an unimplemented negative-occlusion port fixes every later residual.
