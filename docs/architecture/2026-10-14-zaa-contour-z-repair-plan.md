# ZAA ContourZ repair PLAN

Status: PLAN-REVIEW-REQUEST, conditional on approved public RED.
RED candidate:6d2b6f37a0b0115f0880599bacef5247fc351020.
Base main:de1211be5067d0eb6f709602a1299a0854c89145.
SPEC06a8bd6fe approved; no production changes.
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
