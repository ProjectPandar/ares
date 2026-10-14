# SupportCubic Slice B: object-octree input localization

Status: SPEC-REVIEW-REQUEST, docs only; no production or RED test authorized.
Base:57fea94989ff362129d0e53da0c35c298822f689.
Branch:worker3/support-cubic-localization-spec.
W6 owns Adaptive production files. W3's approved N-split79891db69 remains
frozen on its separate branch, with unchanged HEAD and clean worktree.

## Bounded assignment and public reproduction

The60min window started2026-10-14 22:02:11 Asia/Singapore, deadline23:02:11.
Runtime localization completed22:19:27,1036s (~17min); no further runtime
investigation is planned. All debugger inferiors inherited CPU12, ASLR off,
and fresh cache/artifact/output roots. No builds, source edits, inferior
function calls or inferior data substitutions were performed by W3.

Frozen W6 S45 input:
/tmp/worker6-adaptive-spacing-red.mUVyz6/official/artifacts/adaptive-spacing-69TKxR/input.3mf
SHA256:8e1d3831a3db446cd62f4c381d07281520f6cc9b0b5fa3a39b295e2d0a3dcc35.
S45 is one object/printing region, SupportCubic, density30, width0.45,
multiline1. Its public Native/API/CLI exports are admitted. Both complete
streams have5425 lines. The archived comparator first differs at byte36369,
layer4 line1418:Native `G1 X141.02 Y131.734 E.04926`, Ares `E.04925`.
This is the retained full-byte RED capability; no E/XY/timing/footer
normalization, relaxed tolerance or diagnostic PASS is proposed.

W3 replayed that archived public difference before source hypotheses. Fresh
read-only Native official/seeded exports reproduce the frozen whole Native
stream under the existing generated-by-line treatment. Fresh Ares exports
are raw-byte identical to the original S45 Ares stream. No input minimization
or replacement was needed; fixture/member bytes remain frozen.

## First measured value and independent state differences

The first verified scalar difference at the object-octree build boundary is
`line_spacing`, supplied by PrintObject.cpp:977/1007. This is not a claim
that every earlier configuration/pipeline stage was instrumented.

| Captured field/stage | Native official and seeded | Ares |
|---|---|---|
| build entry line_spacing |4.4999998658895537|4.4999998852907108|
| build entry support_overhangs_only |true|false|
| original bridge payload at entry |6 Vec3d points /2 triangles|empty|
| source mesh input |8 Vec3f points /12 ordered triangle indices|8 Vec3d points /same12 indices|
| actual top-level insert calls |4:2 filtered mesh +2 original bridge|12|
| root center, first component |1.1546878814697266|1.1546894319625274|
| bbox maximum size |67.160446166992188 (source-derived from captured Vec3f)|67.160442681381397 (actual make_cubes_properties argument)|

Spacing delta Native−Ares is−1.9401157125287227e−8. All8 captured Native mesh
vertices equal the respective captured Ares coordinate after source f32
rounding, and the12 triangle-index triples have identical order. Therefore
these captures distinguish scalar precision/caller selection from changed
public mesh topology or input identity.

Actual Native root centers are
[1.1546878814697266,1.1547050476074219,1.1547060012817383];
actual Ares root centers are
[1.1546894319625274,1.1547060915475527,1.1547060915475544].
The Native derived float bbox center matches its actual root center in all
four top-level calls. The Native bbox maximum above is labeled derived;
it is not an instrumented Native make_cubes_properties local.

Native actual first two root triangle payloads match mesh triangles8/9;
its next two exactly match the ordered six-point bridge payload supplied at
build entry. Both modes have identical input/state/payload observations.
Ares root addresses/centers and12 call counts are readable. Its destructured
triangle locals are optimized out: per-call Ares triangle payloads are
explicitly unavailable, never reconstructed and labeled as observed.

## Exact source boundaries

- Fill/FillAdaptive.cpp::adaptive_fill_line_spacing:276–357:
  :287–309 retains config density/width as double; :315–340 identifies actual
  contributing regions; :344–348 averages and computes width divided by
  density/100 times the widened f32 one-third; :352–354 applies multiline
  to both Adaptive and Support spacings. A45's formula also owns Support;
  it is not the f32 Flow.width/per-fill-density approximation.
- PrintObject.cpp::prepare_adaptive_infill_data:972–1007:
  spacing pair at977; raw mesh transform at981–984; original candidate
  ExPolygon triangulation at995–996; ordered gather at1002–1003; Support
  uses support_line_spacing and true at1007 (Adaptive false at1006).
- PrintObject.cpp:2728–2735 gathers each CandidateSurface.original_surface
  and its region layer bottom_z BEFORE rewrite and prepares both object
  trees. The pair stores z as float, then triangulation widens it to double.
  This input is not the final InternalBridge-tagged surface collection.
- Fill/FillAdaptive.cpp::build_octree:1483–1530:
  :1495–1497 float mesh bbox, widened center and maximum size;
  :1512–1518 Support-only overhang predicate over source float vertices
  widened to Vec3d; :1520–1521 always inserts original bridge triangles;
  :1523–1527 rotates completed tree centers/origin back to world frame.
  is_overhang_triangle:1465–1469 uses the original cross/dot/norm predicate.
  CubeProperties ladder:1434–1462 retains source arithmetic/order.
- BoundingBox.cpp:182–185/232–235 owns f32 size/center arithmetic.
- Tesselate.cpp::triangulate_expolygon_3d:192–195 delegates to
  GluTessWrapper::tesselate3d:23–56, submitting contour and holes in order.
  Tesselate.hpp:13–16 supplies NORMALS_UP as the default. This separate
  primitive is a prerequisite for original candidate triangle payloads.

These are libslic3r rewrite boundaries. Existing Ares fill/octree and bridge
candidate stages are temporary compatibility shells around those objects,
not a new Ares pipeline or a different support-density policy.

## Exact Rust destination proposal and ownership

This proposal describes the required Support preparation boundary; it does
not authorize implementation or a numerical-only GREEN. Each sub-slice below
requires its own RED/GREEN approval; tessellation is independently reviewed.
Rebase a future implementation after W6's production chain is accepted;
coordinate any shared projection/carrier change with W6 and W2 first.

1. crates/ares-core/src/project_slice/fill_entities/adaptive/line_spacing.rs
   ::configured and ::for_fill (W6 chain), plus
   crates/ares-core/src/project_slice/group_fills/params/projection.rs
   ::project_surface: carry the actual source region's config spacing to
   Support as well as Adaptive. Preserve W6's exact configured arithmetic
   and ownership; do not reconstruct it from f32 Flow or select regions[0].
2. crates/ares-core/src/project_slice/fill_entities/adaptive.rs
   ::build_object_octree and ::octree_for: Support must use its own
   configured spacing, source float-bounds preparation, true filtering and
   original candidate triangle payload. append/anchoring_lines share the
   same prepared object tree; preserve W6's kind-disambiguated cache.
3. crates/ares-core/src/fill/adaptive/octree.rs::build_octree and the W6
   ::build_octree_from_bounds/float_bounds::of boundary: source Vec3f bbox
   arithmetic before widening; source mesh vertices widen from f32 for
   Support triangle tests. Bridge Vec3d payloads must NOT be globally cast
   to f32. Preserve ladder, SAT, child order, root/world transformations.
4. crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/
   transaction/candidate_expansion.rs::prepare_object, immediately before
   prepare_infill_lines; candidate_expansion/layers.rs::prepare_infill_lines
   and types.rs::{CandidateSource,BridgeCandidateObject}: resolve each
   original surface by its retained layer/region/surface identity, pair it
   with source float bottom_z, and hand the ordered payload to object-tree
   preparation before anchors/rewrite. Preserve original candidate order;
   do not collect only already-retagged InternalBridge surfaces.
5. The independently reviewed tessellation prerequisite has its own Ares
   destination: `crates/ares-core/src/geometry/tessellation.rs`
   ::triangulate_expolygon_3d, with safe CPU implementation files under
   `geometry/tessellation/` as necessary. It ports Tesselate.cpp:23–56,
   :100–170 callback emission and :192–195; Tesselate.hpp:13–16 fixes the
   normal/flip contract. It must preserve contour/holes, winding, generated
   intersection points and ordered triangle emission. No TS implementation,
   external Rust tessellator, GLU/OpenGL FFI or other runtime library.
   Core has no matching primitive; organic branch_mesh fan/strip helpers
   port TreeSupport3D, not this ExPolygon/holes boundary.

No public load/selection/options contract change, global cache, feature flag,
new Unsupported guard, empty fallback, Native FFI/OpenGL/GLU runtime, or
fixture-specific triangulation is included. Every Rust file remains<=400LOC.

## Causal limits and mandatory prerequisite STOP

W6 explicitly confirmed in inbox22:05:05 that Support output is unchanged
by its Adaptive GREEN. Its prior combined spacing+bbox+actual-root/origin
intervention still fails S45 at the same public line1418. W3 did not run
those write interventions; their evidence is retained as W6's diagnostic,
not public acceptance or a source-correct GREEN.

The new flag/payload/insertion observations establish additional unported
Support preparation state. They do NOT prove that changing those fields
alone closes every byte or that downstream fill/extrusion has no residual.
No numeric-only patch is proposed as complete SupportCubic parity.

Before Support GREEN, the independently accepted geometry primitive and
original candidate domain/bottom_z preparation must be present. Do not
invent a rectangle fan, copy Native triangles into Rust, use a different
triangulator merely because S45 is a quadrilateral, omit bridge triangles,
add a guard, mutate Native or relax output comparison. An unclosed full S45
or earliest changed geometry domain remains a STOP for amendment.

## Ordered sub-slices: each RED closes within its own boundary

### B-T: standalone GLU tessellation source port, first

Upstream: Tesselate.cpp::GluTessWrapper and triangulate_expolygon_3d, cited
above. Ares owns it in the geometry module named in destination5, platform
neutral safe Rust, no TS or external libraries. It does not change Support,
Adaptive, candidate ownership or the infill caller while this slice lands.

The Native-backed seam is an actual Native public 3MF slice that reaches
triangulate_expolygon_3d: read its input ExPolygon/holes/z/flip and its returned
ordered Vec3d triples using the approved read-only GDB flow. Pair that oracle
with the public Ares geometry API `geometry::triangulate_expolygon_3d` over
identical pure polygon inputs. Verify COMPLETE ordered triangle coordinates,
orientation, count and order, not only area/set equality. Native products
remain runtime artifacts; author pure input fixtures/generators only.

Preflight simple/concave/hole and normal-up/down controls with Native before
freezing the RED matrix; no unobserved case is labeled Native-admitted.
Inventory the actual Native GLU supplier/source algorithm behind the wrapper
before implementation: Tesselate.cpp owns the wrapper but delegates the
kernel. If its deterministic emission order cannot be source-identified,
STOP and submit an algorithm addendum, not an independently designed earclip.

The core currently lacks this public geometry entry. Its first TDD test must
therefore expose the absent API at compile time; Native admission/results
are collected independently first. This is an explicit proposed compile RED
for a new geometry API, NOT a claimed Native-runtime parity failure or PASS.
The RED commit must name that distinction for coordinator approval before
GREEN; no empty/Unsupported stub, cfg skip or fallback may manufacture a
runtime RED. If a runtime-only RED is required, STOP for a seam ruling rather
than pretending the existing whole S45 comparison isolates tessellation.

Once the API exists, the same tests close on the geometry port alone. Support
S45 may continue to diverge and is neither a B-T PASS nor B-T exit criterion.
Keep all current pipeline controls byte-identical; strict/fmt/wasm precede
its independent review/full allocation. Do not merge a RED without GREEN.

### B-S: complete Support object preparation, after B-T and W6 Adaptive

This slice ports destinations1–4 as one complete Support preparation boundary:
config spacing, Vec3f bounds/mesh cast, support_overhangs_only, ordered
original bridge payload and shared anchor/emission tree lifecycle. Its
prerequisites are the accepted B-T primitive and W6's Adaptive source chain;
there is no later geometry dependency left intentionally missing.

Freeze the existing authored S45/S50/S125/SAUTO inputs and their generators/
provenance. These four width cases, one region/count1, are the intended
Support matrix. Re-run Native admission plus complete public API/CLI output
in both modes, CPU12, fresh cache/root. Commit genuine public behavioral RED
separately and report before production. All original failures remain failures.

GREEN must make the complete four-case matrix strict PASS on its own, preserve
W6 Adaptive A45/A50/A125/AAUTO and every non-Support raw control, and keep
assertions/comparator unchanged. A spacing/bbox-only RED is deliberately not
scheduled: existing Native evidence proves it cannot close S45 independently.
Residuals STOP for localization/addendum; do not label a partial patch GREEN.

No tests/fixtures/production have been added by this docs branch. Each
sub-slice requires separate RED approval, GREEN review and allocated full;
strict/fmt/wasm on its exact final head. Shared private carrier adapters need
owner coordination and the existing approval rules. No push here.

Deferred: multi-region config averaging/selection changes, multiline>1,
other fill patterns, octree child/SAT/chaining redesign, triangulation of
unrelated organic meshes, G-code E/ETA adjustments, independent public
load/selection contracts and Support implementation before its prerequisites.
No new admission guard marks these deferred behaviors.

## Capture provenance, corrections and full-stream audit

Valid Native seeded:/tmp/worker3-support-seeded-f_ugpjhl.
Valid Native official:/tmp/worker3-support-official-5712y_nh.
Final Ares state:/tmp/worker3-support-ares-0ysa48ym.
GDB/actual inferior exit0 for all three; recorded affinities12.
Native source ABI/layout reads are checked against symbol-relative assembly:
build stop+20 precedes writes to ecx/xmm0/rdx/rsi, preserving entry args.
Mesh vector strides12 and double bridge stride24 are validated by counts and
exact payload matches; actual live root pointers are only read, not modified.

Ares producer is the immutable main57 CLI copy supplied by W6:
/tmp/worker6-adaptive-baseline-main57/ares, SHA256
978c0b308749484b85309cb63dc03ab5901e494b8c693c8fbab107d8cee09ffc.
This is NOT an actual be637/6cf entry capture. W6's independent six raw
Support/control outputs and source-equivalent Support spacing/build tail
carry its applicability after Adaptive GREEN; its tuple-only successor
6cf712483 is not rebuilt or modified by W3.

Raw S45 Ares SHA (all valid/partial Ares runs, identical to frozen public):
d5d67d96abf4f4d22f67685120d5d1d0ebc4ad71082f334bc8609995f13807a8.
Fresh Native raw hashes (only generated-by identity differs from frozen):
seeded11bea56b5182dde04e4a2dc7b28d74c06269dd10ad26956d3e24709b924c6a84;
official86f3954a6ad42407d4dee16e391dcd7774c38dbc9b166692844112faf1dc7660.

Retained correction roots:seededwdruh63f was INVALID_SETUP, run redirection
replaced slicing args, causing GUI exit1 and no build observations (GDB0
was never treated as Native success). Fresh f_ugpjhl fixes plain run args.
Aresbba_0e0l has valid entry/bbox only; r61oakjv has readable entry/bbox and
optimized-out triangle payloads, not a full payload trace. Final0ysa48ym
preserves actual roots/counts with explicit unavailable triangle fields.
No missing data was filled with guessed coordinates. All roots are retained.

## Durable receipt and coordinator order amendment

Coordinator's follow-up ruling requires independently closable sub-slices and a
standalone geometry/WASM-safe GLU port with no TS/external libraries. This
document therefore orders B-T geometry first, then complete B-S preparation;
no unclosable spacing/bbox-only RED is scheduled. The absent geometry API
compile-RED distinction and native GLU supplier preflight are explicit.

Evidence archive outside Git (all valid/partial/invalid roots, commands,
ordered state, assembly, public four-case inputs/outputs and W6 carry proofs):
/home/indexyz/ares-evidence/worker3/support-cubic-localization-20261014.tar.gz
Bytes:715107; SHA256:72dd6ab82acb207290ac86f9bd266ee5707a5df3eb2e3d3f95d974739f63f0ec.
Compiler binaries and caches are excluded. No Native products enter Git.
Committed value/count/hash summaries above survive /tmp's10day cleanup.

Frozen N-split79891db69 was verified unchanged/clean after this work. The
new branch contains documentation only; no rebase/amend/push on that frozen
branch, no production, no new tests, no full/workspace/builds.
