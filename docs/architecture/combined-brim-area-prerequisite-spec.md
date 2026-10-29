# SPEC request: global combined-brim area consumer prerequisite

Status: **DOCS-ONLY SPEC-REVIEW-REQUEST**, discovery only completed; no public RED
or production authorization. Fresh branch `worker1/brim-combine-prereq-spec` from
`origin/main 41f2c90f765d42c6620583121296bb61a2cb839c`, as explicitly assigned.
Upstream source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
FULL24 is active; all probing was targeted CPU18, no full/workspace/merge.

## Premise reverified on exact main, not the pending W1 chain

Fresh `/tmp/ares-brim-control-main-premise/discovery-report.json`: official and
seeded Native FIRST, independent roots/cache, current coordinator canonical
archived, current main API/CLI rebuilt, sampled descendants/threads pinned CPU18.
Input SHAs retained. Native accepts all three inputs; Ares accepts all with raw
API==CLI. Whole-file comparisons use the existing generator-only normalization.

- combine_brims1 / outer_only / combined skirt: first strict difference
  byte5513, line184, Native
  `G1 X127.841 Y103.322 Z.6 ; move to first brim point`, Ares Y103.321.
  Normalized expected196559 bytes / actual197090. Identical result both modes.
- copies / outer_only / combined skirt: first strict difference byte1608,
  line67, Native `G1 X74.265 Y90.954 ; move to first skirt point`, Ares
  `G1 X73.627 Y90.998 ; move to first skirt point`. Expected281174 /
  actual281007 bytes. Identical result both modes. This includes one of two
  PrintObjects with two instances; it is not a single translated object.
- SAME combine input with only combine_brims changed1→0: strict whole
  Native/API/CLI PASS, raw API==CLI, both modes. This is the passing paired
  control; outputs of DIFFERENT inputs are not required to equal one another.

All are ByLayer, one filament, no support/raft/tower, outer_only width8/gap0,
existing geometry and instance placements. No geometry/setting workaround,
rejection-as-success, coordinate tolerance or Native-output fixture.

Select **combine_brims global area consumer** as the smaller source-owned slice:
it has a direct missing configuration consumer and a single-setting strict-PASS
control. The copies failure occurs already in skirt positioning and spans
instance-footprint ownership; defer it, do not bundle it with this repair.
The 0.001 first difference is only an observable symptom, not permission to
round/snap a coordinate or claim all remaining differences are the same bug.

## Owning upstream sequence and current Rust gap

- `OrcaSlicer/src/libslic3r/Print.cpp:2532-2543` clears brim maps and calls
  `make_brim`, obtaining per-object occupied footprint maps before `_make_skirt`.
  There is no separately named `_make_brim` function in this pinned file;
  `Print.cpp:2646::_make_skirt`, :2798-2802 expands object hulls by instance
  shifts; :2847-2971 preserves object IDs and associates group brims. This
  locates the copies prerequisite without claiming its root cause is proved.
- `OrcaSlicer/src/libslic3r/Brim.cpp:890::make_brim`, :914-924 accounts instance
  translated object/support contours, :945-956 stores translated brim footprint
  maps for skirts. Those footprints are preserved separately from emitted paths.
- `Brim.cpp:958-962` defines the exact GLOBAL combine gate: combine_brims &&
  no active perobject skirt/shield && sequence != ByObject. :964-977 is the
  original separate-object path, retained for the passing control.
- `Brim.cpp:979-998` gathers nonempty object brim areas, `union_ex`, then
  `offset2_ex` cleanup with `max(float(scaled_resolution), float(SCALED_EPSILON))`,
  jtRound and source resolution; generate infill ONCE via makeBrimInfill.
- `Brim.cpp:1000-1018` chooses deterministic carrier: first contributor in
  objPrintVec print order, falling back to first contributor only if none found.
  Preserve the source algorithm, not an arbitrary Ares first-vector rule.
  `Brim.cpp:836-886::makeBrimInfillImpl/makeBrimInfill` owns path generation,
  chaining and plate-offset convention; reuse matching existing Rust consumers.
- `GCode.cpp:4486-4531::generate_object_brim` emits one global map entry under
  that same global gate, plate origin0, once-only owner bookkeeping; :5334-5335
  orders per-object skirt then brim before the instance's model extrusion.

Rust destinations (full paths):
`crates/ares-core/src/project_slice/gcode_emit/orchestration.rs::emit` currently
calls `brim::BrimPlan::generate` independently for each traversal object then
immediately constructs skirt from those plans.
`crates/ares-core/src/project_slice/gcode_emit/brim.rs::BrimPlan::generate`
clips/retains the per-object area, derives covered_hull, and converts it to paths
via `crate::brims::ears::paths::from_area`; no global combine consumer is present.
`crates/ares-core/src/project_slice/gcode_emit/brim/bed_area.rs::clip` translates
model-local↔global geometry; preserve that existing bed/coordinate convention.
`crates/ares-core/src/project_slice/gcode_emit/brim/emit.rs::BrimPlan::emit` emits
every stored per-object path. The only project_slice combine_brims reference
on this base is the skirt unsupported guard, not a make_brim merge.

Prospective minimal port after authentic public RED approval: retain the existing
source-owned pre-infill per-object areas at the current brim preparation seam,
apply ONLY the cited global gate's union/cleanup/infill and deterministic carrier
semantics, then feed the existing emitters/skirt footprint consumers. Keep
separate object occupied footprints for skirt grouping, not just the merged
carrier hull. Source geometry cleanup occurs before path generation; merging
already-generated polylines or rounding emitted XY cannot implement it.
Bounded extraction into private brim files if required to stay <=400 LOC is
permitted only for this source seam, not a generic planning framework. No new
public API, core I/O/env, policy abstraction, independently designed pipeline,
legacy fallback or new error behavior. Existing emitter is a compatibility shell
for Print/Brim/GCode source concepts. ARD decisions unchanged.

Exact numerical geometry provenance must still be established from area/path
artifacts during RED localization; missing consumer is a justified hypothesis,
not a claim that a patch is proven by the first Y mismatch alone. If differing
upstream path arithmetic/other owner is necessary, stop for amendment.

## Planned public Native-oracle RED and control

After SPEC approval only: freeze combine1 failing input plus its combine0 paired
control as input-only 3MFs with SHA/member manifest. Native generated at runtime
FIRST with OrcaRunner::from_env; assert acceptance, saved/effective configuration,
actual brim extrusion and ordinary skirt/model output. Same input then validated
API and real CLI. BOTH tests assert strict complete ordered Native equality and
raw API==CLI; target must FAIL the real whole-file first difference until GREEN.
No inverted assert_ne/is_err/divergent-is-success branch. Anchors supplement,
never replace, full-file comparison. No Native outputs in Git.

Localize full changed brim commands and source area/cleanup/infill/carrier mapping
without changing fixture/model/comments/resolution. Preserve original RED output
and do not assume a single 0.001 formatting fix. Independent RED review before
production. Same accepted control's raw API/CLI bytes must remain unchanged
RED→GREEN. Existing ordinary brim and combined/perobject skirt controls remain
unchanged. Fresh serial official+seeded CPU18 jobs2/threads2, separate caches/
artifacts, canonical archive/hash, actual Native affinity, exact clean hash,
allocated fmt/strict/WASM. Broad gates only coordinator-authorized.

## Failure inventory / scope / deferred prerequisites

Native rejects or output absent: NO-RED STOP for the proposed accepted-output
slice. Environment/compiler/timeout/cancel/witness fault: NON-GATE. Paired combine0
control fails: stop and report before changing production. Wrong frame, double
translation, missing owner/carrier, duplicated brim or skirt footprint change
are full-output failures, not tolerated noise. Never change comparator,
classification, rounding/tolerances, unsupported gates or fixture to hide them.
If all match on a new base, evidence-only NO-RED STOP, not accounting work.

Included initially: existing two distinct singleton objects, combined skirt,
outer_only, one filament, ByLayer, no support/raft/tower; global combine gate vs
same-input combine0 control. Deferred: copies/instance footprint failure,
perobject skirt combine_brims admission, group-local combined brims, ByObject,
inner/auto/ears/painted/support brims, multiple filaments, touching/overlapping
geometries beyond this input and new producer geometry. Keep combine_brims and
multi-instance perobject guards untouched even if this prerequisite is repaired.
Current pending W1/W2/W5 guard conflicts are coordinator-owned and not part of
this main-based source slice. No full/workspace/merge/push during FULL24.
