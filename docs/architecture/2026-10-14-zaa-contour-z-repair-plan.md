# ZAA ContourZ repair PLAN

Status: PLAN-REVIEW-REQUEST; RED6d2b6f37a APPROVED by coordinator.
RED candidate:6d2b6f37a0b0115f0880599bacef5247fc351020.
Base main:de1211be5067d0eb6f709602a1299a0854c89145.
SPEC06a8bd6fe and RED6d2b6f37a approved; no production changes.
Must not implement before coordinator approves an exact committed PLAN.

RED6d2b6f37a is independently committed and reported before this PLAN:
officialCPU13 and seededCPU14 each have disabled strict PASS and enabled
expected RED (nextest100). Enabled Native is complete with418 positive XYZ
extrusions; API rejects zaa_enabled, CLI exits1 with the same named error.
Final fresh roots: /tmp/worker2-zaa-red-official.qqyF7Q and
/tmp/worker2-zaa-red-seeded.i7z8wL. Inputs and disabled raw API/CLI streams
are unchanged by test-helper lifetime/lint corrections. Test strict clippy
and fmt exit0. External immutable-hash proof and original streams are indexed
by /tmp/worker2-zaa-red-evidence.json; no Native products are committed.

## Source ownership and CPU determinism

SLA/IndexedMesh.cpp:17-43 uses AABBTreeIndirect::Tree3f, fixed triangle-ray
epsilon and source indexed vertices. :139-171 provides float hit distance
and normalized face normal. Port AABBTreeIndirect.hpp's indexed-triangle
build and nearest-hit traversal, not an independently designed BVH heuristic.
Construction order, median partition, axis selection, tie behavior, child
visit order and triangle order must follow this serial source implementation.
Never use available_parallelism, CPU affinity or worker count to choose any
of these. The initial port remains serial within each object; no parallel
floating-point reduction. Layer/object-level concurrency is deferred, not
required for the first port. Any later concurrency must not change ordering,
selected face or geometric results.

## Modules and data changes requiring approval

1. project_slice/contour_z.rs is the stage owner. Small files beneath
   contour_z/mesh.rs, tree.rs, ray.rs, path.rs and entities.rs split mesh
   transformation/ground frame, Tree3f construction, nearest-hit, sampled
   contouring and collection traversal. Every file<=400LOC; no native I/O.
   Cache one immutable source-shaped mesh/tree per resolved print object.
2. slice_source inserts the stage after fill/ironing and before XY-only
   ordering consumers, by mutating PreparedPostFillEntities in place.
   Per-region ranges remain authoritative; no first-region shortcuts.
   Inspect current region access and object mesh frame before wiring.
3. ExtrusionPath gains source z_contoured=false metadata; Polyline3 already
   retains XYZ. FillExtrusionPath's XY Polyline needs an XYZ carrier matching
   the same upstream ExtrusionPath concept. Proposed change: Polyline3 plus
   z_contoured rather than parallel XY and XYZ copies. Ordinary fill producers
   mechanically create Point3(x,y,0); preserve existing XY sequences and
   fitting metadata. Move the current fill-level fitting Vec into
   Polyline3.fitting and remove that duplicate field; adjust its consumers
   mechanically. Compiler inventories all constructors/consumers.
4. Contour path reads current layer/region minZ and angle, samples0.1mm,
   matches ContourZ.cpp:33-159 exactly, including no-hit/clamps/wall exclusion
   and collinear simplification. Preserve every path unchanged if no point
   is contoured; first layer unchanged. Collections only eligible roles.
5. path_simplification must early-return for z_contoured in both ordinary
   and arc-fitting branches, exactly ExtrusionEntity.cpp:33-47. No new 3D
   simplify algorithm: ContourZ.cpp already simplifies its sampled points.
   seam_placement and loop clipping retain/split Z with source path rules.
   XY projection for geometric queries is separate from output vertices.
   No 3D-to-XY-to-zero roundtrip or flattening of the sampled contour.
6. gcode_emit/motion/materialized.rs, features::PathProperties and path
   dispatch receive the authoritative per-point contour flag/Z. Add a
   contour emission module under motion/path; start_travel handles source
   first-Z restoration/ordinary-path nominal reset. Port GCode.cpp:6374-6400,
   7012-7030 and processed-point branch7220-7237, with source 3D lengths,
   height-based E ratio (except ironing) and XY origin offsets applied once.
   Ordinary noncontoured motion must remain byte-identical.
7. capabilities.rs removes only the zaa_enabled rejection and compiler-
   proven orphan predicate helpers after paths can actually render. Retire
   the now-obsolete capability rejection unit assertion only with separate
   exact-diff/upstream-cited coordinator permission. Existing behavior tests
   and all RED assertions remain intact.

## Risks and tests before production

Public RED must prove disabled whole-output PASS, enabled active Native XYZ,
and Ares API/CLI capability rejection on the same bytes. If disabled differs,
STOP: do not mask a different Fill/perimeter gap to force this RED through.
After approved PLAN: preserve source tests; first turn the admitted public
RED strictly GREEN. Retain input/footer/roles/SHA and enabled E/Z streams.
Further tests written before any respective extension cover wall-minimum
clamps, no-hit, transformed object frame, ironing E exception, first-layer
exclusion, nominal-Z restoration and contour-point propagation through seam
and simplify. These are public Native E2E at the agreed slice seams; numeric
kernel isolated tests require their own written failure inventory first.
Do not add broad guard deletions, new guards, fallback or weak comparators.
If exact source-cited carrier edits require scope changes, stop for PLAN
amendment. All local runs CPU13/14, jobs2/threads2 and fresh Native roots;
workspace/full only on coordinator GO. Strict/fmt/wasm exact hash before
code REVIEW; coordinator alone pushes or merges.

## Exact carrier/consumer change inventory

| Rust destination | Source-cited change |
|---|---|
| perimeters/classic/materialize/types.rs::ExtrusionPath | Add z_contoured:false to match ExtrusionEntity.hpp:166; retain existing Point3/Polyline3 fields and flags. Copy/reverse/clone/split preserve flag and Z (ExtrusionEntity.cpp:289). |
| fill_entities/types.rs::FillExtrusionPath | Replace XY Polyline with existing Polyline3 and source z_contoured flag. Retain role/width/height/mm3_per_mm; migrate the separate fitting Vec into Polyline3.fitting and remove the old field. Ordinary constructors embed Z=0. Avoid parallel contour/fitting sidecars. |
| fill_entities producers/consumers; extrusion_islands/island_print_order | Mechanical XY projection only for spatial lookup/ordering. Stored/emitted vertices remain XYZ; retain region associations and collection order. Constructor adaptation is compiler-enumerated, not unrelated Fill algorithm cleanup. |
| path_simplification.rs | z_contoured early-return in simplify_path3 and fill paths, matching ExtrusionEntity.cpp:33-47. Ordinary paths unchanged. |
| seam_placement/split.rs and motion/loop_paths clip/split consumers | Preserve source 3D split, reverse and clipping semantics; audit Polyline3/ExtrusionLoop before patching. No invented Z interpolation policy. |
| gcode_emit/motion/materialized.rs; motion/features.rs::PathProperties | Propagate source flag and authoritative XYZ polyline through perimeter/fill emission; keep role, speed, acceleration, flow and slope metadata. |
| gcode_emit/motion/path.rs and new path/contour.rs | Dispatch source contoured XYZ branch instead of flattening iterator. Ordinary entry points still produce identical XY output; include GCode.cpp processed-point contour branch. |
| motion/path/start_travel and motion/path/contour.rs | First-point Z set/reset, nominal-Z restoration and XYZ extrusion with source E height ratio, including ironing exception. Existing travel/retraction/formatter/writer APIs stay authoritative. |
| capabilities.rs | Delete only live zaa guard and resulting compiler-proven orphan helpers; preserve unrelated layer-range/spiral rules. |

The carrier migration may require mechanical changes across fill construction
files and existing test literals to supply unchanged Z=0/z_contoured=false.
It does not authorize behavioral assertion edits. Submit any test-contract
retirement exact diff separately before applying it. Files approaching400LOC
are split by the named source concern; do not add lint suppression.

Included and deferred behavior remain the approved SPEC's boundaries.
Generic mesh/path algorithms replace the capability scaffold; there is no
wedge shortcut, shape whitelist, new feature guard or silent omitted role.
First validation is the finite ordinary-layer witness; additional interactions
are deferred as coverage cases, not silently replaced with different behavior.

## Implementation sequence after approval

1. Port deterministic Tree3f/IndexedMesh query and contour entity stage;
   carry source path metadata. Test seams/failure inventory exist before code.
2. Propagate XYZ through carriers, simplify/seam/clip and emission in one
   source-bounded repair; no intermediate guard-only GREEN claim.
3. Only then remove the capability rejection, with separately approved
   retirement of its obsolete private-test contract. Keep public RED intact.
4. Verify the exact GREEN hash in both Native modes, preserve disabled bytes,
   then strict/fmt/wasm and code REVIEW. No production before PLAN approval.

## Concrete ray-query layout and source traversal

Port AABBTreeIndirect.hpp::Tree<3,float> / Tree3f (:217), build_recursive
(:121-155), partition_input (:157-212), indexed triangle construction
(:672-717) and first-hit query (:396-440,723-746). mesh.rs owns transformed
f32 indexed vertices, original-order triangle indices, immutable tree and
ground offset. Tree nodes carry source bbox and leaf triangle index (or
inner-node marker), in the source implicit binary layout; source child
indices are node*2+1 / node*2+2. No task/worker metadata enters this layout.

Build from source face order, compute source bbox/centroid, select longest
bbox axis with source axis ties, then source median-of-three QuickSelect.
Do not substitute a stable sort, an independently designed tie rule, or
CPU-dependent bucket sizes. Query left first, update min_t only for strict
less-than hit distance, then right with the updated bound. Equal float hit
distance does not replace the left result. Compute ray intersection in
source f64 precision, then retain the source f32 hit-distance carrier and
normal conversion. Match source positive-t and epsilon semantics; no-hit
is the source no-hit outcome, not an invented fallback.

All tree construction, partition and query recursion remains serial and
fixed-order. No available_parallelism, CPU-count branching, dynamic task
partition or floating-point parallel reduction. A later parallel layer
schedule could only write independent slots without changing any query;
that optimization is deferred. This immutable indexed structure avoids
per-sample all-triangle scans while remaining WASM/Windows/macOS/Linux pure
Rust with no Native I/O or platform-dependent topology.

## Guard deletion and compiler-proven orphan inventory

capabilities.rs:38-43 deletes the zaa-requested/inactive-box rejection only
after admitted paths contour and emit. The resulting local candidate-orphan
chain is zaa_requested (:47), zaa_is_provably_inactive (:65),
is_axis_aligned_box (:93), extrema (:125), subtract (:132), cross (:136),
and close (:144). Point3d, ProjectVolumeType and Transform3d imports then
become unused. Confirm each with compiler/reference searches before deletion;
no pre-existing unrelated dead code is included. Expected file150 ->40-50LOC.

Keep validate's layer-range and spiral clauses, ProjectObject,
ResolvedProjectObject, SliceError and unsupported: the latter still serves
two layer_config_ranges rejections (:26/:36). No guard replacement or shape
whitelist; real contouring replaces the current capability scaffold.

Private tests/capabilities.rs::zaa_requires_a_provably_inactive_axis_aligned_box
and its now-unreferenced test helpers are a separate retirement request,
not implicitly approved by this PLAN. Exact diff/source citations must be
approved before changing that test. RED6d2b6f37a assertions remain intact.

## Estimated LOC and file split budget

These are implementation estimates, not code already written. LOC counts
physical source lines; all files stay<=400 including comments and tests.
If the source port requires a material expansion, STOP for amended PLAN.

| New source file under project_slice/ | Expected LOC / source responsibility |
|---|---|
| contour_z.rs | 60-100: stage wiring / PrintObject::contour_z |
| contour_z/mesh.rs | 90-140: transformed raw mesh, ground frame, indexed normal/query owner |
| contour_z/tree.rs | 140-200: Tree3f nodes and recursive construction |
| contour_z/tree/partition.rs | 80-140: source QuickSelect and axis comparison |
| contour_z/ray.rs | 200-300: bbox/triangle intersection and first-hit traversal |
| contour_z/path.rs | 170-250: ContourZ sampled path, bounds and collinear simplification |
| contour_z/entities.rs | 130-200: eligible region/entity traversal, source loop/multipath/collection dispatch |
| gcode_emit/motion/path/contour.rs | 180-280: source XYZ start/end and per-point E emission |

Estimated new implementation:1050-1610LOC across these source-owned files.
Additional net carrier/wiring changes: roughly200-400LOC; guard/orphan
removal roughly100-110LOC. Total net source growth estimate1140-1910LOC,
with mechanical edits spread across existing consumers, not a single file.

| Existing source file/group | Current LOC / expected boundary |
|---|---|
| perimeters/classic/materialize/types.rs | 97 ->100-120; source flag and carrier conversions |
| fill_entities/types.rs | 83 ->85-115; Polyline3/flag, remove separate fitting field |
| path_simplification.rs | 223 ->225-260; source contoured early-return plus mechanical 3D fill projection |
| motion/features.rs | 328 ->335-355; contour metadata, no extra kinematics policy |
| motion/materialized.rs | 69 ->80-110; preserve XYZ and flag through dispatch |
| motion/path.rs | 352 ->365-390; small dispatch only; contour emitter stays in separate file |
| capabilities.rs | 150 ->40-50; specified guard/orphan removal only |
| fill producers, ordering, seam/clip/start-travel | compiler-enumerated mechanical adaptations; split any file before exceeding400 |

Static constructor search finds FillExtrusionPath literals in28 files,
ExtrusionPath literals in53 files, and PathProperties literals in11 files
(overlapping counts include tests). This is an honest migration footprint,
not permission for algorithm refactors. Preserve all ordinary XY sequences,
roles, flow/height/width, reversibility, fitting semantics and collection
order. Constructor test literals may need the unchanged default flag/Z to
compile; no behavioral assertion deletion/weakening is authorized.
