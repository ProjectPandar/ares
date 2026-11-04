# PlanePath sparse object bounds after XY compensation

Native-first discovery only. No public RED or
production edit. Base main:
`9c097e3c41d066c996fa85114cf44fe5e109fa14`.

## Source boundary, not an Ares pipeline design

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `PrintObject.cpp:95-112` obtains the transformed model object's raw bounds
  and truncates scaled size into `m_size`. `Print.hpp:347`
  `PrintObject::bounding_box()` halves that integer size into a symmetric
  box around zero. It is not the post-XY-compensation slice envelope.
- `Fill.cpp:1224,1237` (`Layer::make_fills`) and `:1398,1442`
  (`Layer::generate_sparse_infill_polylines_for_anchoring`) both assign
  that object box to the filler.
- `FillPlanePath.cpp:75-118`, `FillPlanePath::_fill_surface_single`: below
  density0.995, rotate the supplied object box; Hilbert uses its minimum as
  shift, while centered Archimedean/Octagram use its center. Generate on this
  object-wide frame and clip to the independent snug/contracted surface.
- `FillBase.cpp:105-118` still owns base contraction. XY compensation changes
  the surface, not the model-size reference frame used for sparse alignment.

Rust currently substitutes all compensated slice-contour extents:
`fill_entities/plane_path.rs::object_bounding_box`, consumed in
`fill_entities/generate.rs:283-296` and the PlanePath anchoring arm via
`sparse_anchoring/{mod,projected}.rs`. The compatibility shell conflates
surface/clipping bounds with upstream object reference bounds. This slice
replaces only PlanePath callers with the cited PrintObject concept, not
another fill algorithm or an independently designed geometry pipeline.

## Independent witness and bounded controls

Reuse the authored anchoring mesh/profile bundle (10mm model,
classic, single object/instance, direction45, sparse20%, multiline1,
layer0.2, top6, no ZAA). Preserve all input ZIP members. Vary only sparse
pattern and `xy_contour_compensation` after that common setup.

Both canonical official and seeded modes use two fresh Native references
per input. All repeat and cross-mode complete normalized bytes agree:

| Sparse pattern | XY compensation | input SHA256 | outcome, both modes |
|---|---|---|---|
| HilbertCurve | 0 | `50db1ed174c0914cbfbe0b36ca4d4b21e22d36acca3125e05b343c3db2af92f2` | PASS |
| HilbertCurve | +0.1 | `2d59332f77beced29dddeff87eded5d8fb6d980042c8566cd46c36de630644b3` | DIVERGENT |
| Rectilinear | +0.1 | `d57952e651047eabddf0afb6ce97805ea3b5ddfc08fbb67cbd01cd54728cc022` | PASS |
| ArchimedeanChords | +0.1 | `07f2127d3b9c85f3660da93f2732c554de1ccb1e558e2d6718ae36f56b2e3859` | PASS |
| OctagramSpiral | +0.1 | `12d4b5c3d98e26014e3166dfaf90a2dde8b57bb3a8e5120bd40393c3e2e7e970` | PASS |

All20 fresh references admitted the same inputs/settings. Rectilinear with
identical compensation rules out a general compensated-surface/perimeter
failure. Hilbert0 isolates the compensation-triggered frame mismatch.
The two other FillPlanePath subclasses are explicit shared-boundary controls,
not unrelated pattern widening. No alternative mesh or geometry search.

Target first strict difference: byte1797,line82; Native `M73 P9 R8`, Ares
`G1 X102.334 Y104.9 E.31342`; normalized lengths114343/112936. Retain that
M73 difference. `GCodeProcessor.cpp:1232-1258` depends on whole-file duration;
earlier progress placement is not the owning fill boundary.

All G0/G1 rows in the first five solid-only layers match. The first Sparse
layer is physical6; first coordinate-token divergence is its entry travel:
Native source729 `G1 X113.856 Y105.88 Z1.6`, Ares source728
`G1 X114.056 Y105.88 Z1.6`. The first positive Sparse extrusion (source734)
has the same formatted endpoint but E.00875 versus E.00212. Both have936
positive Sparse extrusion rows; equal counts do not establish equal geometry.
Motion/role observations are localization, never replacement acceptance.

## First inconsistent intermediate

Read-only GDB on the actual seeded reference executable + existing matching
DWARF and the already verified main9c097e3c4 Ares CLI:

- Native PlanePath object bbox is exactly `[-5000000,5000000]` on each axis
  in both anchoring and emitted calls at layer43, despite +0.1 compensation.
- Ares anchoring PlanePathFillParams carries
  `[-5100000,5100000]`, with its backtrace identifying the anchoring arm.
  Its emitted PlanePath adapter also receives `[-5100000,5100000]`.
- Native contracted anchor/sparse domains remain independently compensated:
  anchor `[-4119688,4119693]`, emitted `[-3712599,3712604]`. These are not
  instructions to remove surface compensation or change base contraction.
- Matching default geometry before Sparse and the Rectilinear control,
  combined with these two bbox arguments, distinguish the reference-frame
  discrepancy from generic compensation or Hilbert generation defects.

No debugger mutation, diagnostic build, nix invocation, gcroot or temporary
production instrumentation. Earlier missing optimized/local debug fields
were not used as proof; bounded entry probes completed normally. Canonical
fresh Native output alone supplies the strict verdict. Closure after the
bounded port is not yet claimed; any residual means STOP/report.

## Exact proposed Rust destination and preserved behavior

Include emitted and projected anchoring FillPlanePath object bounds for
HilbertCurve, ArchimedeanChords and OctagramSpiral, initially witnessed through
ordinary sparse fill. Full-density solid/ironing uses snug bounds and is a
control, not new behavior. Reduced-density solid fixture expansion is deferred;
no role-specific guard or alternate bbox rule is introduced. Keep all non-PlanePath
arms, especially consistent-pattern extended-grid behavior, unchanged.

An existing source-cited raw model-size calculation is already present in
`fill_entities/stars_bounds.rs::extended` (`Model.cpp:1586-1599`,
`TriangleMesh.cpp:440-446`, `PrintObject.cpp:112`, `Print.hpp:347`). Factor its
centered, integer-halved **base box** into one private reusable function;
Stars' transpose-merge/sqrt2 extension must remain byte-equivalent. Reuse it
from only:

1. The PlanePath branch in `fill_entities/generate.rs`, with the actual
   source object and owning print-object no-offset transform.
2. The PlanePath branch in `sparse_anchoring/arms.rs`, with the same owner.

A necessary crate-internal export through `fill_entities.rs` is allowed for
that existing helper boundary; no new public API, state/cache framework,
capability, guard, fallback or per-point defensive copy. Keep ModelPart
membership, float-vertex-to-double transform and truncate-size-then-halve
order. Do not hardcode the witness10mm box or compensate the bbox manually.
The contour-derived helper remains for unrelated current callers; do not
replace the shared consistent-pattern closure or speculate about its behavior.
Remove/update only comments made stale for the PlanePath calls. A mechanical
leaf split, if required for <=400LOC, is separate and changes no behavior.

Included evidence is one classic, unrotated single instance. Existing Stars
transform/bounds controls must stay exact; new complex-transform, modifier,
shrinkage, Arachne/multi-object bounds algorithms are deferred, not new guards.
No shared PlanePath generator, rotation/ceil/shift formula, clipping, chaining,
multiline, ordering, flow/E, bridge-angle, M73/time/cooling, support/raft or
independent pipeline design change is in scope.

## Separate RED then bounded GREEN

Freeze five SHA-pinned input-only cases from the
matrix. Reuse existing runner/artifact/golden infrastructure for public API
and CLI; do not clone a full harness or weaken the existing anchoring tests'
Sparse-equality invariant, which is not the invariant of this emitted-fill
slice. Require effective pattern/compensation settings, fresh Native repeats,
positive Sparse activity and the target's independent Native geometry delta,
raw API/CLI equality and unchanged strict whole-file Native equality.
Discovery currently covers CLI, not new public API acceptance.

Expected RED: Hilbert+0.1 FAIL and all four controls PASS in both modes,
including the other PlanePath patterns. Retain the initial-solid-layer motion
control. Keep M73/feedrates/E and all bytes under the existing comparator.
No fixed Native output, tolerance, inverted assertions, ignored tests or
UnsupportedProjectFeature/allow/expect additions. RED lands on an exact clean hash
before production.

GREEN only changes bbox provenance at the two named PlanePath consumers and
the reused source base-box helper extraction. Preserve every other value,
algorithm and all frozen inputs/assertions. Strict residual or unexpected
non-RED output change means STOP/report, not widen the fix.

Exact-final-hash gates: fmt, workspace all-target clippy `-D warnings`, wasm32,
new and existing PlanePath/Stars/consistent-grid/anchoring controls in both
modes, then canonical official/seeded FULL with fresh artifacts/cache and
zero FAIL/skipped.

No RED/public fixture/production change is part of this SPEC.
