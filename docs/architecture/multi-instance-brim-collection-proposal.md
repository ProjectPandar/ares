# SLICE-PROPOSAL: PrintObject multi-instance brim area collections

Status: proposal only; Native/public discovery complete, awaiting SPEC approval.
No new test or production change. Base inspected exactly
`cf8597e183d44bd24a28959e4dbd904d3a90512f` (approved global-combine chain queued
stack25), atop `4351a644d6683555ae48e44ea7f898a66f75d585` stack24.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Observed residual, not covered by the just-approved slice

Fresh evidence `/tmp/ares-copies-premise-cf8597e18/discovery-report.json`.
Official and seeded, independent fresh artifacts/cache, runtime Native FIRST,
CPU18 actual descendant/thread masks checked, current canonical archived/hash
checked, exact clean HEAD before/after. API rebuilt from exact hash and real
CLI both used. Comparator unchanged, complete ordered output, API==CLI raw.

Original copies input SHA256:
`aac4f3c278fe16614d0694ba9fbe94db059e7bb68d75c7cf55f7ecf0f47aa07d`.
Two PrintObjects, THREE instances: 3MF object2 at (80,110,2) and (140,110,2),
object3 at (110,110,2), Z scale0.4. One filament, ByLayer, outer_only width8,
gap0, combine_brims0, combined skirt2, no support/raft/tower.

1. **Original accepted divergence**, both modes: first strict difference
   byte1608/line67, Native `G1 X74.265 Y90.954 ; move to first skirt point`,
   Ares `G1 X73.627 Y90.998 ; move to first skirt point`.
   Expected281174 / actual281007 normalized bytes. Native two Brim feature
   blocks, Ares THREE. Native's first block includes both object2 islands
   (extrusion XY bounds X67.647..152.353, Y97.647..122.353); second contains
   object3's middle island. Ares instead emits right, middle, then left as
   separate blocks. This is not merely an inaccurate starting coordinate.
2. **Same-input passing control**, only brim_width8→0: Native/API/CLI full
   PASS both modes; raw API==CLI; no brim blocks, ordinary copies/skirt/model
   output retained. All other config and all non-config ZIP members identical.
3. **Localization input**, only skirt_loops2→0: Native accepts, strict whole
   comparison FAIL both modes, first byte6605/line213. Native next command
   `G1 X147.921 Y99.085 E.02293 ; brim`, Ares next `M73 P8 R10` (that G1 follows
   afterwards). Native still TWO Brim blocks spanning both copies in one owner
   collection, Ares THREE. Expected277240 / actual277359 normalized bytes.
   This is a failing localizer, NOT a passing control. Never discard M73 or
   relabel this a timing-only mismatch; full output remains the acceptance gate.

Brim block counts/bounds are supplementary localization only. Whole ordered
comparisons above are unchanged acceptance comparisons, not geometry tolerance
or TYPE-count equality tests. No private Native ExPolygon trace is claimed.

The original mismatch was also observed on exact main41f2c90f7 before stack24
(`/tmp/ares-brim-control-main-premise/discovery-report.json`), and persists on
stack24 + approved global-combine chain. The latter consumer is gated OFF here
(combine_brims0). `skirt/occupied.rs` has no changes between stack24 and this
HEAD; last source owner change predates stack24. Thus the observed behavior is
not already supplied by main/stack24/global-combine. Coordinator should check
other queued worker ownership before assigning implementation.

## Owning upstream slice, not an Ares feature or guard widening

- `OrcaSlicer/src/libslic3r/Brim.cpp:36-45::append_and_translate` translates an
  instance by shift_without_plate_offset, subtracts existing destination
  footprint and appends to brimAreaMap keyed by PRINT OBJECT ID.
- `Brim.cpp:573-581::outer_inner_brim_area` traverses ALL instances per object
  for brim/no-brim masks, holes and object-island contact geometry. Bed/no-brim
  clipping/contact in :636-689 must keep its source frame/lifecycle; do not
  duplicate final already-clipped local polylines instead of area expansion.
- `Brim.cpp:914-956::make_brim` accounts all instances in object/brim bounds and
  stores plate-translated per-owner brim area maps before path generation.
- **combine0 ordinary branch** `Brim.cpp:964-977` calls makeBrimInfill ONCE for
  the complete area belonging to EACH PrintObject, including all its instances;
  it is NOT once per copy. Keep separate owner maps, unlike the already ported
  global branch :979-1018 that unifies every owner into one carrier.
- `Print.cpp:2794-2816::_make_skirt` expands object hulls per instance then adds
  ALL contours from object/support brim maps to occupied points. Existing
  skirt group generation and loop history remain consumers of those maps.
- `GCode.cpp:4486-4531::generate_object_brim` consumes/erases owner IDs once,
  emits the complete owner collection at origin0, before its model entry
  (:5334-5335). Copies do not each re-emit the same owner map. Existing group
  carrier/order logic must follow Print.cpp:2847-2971, not an arbitrary order.

Exact Rust destination boundary:
`crates/ares-core/src/project_slice/gcode_emit/brim.rs::BrimPlan::generate`
and `brim/bed_area.rs::clip` currently prepare a local first-instance-compatible
brim then discard pre-infill area on combine0; `brim/emit.rs` exposes local
paths/hull/bounds. `skirt/occupied.rs::items` shifts the model hull for every
instance but appends a brim hull only at `footprint::print_object_origin`, which
is explicitly the FIRST instance. `layers/brim_emit.rs::emit` reuses the local
plan for each copy on physical layer0; it lacks ordinary per-owner once-only
collection bookkeeping. `orchestration.rs::emit` is the existing preparation
seam. Rewrite these compatibility shells toward the source keyed owner-area
map and per-owner emission lifecycle; no independent pipeline or public API.

## Proposed bounded work after approval

A cohesive rewrite of **per-PrintObject instance-expanded brim area → occupied
footprint and ordinary brim collection** for this accepted combine0 fixture.
Retain/reuse the just-approved source global-combine implementation; it is not
permission to unify the ordinary owner maps. Preserve separate occupied
footprints and source frame transforms, clipping/contact, per-owner infill-once
and once-only emission. Reuse current source infill/geometry primitives, not a
copy-count-specific coordinate offset. All changed Rust files <=400 LOC; small
private brim files may hold this existing source responsibility if necessary.

After approved SPEC: freeze original copies-control + same-input width0 control
as input-only 3MF/SHA/member manifests; public runtime Native-first RED asserts
TARGET whole Native/API/CLI strict equality and genuinely FAILS. Control asserts
full equality and raw API==CLI too. No passing-because-divergent expectation.
Capture both independent modes at exact clean hashes, CPU18/jobs2/threads2.
Preserve passing control raw bytes RED→GREEN. Localizer may be frozen as a
separate genuine failing case only if review includes its source boundary;
it is not to be misrepresented as a control. Independent RED approval before
production; source provenance/area-owner lifecycle before GREEN. Existing
singleton/grouped brim/skirt and global-combine families remain unchanged.
Required fmt/strict/WASM and coordinator-authorized targeted runs only.

The no-skirt M73 first difference is transparently OPEN. It may follow from
changed source collection length/order, but that has NOT been proved. No M73,
estimator, parser, comparator or timing-specific repair is authorized by this
proposal. If source-faithful area/owner implementation leaves that or any other
whole-output divergence, STOP/report and request amendment or split before
changing another owner. No claim that repairing occupied hull alone is GREEN.

Deferred: perobject-skirt multi-instance admission (retain existing guard),
ByObject, combine1+copies expansion beyond current approved behavior,
inner/auto/ears/painted brim, overlapping/touching copies/other shapes, support,
raft/tower/multiple filament. No arbitrary fallback, defensive check, acceptance
guard expansion, coordinate snap, output normalization or invented diagnostic.
Native rejection/undefined input would be NO-RED STOP; preparation/compiler/
affinity/witness faults are NON-GATE, never parity evidence. FULL freeze remains:
no full/workspace, merge or push. This proposal does not alter the queued chain.
