# Region-role tool emission implementation plan

Status: approved next independent milestone; do not implement in the current
single-tool Normal/Tree batch. Isolated `94d33b59` reproduces the baseline body
gap. The modifier input was previously rejected, not previously supported.
Spec: [contract](../specs/2026-10-03-region-role-tool-emission.md).
Architecture: [ARD-0094](../../architecture/ard-0094-region-role-tool-ordering-and-writer-state.md).

## Confirmed RED and diagnosis

The unchanged inherited-second-filament public baseline/modifier comparison
fails against both official and seeded Orca. Expanded final runs each executed
7 controls (6 PASS, 1 FAIL), then canceled 40 under fail-fast. These canceled
controls were subsequently rerun separately: official 46 PASS/225.131s,
seeded 46 PASS/266.085s, both exit 0. This is not a passing 47-test suite.

Logs: `/tmp/ares-normal-tree-merge-{official,seeded}.log` and
`/tmp/ares-normal-tree-merge-{official,seeded}-finite.log`.
Official artifact root: `/tmp/ares-normal-tree-merge-official-artifacts/`;
`case-Na2CR7` baseline and `case-NQMkDs` modifier preserve inputs, complete
outputs and manifests.

First normalized difference is byte 186/line 7: `filament: 1,2` versus
`filament: 2`. Baseline sizes are 182130/173811 bytes; modifier sizes
270578/226756. Native baseline printing switches occur at lines 232 (`T1`),
413 (`T0`), 535 (`T1`); Ares has no corresponding body transitions.
Native `total_toolchanges` is 16; Ares reports 0.

Effective-config inheritance is present. The missing chain is
`GCode.cpp:4971–5075` region/role bucketing, then per-tool traversal at 5241
and `set_extruder` at 5308. Rust object entries only pass region density;
motion does not consume role-filament assignments or maintain a schedule.
`gcode_emit/header.rs` also writes the used-filament count as an ID; that
exact header defect is present in base commit `94d33b59`. This source proof
does not substitute for a baseline runtime comparison of the body defect.

## Finite TDD slices

### Mainline regression attribution

Isolated worktree `/tmp/ares-main-tool-regression.AeeJvE` is detached at
`94d33b596809096928d0fe163e6e065183e2e7f2`. Its production files are unchanged;
only an artifact-replay test was added to the existing parity test crate.
The public CLI independently slices the identical baseline successfully,
but still emits `filament: 2`, `total_toolchanges: 0`, startup-only tool
instructions at lines 37/96/140/181, and no printing-body tool transitions.
This establishes the body gap on the old mainline, not merely the header bug.
Baseline CLI output SHA256:
`c8e7de5b29b950bbf1526cfa1032b560d4784bfb23b8d7f82cffecb5ecc668b0`.

The identical modifier input is rejected by that mainline with
`UnsupportedProjectFeature("sparse_infill_density")`. No old modifier G-code
exists, so do not claim an old modifier body comparison. This batch newly
accepts that regional configuration; its dual-tool consumer gap remains
explicitly unimplemented, not reclassified as parity.

Input SHA256, baseline:
`78eca1c5df43561a9bb9da3526040b1a15d12a488d00c5ddf86d5bfbda00629f`;
modifier:
`30a532e027f991b2674066d9ff853201b329425106e9f09c141a4bb140cdd0d3`.
CLI logs: `/tmp/ares-main-tool-regression-cli-{baseline,modifier}.log`.
Original-comparator replay finished in1.306s, exit100, with both inputs
evaluated before the unchanged PASS/PASS assertion:
baseline `DIVERGENT` (byte186/line7), modifier `ARES_ERROR` (unsupported density).
Log: `/tmp/ares-main-tool-regression-replay.log`; output artifacts are
`/tmp/ares-main-tool-regression-artifacts/case-L7GWAx` for baseline and
`case-8BVJ3T` for modifier.
These outcomes plus actual baseline body inspection establish the existing
gap; they do not claim the old modifier generated a comparable body.

### Implementation order

First finite public tracer: a single-tool modifier sets only `is_infill_first`
while the parent is wall-first. Source `GCode.cpp::extrude_perimeters` applies
each nonempty region configuration and selects its before/after-infill phase;
the first layer always prints walls first. Existing Rust order preparation
reads only the first model-part region flag and drops perimeter region IDs
despite `LayerFillEntities::perimeter_ranges` already retaining them. Public
test `orca_parity_modifier_infill_first_preserves_region_wall_phases` compares
both baseline and modifier complete files with unchanged comparison rules.
Provisional official RED exit100/2.342s: baseline PASS, modifier DIVERGENT, first difference
byte1296/line46 (`M73 P3 R16` versus `R17`), followed by body differences.
Native role-sequence witness differs; this is not just a config-header change.
Log `/tmp/ares-region-infill-first-red.log`; artifacts case-ueaqdU and
case-Om1XYs in `/tmp/ares-region-infill-first-red-artifacts/`.
This is a source-owned traversal prerequisite, not a multi-tool completion.

The old held worktree was temporarily stashed and checked out to94d33b59 by
the user's review process, then restored. Do not accept the provisional RED
from that overlap as authoritative; rebuild and rerun on
`codex/region-role-tool-emission` in `/tmp/ares-tool-ordering.jdZMyT`, with a
separate build target, before implementation. Remote main and the accepted
b86b73bd commit remain intact. Preserve the restored original-worktree edits.

Authoritative isolated rerun completed before production edits:
`/tmp/ares-region-infill-first-isolated-red.log` exit100/2.354s, baseline PASS,
modifier DIVERGENT at the same byte1296/line46. Both reference role-sequence
and unchanged complete-output assertions executed. Separate target
`/tmp/ares-tool-ordering-target` prevents reuse of overlap builds.
Implementation carries `perimeter_ranges` region IDs through island
assignment and ordered collections; motion emits the source four phases using
each region's flag. No geometry copies, header-only fix, tool guards or
unused ToolOrdering scaffolding. First-layer semantics are explicitly derived
from actual object/raft id and bottom_z, not the emitter's simplified id check.
Authoritative official GREEN `/tmp/ares-region-infill-first-isolated-green.log`
exit0:1 PASS/2.351s, both complete baseline and modifier comparisons executed.
The final build took6m49s because the process-wide replay affinity also reached
compilation; compiler-only affinity was expanded while actual test and Orca
replay stayed CPU0. This does not change production worker behavior.
Final official/seeded controls, workspace, clippy and WASM remain in progress;
do not commit the slice as closed before their actual terminal results.

Final frozen-source Clippy `/tmp/ares-region-infill-first-final-clippy.log`
exit0, core257 lib/273 lib-test diagnostics unchanged from the accepted
batch, no never-read/never-used/dead_code match. No added warning suppression.
This is not a `-D warnings` claim. Fmt/diff checks pass; modified functional
Rust files remain below400 LOC. Remaining validation is still live.

WASM check `/tmp/ares-region-infill-first-wasm.log` exit0 for ares-core and
ares-wasm, elapsed9m47s including build-lock queue. This is compilation, not
a fresh browser/native Windows/macOS runtime certification. Both activated
complete-output control runs select47 tests, exactly excluding the preserved
dual-tool RED; 282 other tests are outside these filters. Workspace selects7546
tests. Do not call any still-running control or workspace result PASS yet.

Workspace terminal `/tmp/ares-region-infill-first-workspace.log` exit0:
7546 PASS/36 slow/9 skipped in265.487s. The default env-gated parity returns
are not substituted for activated native controls: both47-control runs are
still live, and the new modifier tracer has already passed in seeded output
(14.335s). Original fixtures, complete comparators and the dual-tool RED are
unchanged. No current source edits remain after this frozen validation.

Seeded47 terminal exit0/349.876s. First official47 terminal exit100/305.864s,
46 PASS/1 FAIL: existing later-higher-density control failed its parent
baseline before reaching the modifier, at byte12097/line524, equal normalized
length138530. Native expected travel X113.388Y113.135; Ares X113.244Y113.297,
followed by swapped variable-width solid-fill paths. Keep this result; do not
report the initial suite all-green.

Diagnosis used the identical input SHA256
15811835a4ebaf38e3e391accc6a686d4255229021d476b48315677d9e3e41c2.
Saved failing case-BJ4QiK is in the official-controls artifact root. Both the
saved input and original producer path in orca-runner-NxZYui have that hash.
Four official CPU0 replays (two saved path, two original path) print the Ares
travel at line524, unlike the original reference. Replay1/2 are raw-identical,
SHA4e29ad6ba7b3bd108925120fc881a875406c6340a41c8fef8f568118646f438a;
replay3/4 are raw-identical too. Outputs/logs retained under
`/tmp/ares-region-infill-first-official-replay-{1,2,3,4}`.
This establishes observed producer variation, not its internal mechanism.

Untouched isolated b86b73bd production in
`/tmp/ares-region-phase-mainline.9Vk9pc` successfully slices that same input.
Its complete Ares output is byte-identical to the candidate after removing
only each Ares generator-time line (diagnostic old/new comparison, not a
relaxed native acceptance assertion). CLI output/log retained at
`/tmp/ares-region-phase-mainline-baseline.{gcode,log}`; source status is clean.
Thus this baseline difference is not introduced by the region-phase slice.
Unchanged full public test focused rerun exit0/2.338s:
`/tmp/ares-region-phase-official-focused.log`. No source, fixture or comparator
was modified during diagnosis. Fresh complete official47 terminal exit0:
47 PASS/282 outside-filter skips in114.018s, recorded at
`/tmp/ares-region-infill-first-official-final.log`. All validation processes
have terminated and their handles have been collected. This closes the
region-phase prerequisite validation, not the deferred dual-tool milestone.
The initial official failure and its retained diagnosis remain part of this
record; the complete-output assertions were never relaxed.

1. Preserve canonical region ownership through ordered extrusion collections.
   Source: `GCode::process_layer` region/island buckets and split wall roles;
   destination: `island_print_order` and layer-entry adapters. Add behavior
   coverage through the existing public role/region configuration seam before
   replacing adapters. No private-source pinning or fixture proliferation.
2. Port actual layer tool selection/order. Source: `LayerTools::extruder`,
   `ToolOrdering::initialize_layers`, `collect_extruders`,
   `handle_dontcare_extruder`, `reorder_extruders_for_minimum_flush_volume`,
   `collect_extruder_statistics`, `mark_skirt_layers`. Destination:
   `gcode_emit/tool_ordering/` consumed by `layers`. Role witnesses must cover
   wall versus bottom/internal solid and newly active sparse infill. Do not
   replace upstream ordering with a sorted-ID heuristic.
3. Port writer/per-filament state and transitions. Source: `Extruder`,
   `GCodeWriter::toolchange` / `reset_e`, `GCode::set_extruder`; destination:
   `motion` state and a source-owned transition module. Exercise switch-back,
   retraction, E reset/conversion, offsets and template counters through the
   public baseline. Preserve global machine state and selected filament state
   separately. Reuse existing template and motion consumers where source-correct.
4. Connect tool-outer traversal and derived metadata/statistics. Source:
   `GCode::process_layer` and ToolOrdering statistics; destination: `layers`,
   `header` and existing final-output consumers. Run the existing complete
   baseline/modifier comparator against official and seeded until both GREEN.
   Partial metadata/body witnesses are diagnostic only, not this exit gate.
5. Run the full 47 activated controls with `--no-fail-fast`, then workspace,
   lint/format, WASM/browser and cross-target validation; request independent
   six-axis review of a frozen source snapshot before commit/push.

Each slice starts from a public behavioral RED and ends in a consumed runtime
path. Commit verified independently closed changes with their docs. If the
tool schedule/state/traversal dependencies cannot form a passing independent
slice, keep them together rather than committing unused modules or broken
intermediate runtime. Keep functional Rust files below 400 LOC.

## Reference reproduction and scope

Use the existing Orca parity runner and approved whole-process CPU-0 replay,
with `ARES_PARITY_REGION_FILAMENT_REFERENCE=1`; retain the actual official and
seeded binaries and artifact manifests. No new reference modifications.
Do not turn the confirmed failure into an ignored test or edit its assertion.

The user approved this separate milestone after single-tool closure. Isolated
`94d33b59` reproduces the baseline body defect; the modifier is separately
recorded as a newly accepted input that still lacks multi-tool output parity.
No further painting edge combinations are a current merge requirement.
