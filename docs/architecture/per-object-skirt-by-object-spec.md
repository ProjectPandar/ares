# SPEC request: per-object skirt in ByObject sequence

Status: DOCS-ONLY SPEC-REVIEW-REQUEST; premise discovery completed under explicit
assignment. No public RED or production modification yet. Base main
`41f2c90f765d42c6620583121296bb61a2cb839c`; upstream pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
APA-R2 M NO-RED STOP remains accepted and separate.

## Source-owned boundary

Paths below are under `OrcaSlicer/src/libslic3r/`.

- `Print.cpp:2646::_make_skirt`, :2780-2977: per-object hulls include instance
  shifts and brim/support footprints, union-find grouping at nominal skirt reach,
  with per-group skirts. ByObject does not disable this grouping.
- `Print.cpp:2548-2557`: after `_make_skirt`, ByObject + perobject + shared
  per-object skirt throws a slicing error. Separated singleton groups are valid;
  do not remove this collision constraint or treat shared skirts as supported.
- `Print.cpp:1286-1303::validate` gates sequential clearance checks and rejects
  smooth timelapse/clumping detection for multi-instance sequential jobs;
  `Print.cpp:613::sequential_print_clearance_valid` owns clearance/height/order.
  Existing validation remains authoritative, not bypassed by skirt admission.
- `GCode.cpp:2730-2762`: model-order instance ordering, per-object ToolOrdering
  and initial extruder. :3280-3374: each instance's entire layer stack is passed
  to `process_layers`, with inter-instance origin, retract/hop, temperatures and
  cooling reset. :5127-5136 retains sequential ordering in per-filament dispatch.
- `GCode.cpp:2036` clears `m_skirt_group_done` once at export initialization;
  :5240 resizes it to print groups; :4465-4483::generate_object_skirt_group passes
  the group's own vector to `generate_skirt`, invoked before object content at
  :5334. The sequential object loop does NOT clear every group's history between
  objects. `skirt_done` is the reference to this group vector, not another global
  reset obligation. First encounter of a distinct group is empty; another group
  must not inherit the first group's Z history.
- `GCode.cpp:4286-4328::Skirt::make_skirt_loops_per_extruder_*`: first-layer empty
  history and later skirt-height/strict-Z advancement gates; :4395-4414 uses
  that history for flow height. Preserve these semantics, no reset-per-layer or
  blind re-emission policy.

Exact Rust destination: ONLY the `print.print_sequence != ByLayer` disjunct in
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`
is proposed for removal after approved public RED. Reuse existing generation,
`crates/ares-core/src/project_slice/gcode_emit/layers/object_skirt.rs`,
`layers/drive.rs` and per-group accounting unchanged if whole-output evidence
permits. Existing ByObject traversal is already exercised by
`crates/ares-cli/tests/orca_parity/runner/print_sequence_by_object.rs`.
No new sequencing pipeline or state abstraction. If consumer changes are needed,
stop for source-cited amendment before implementation.

Tower/multi-filament guard disjuncts belong to W5; support/raft belong to W2.
Do not remove, reorder or generalize those disjuncts. No brim expansion.

## Premise discovery: measured, not assumed

Runtime root `/tmp/ares-byobject-skirt-discovery` on clean production main plus
accepted docs-only M stop `7add1f3b5868f0e9a656bc98f3e2346aa511722a`.
Canonical `/home/indexyz/ares-coord/bin/parity-control-set.sh` archived/hashed;
all current reference/oracle paths extracted, CPU18 jobs2, separate official and
seeded roots/cache, Native first then same-input validated API and real CLI.

Source fixture `tests/parity/per-object-skirt/disjoint.3mf`: two distinct objects,
centers x80/x140, y110, height4, single filament, skirt2/height1, no_brim,
combine_brims0, support0/raft0/tower0. Geometry/ZIP non-config bytes unchanged.
Only print_sequence and skirt_type differ across these three inputs:

| Input | Official and seeded Native | Public Ares |
| --- | --- | --- |
| bylayer-control: ByLayer/perobject | exit0 | strict whole Native/API/CLI PASS; raw API=CLI |
| target: ByObject/perobject | exit0, complete G-code; total layers40 | API UnsupportedProjectFeature("skirt_type per-object"), CLI1/no output |
| combined-control: ByObject/combined | exit0 | accepted API/CLI, raw API=CLI, BUT whole-output mismatch |

Thus the assigned guard is genuinely reached by a Native-valid input; all other
perobject guard predicates are false. No production admission widening is needed
to manufacture the premise. Native target shows skirt and ordered object markers;
public tests must bind each object's first-layer skirt to its sequential stack,
not just assert effective print_sequence or total layer count.

**Independent control blocker:** combined-control first difference byte55670,
line1916, Native `G1 Z4.4 F9000` versus Ares `G1 Z4.4 F9000 ; `; next line Native
has `normal lift Z` where Ares has an empty description. Same in both modes.
This is not the perobject admission RED, not equivalent output and not repaired
or normalized here. Owner ruling or a separately approved source-owned comment
repair is needed before claiming this whole-output control passes. Preserve this
failure; do not suppress comments or silently select a favorable control.

## Planned public Native-oracle RED, after approval

Freeze input-only 3MFs and exact SHA/member manifest; no Native outputs in Git.
`OrcaRunner::from_env`, required Native, runtime Native FIRST. Target must assert
Native exit0, real two-object sequential output, own-group skirt placement and
strict whole-file Native equality from validated API and actual CLI, plus raw
API==CLI. Before GREEN this fails on actual UnsupportedProjectFeature, never an
inverted assertion that passes on rejection. Both controls use the same strict
whole-output target assertions; resolve the recorded unrelated failure first or
obtain explicit scope ruling, never weaken comparator/classification.

After independent RED approval only: minimal own-disjunct consumer change,
unchanged tests/inputs, both modes fresh caches/artifacts CPU18, same-input control
raw bytes RED→GREEN stable, existing ByObject and perobject-skirt regressions,
fmt/strict/WASM as allocated on exact clean hash. No full/workspace/merge/push.
Changed Rust <=400 LOC. Coordinator owns integration and broad gates.

## Failure inventory / included vs deferred

- Invalid clearance, shared skirts, Native rejection or missing output: stop and
  record exact owner/stage; no bypass or error-as-success.
- Existing combined-control comment mismatch: retain and refer; no comparator
  normalization, comment disabling or unrelated travel repair in this slice.
- Correct admission but duplicate/missing second-group skirt, inherited first
  group's Z/history, wrong coordinates or order: genuine source-localized
  downstream issue requiring amendment if one-disjunct removal is insufficient.
- Fixture/env/compile/timeout/affinity errors: NON-GATE, not RED. Archive exact
  input, command, binary, applied settings, outputs and descendant/thread audit.
- All targeted outputs already equal: NO-RED STOP; no accounting-only repair.

Included: two distinct, separated singleton skirt groups, one filament,
ByObject, no brim/support/raft/tower; preserve Native validation and output order.
Deferred: multiple copies of one PrintObject, shared-group collision admission,
extra skirt heights/minimum/draft-shield combinations, brim, support/raft,
tower/multi-filament, sorting/placement changes and independent comment repair.
The current emitter is a temporary compatibility shell for the cited Print/GCode
boundaries, not an Ares-owned pipeline. Existing architectural decisions stand;
no new ARD or architecture is proposed.
