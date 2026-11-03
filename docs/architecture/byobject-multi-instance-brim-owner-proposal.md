# SLICE-PROPOSAL: ByObject multi-instance brim owner collections

Status: **PROPOSAL ONLY / REVIEW-REQUEST**. Base exact clean
`0577970d4f8a5f34d26389949238eac9ffe35586` (approved ByLayer guard chain queued
stack25). Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
No production/test change committed; temporary guard probe restored byte-for-byte.

## Frozen accepted ByObject family and Native-first evidence

Evidence `${TMP_ROOT}/ares-byobject-owner-proposal-0577970d4/discovery-report.json`.
Derived from approved owner-collection cube fixture, retaining meshes/owners and
Z scale0.4; freeze ByObject, object2 copies at X50/X125,Y110,Z2, object3 at
X200,Y110,Z2. Centers are separated75 to avoid clearance/group-sharing blockers;
all settings/ZIP member hashes retained. One filament, outer_only brim8/gap0,
combine_brims0, skirt2/height1, no support/raft/tower.

Primary combined-brim input SHA256:
`5467219291d3d177824fc4b7677605eb855707962ee9fd5d773db96dc15d9922`.
Four related inputs freeze only named config differences: perobject skirt,
combined width0, perobject width0, combined skirt_loops0 with brim8. All five
Native runs succeed in BOTH official/seeded modes, with positive outer-wall
extrusion at ALL X50/125/200 placements. This is accepted-output divergence,
NOT NONE, Native rejection, or a witnessed Print.cpp SlicingError.

Baseline Native FIRST, current validated API and real CLI:

| Input | Native | Public Ares / first strict difference |
| --- | --- | --- |
| combined-brim | CLI0,2 brim blocks,1 skirt | accepts; byte3509/line123: Native next brim-independent skirt G1 X57.055 Y90.894 E.37042, Ares M73 P7 R12 (Native M73 follows that G1) |
| perobject-brim | CLI0,2 brim blocks,2 skirts | API UnsupportedProjectFeature("skirt_type per-object"), CLI1/no G-code |
| combined-no-brim | CLI0,0 brim,1 skirt | COMPLETE strict Native/API/CLI PASS |
| perobject-no-brim | CLI0,0 brim,2 skirts | COMPLETE strict Native/API/CLI PASS |
| no-skirt-brim | CLI0,2 brim,0 skirt | accepts; byte1607/line67: Native G1 X114.334 Y100.314 ; move to first brim point, Ares G1 X42.008 Y98.284 ; move to first brim point |

Each row reproduces identically in both modes; all accepted public cases have
raw API==CLI. The no-skirt first point is direct brim path/order evidence,
independent of the earlier M73 first differences. Complete comparisons remain
unmodified; never remove/reorder M73 or call this a passing timing diagnostic.

## Local uncommitted guard-only probe: NOT guard retirement alone

After Native acceptance was established, locally removed ONLY the remaining
ByObject && instance_indices.len()!=1 disjunct and its now-orphaned comment/binding.
No change to brim planning, sequencing, emitter, other guards or comparator.
Exact uncommitted patch `probe/guard-only.patch`, SHA256
`979bc7d7bd98fc4d416e4ea43bf792e1c0e407fdfef2e6aea2e992365a1199ec`.
This is DIRTY discovery on base0577970d4, NOT public clean-hash RED/GREEN proof.
Native FIRST again, separate fresh probe roots/caches and rebuilt API/CLI.

The perobject-brim input now slices, but FAILS complete comparison in BOTH modes:
byte1296/line48, Native `G92 E0`, Ares `M73 P5 R12` before that same G92.
Expected290145 / actual289362 normalized bytes. Native emits TWO brim OWNER
blocks and TWO skirt blocks; Ares emits THREE brim PER-COPY blocks and TWO
skirt blocks. Combined-brim and no-skirt-brim retain their exact original
strict differences and Native2/Ares3 brim blocks. Both width0 controls remain
whole-output PASS with API/CLI raw bytes stable baseline→probe.

Thus the remaining guard is not the only divergence. Removing it alone is
NOT GREEN. No estimator repair is proposed; M73 is an observed symptom with
unproved causality. A source-faithful owner port must pass the whole comparison,
or STOP/report rather than changing estimator, timing or normalization.

Source restored byte-for-byte and clean porcelain verified before this document;
restoration build resets reusable CLI target to clean source. Actual Native/
public descendant+thread masks CPU18, serial independent modes, exact clean base
and dirty-patch identity, canonical archive/hash/all reference settings recorded.
No Native outputs in Git; all output evidence stays in the scratch evidence root.

## Owning source and exact remaining Rust mismatch

- `OrcaSlicer/src/libslic3r/Print.cpp:2465-2467` uses model instance order for
  ByObject; :2520-2530 de-duplicates PrintObject IDs into objPrintVec. Object
  sequencing is per INSTANCE, but brim ownership is not per instance.
- `Brim.cpp:36-45,573-581` accumulates all instance-translated object areas under
  ONE PrintObject ID; :914-956 preserves complete owner bounds/footprints.
  `make_brim:958-977` disables GLOBAL combination for ByObject but STILL runs
  ordinary infill once per PrintObject area. There is no ByObject branch reverting
  the ordinary area map to per-copy templates. :813-886 owns infill/path ordering.
- `Print.cpp:2794-2816` expands model hulls per instance and consumes complete
  owner brim footprints. :2847-2920 merges perobject skirt groups and references
  ordinary owner collections. The separated frozen perobject input is accepted;
  :2551-2556 shared-group SlicingError is NOT reached or proven by this witness.
- `GCode.cpp:3230-3243` initializes outstanding brim OWNER IDs ONCE per print.
  :3280-3345 then loops sequential instances and sets each instance origin;
  :4486-4531 emits at origin0 and erases owner IDs; :5334-5335 places brim before
  model extrusion. Owner IDs are NOT reinitialized for each sequential copy.

Existing Rust boundary is
`crates/ares-core/src/project_slice/gcode_emit/brim.rs::BrimPlan::generate`:
`owner_collection = !combined::enabled(traversal) && print_sequence != ByObject`
explicitly selects the old local/per-copy compatibility shell for ByObject.
`layers/brim_emit.rs::emit` applies once-only `emitted_brim_owners` only when
owner_collection is true, so the same local template re-emits at each copy.
`brim/owner.rs`, `brim/bed_area.rs`, `brim/emit.rs`, `skirt/occupied.rs`, and
`footprint/first_layer.rs` already implement the instance-expanded single plate
frame for ByLayer ordinary owners, but that producer is never selected here.
`motion/state.rs` holds owner identity for the whole emission pass; preserve
that lifecycle across `layers/transition.rs` sequential origin changes.
`skirt/generate.rs` retains the separate explicit ByObject multiplicity guard.
These are source compatibility consumers, not a new Ares-owned pipeline.

## Recommended prerequisite split (approval requested, no implementation)

**A. ByObject ordinary owner-area/collection port FIRST.** Public Native-active
RED on accepted combined-brim (and no-skirt failing localizer), with same-input
combined width0 strict PASS control. Freeze input-only SHA/member manifests;
assert positive Native model/brim activity and full ordered Native equality for
all public cases, raw API==CLI. RED must genuinely fail, independent review
before production. Then extend the EXISTING source ordinary-owner preparation/
emitted-ID lifecycle to ByObject, preserving per-copy model order/interludes,
plate-frame area/bounds and all current ByLayer/global-combine behavior. No new
pipeline, copy-count geometry workaround, fallback, estimator or guard widening.
Perobject ByObject guard STAYS through this prerequisite.

**B. Perobject ByObject guard retirement only AFTER A passes.** Combined-brim
currently FAILS and cannot honestly be a passing prerequisite control for this
admission slice. After A GREEN, repeat the frozen perobject guard-lift discovery.
Only if complete comparison matches, propose Native-active public unsupported
RED + newly passing combined control, then minimal guard retirement. Do not
claim today's dirty probe is green or combine unrelated policy/diagnostics.

Both stages remain source-cited and separately reviewed. ByLayer owner/global
and brim/skirt/per_object families stay unchanged and must pass targeted gates,
CPU18/jobs2/threads2, exact clean hashes and fresh mode evidence, fmt/strict/WASM,
files <=400 LOC. Any residual whole-output divergence after a faithful owner
port means STOP/report and scope amendment, NOT estimator work. Deferred:
shared-group error parity, combine1+perobject, overlap/touching/new geometries,
other brim modes/holes, support/raft/tower/multi-filament. No assertion about
unreached SlicingError behavior. No full/workspace, merge or push during freeze.
