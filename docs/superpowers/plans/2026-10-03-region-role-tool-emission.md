# Region-role tool emission implementation plan

Status: in progress as a separate milestone after the single-tool Normal/Tree
merge. Isolated `94d33b59` reproduces the baseline body gap. The modifier input
was previously rejected, not previously supported.
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

### Dual-tool implementation resumed after prerequisite push

The region-phase prerequisite is pushed as `031b3304d41eb734c71a42395c208372374d7600`
on `codex/region-role-tool-emission`. Production work continues only in the
isolated `/tmp/ares-tool-ordering.jdZMyT`; the original restored worktree is
untouched. Fresh official public baseline/modifier RED before further edits:
`/tmp/ares-dual-tool-current-red.log`, exit100/2.452s. Both complete comparisons
are DIVERGENT at byte186/line7; artifacts are `case-9OeTWF` and `case-5HaKnL`
in `/tmp/ares-dual-tool-current-red-artifacts`. No fixture/assertion changed.

Source audit adds the invoked scheduling dependencies to spec/ARD:
`ToolOrderUtils.cpp:385-790` order solvers and grouped layer sequence assembly,
`ToolOrdering.cpp:498-580` area-based initial wall order and configured first
layer order, and `ParameterUtils.cpp:6-29` configured layer-range decoder.
`Print.cpp:2429-2430` / `3399-3400` builds and sorts the ordering before G-code
consumption. Keep collection's one-based/don't-care IDs separate from final
zero-based logical filament order and physical-hotend mapping.

Current first implementation step separates the seven existing active
extrusion counters into `motion/extruder_state.rs`, updating their real motion,
template and accounting consumers without changing arithmetic or output.
Writer XY/Z, wipe paths, lift and motion caches remain global. This active
frame is not yet a logical-filament registry: SEMM's physical shared E and
retraction slots must not be saved/restored as if they were private filament
state. The following registry/selection port must preserve that distinction.
Do not claim this frame alone implements a transition or closes the dual-tool
RED. It remains uncommitted with the coupled schedule/traversal/writer work
until a validated independent closure exists; no documentation-only commit.

Active-frame `cargo check -p ares-core --all-targets` terminal exit0/54.57s:
`/tmp/ares-dual-tool-extruder-frame-check.log`. Fmt/diff checks pass. Focused
G-code nextest and workspace Clippy are still running; no runtime PASS claim
yet. Interim independent six-axis review found no mechanical migration defect
and explicitly did not approve an independent commit or registry completion.
Processor/fan-mover E state is separate and was intentionally not migrated.
Remaining implementation must consume tool-specific conversion/configuration,
switch-back state and all-filament final statistics, not just the active frame.

Focused existing G-code regressions terminal exit0:272 PASS/6827 outside-filter
skips in52.727s, `/tmp/ares-dual-tool-extruder-frame-tests.log`. This includes
the public spiral-vase and multi-object identity regressions; it is not an
activated native parity sweep or a workspace/all-printer acceptance claim.

Workspace/all-targets/all-features Clippy terminal exit0/4m08s:
`/tmp/ares-dual-tool-extruder-frame-clippy.log`, unchanged core257 lib/273
lib-test diagnostics, no never-read/never-used/dead_code match. No suppression
added and no `-D warnings` claim. Public parity binaries are rebuilt without
the runtime CPU0 restriction before executing the approved single-CPU replay.

Fresh official public replay `/tmp/ares-dual-tool-extruder-frame-public.log`
terminal exit100/4.885s: single-tool infill-first complete baseline/modifier
PASS (2.414s), unchanged dual-tool complete baseline/modifier DIVERGENT
(2.469s) at the same byte186/line7, with unchanged normalized lengths.
Cases `case-XWvAc0` and `case-3yhYUQ` preserve the dual-tool results in
`/tmp/ares-dual-tool-extruder-frame-public-artifacts`. Their input hashes match
the pre-edit baseline/modifier, and both complete raw Ares files compare
byte-identical with the pre-edit cases (two `cmp` exit0, no normalization).
This proves the active-counter move preserved those existing outputs, not that
the missing transitions are implemented. All started build/check/test process
handles have been collected at terminal. The coupled dual-tool work remains
active and uncommitted; next implementation is the actual private/shared
registry and schedule-driven transition/traversal consumers.

Use the existing Orca parity runner and approved whole-process CPU-0 replay,
with `ARES_PARITY_REGION_FILAMENT_REFERENCE=1`; retain the actual official and
seeded binaries and artifact manifests. No new reference modifications.
Do not turn the confirmed failure into an ignored test or edit its assertion.

The user approved this separate milestone after single-tool closure. Isolated
`94d33b59` reproduces the baseline body defect; the modifier is separately
recorded as a newly accepted input that still lacks multi-tool output parity.
No further painting edge combinations are a current merge requirement.

## Extruder registry and template consumer slice (uncommitted)

Source boundary: `Extruder.cpp/.hpp` private counters and SEMM physical axes,
`GCodeWriter::set_extruders`, and `GCode.cpp::PlaceholderParserIntegration`
(`1625–1713,3908–3914`). Destinations: `motion/extruder_registry.rs`,
`motion/extruder_state.rs`, and `motion/writer_state.rs`; the latter owns live
template refresh/write-back previously embedded in `path/role_change.rs`.
No geometry copies, public tool API, reference modification or relaxed
complete-output comparison. Mode comes from actual motion configuration, not
a separately cached Boolean. Missing filament-map entries use physical0,
matching `Extruder::extruder_id`, not the first map entry; positive/in-range
entries are already checked at the effective-config boundary.

First attempted test build failed because `GCodeOptions` has no `Default`;
this is not accepted as RED. Corrected tests project the four typed source
views. `/tmp/ares-extruder-registry-red-retry.log` terminal exit100,3 FAIL in
0.026s: inactive position/retraction/extra were discarded, inactive negative
retraction returned Ok, and statistics had length1 instead of3. After wiring
all registered IDs, `/tmp/ares-extruder-registry-green-controls.log` exit0,
275 PASS in53.222s, including spiral-vase and multi-object regressions.

Intermediate Clippy `/tmp/ares-extruder-registry-clippy.log` exit0/4m00s but
259 lib/275 lib-test diagnostics, two above the accepted257/273. This is not
an acceptable final quality result. Both new argument-count diagnostics are
addressed using the semantic retraction/restart pair and area/density pair;
the moved vector validator now returns its error directly, without an
additional error-output parameter or unreachable fallback error.

Independent review found the empty-registry and relative-E output-variable
branches. `/tmp/ares-extruder-registry-output-contract-red.log` exit100,
9 tests:6 PASS/3 FAIL in0.032s, proving empty registries incorrectly published
and validated E output variables and relative-E incorrectly validated absolute
position. Passing controls exercise partial write-back on a later-tool error,
zero-filled unused IDs, SEMM same/different physical-axis switch-back and the
missing-map branch. Those three branch failures were then corrected; SEMM
coverage additionally writes an inactive filament's private template counters
and confirms its shared physical axis remains unchanged on activation.

Frozen final controls, Clippy and public parity rebuild are running; collect
their terminal outcomes before calling this slice validated. Do not call the
intermediate checks complete dual-tool parity or approve a commit from them.

Final consumer controls `/tmp/ares-extruder-registry-final-controls.log`
terminal exit0:281 PASS/53.537s,6827 outside the selected filter. All nine
registry tests, spiral-vase and multi-object controls passed. Final Clippy
`/tmp/ares-extruder-registry-final-clippy.log` terminal exit0:256 lib/272
lib-test diagnostics (one below the accepted257/273), no dead-code/never-used/
never-read diagnostic and no added suppression. `cargo fmt --all --check`
and diff check exit0. These are consumer-stage checks, not workspace or full
dual-tool closure. Public parity rebuild and WASM check are still pending.

Public parity rebuild `/tmp/ares-extruder-registry-final-parity-build.log`
terminal exit0 before CPU0 replay. WASM check for core/browser adapter
`/tmp/ares-extruder-registry-final-wasm.log` terminal exit0/3m52s including
build-lock queue; compilation only, not a fresh browser runtime assertion.
Official public replay `/tmp/ares-extruder-registry-final-public.log`
terminal exit100/4.816s: single-tool infill-first baseline/modifier complete
output PASS (2.347s); dual-tool baseline/modifier remain DIVERGENT (2.467s),
both first differences unchanged at byte186/line7 with the original lengths.
Cases `case-FDqLpd`/`case-6rP14N` are retained under
`/tmp/ares-extruder-registry-final-public-artifacts`. No fixture or comparator
was changed. All process handles started for this consumer stage reached
terminal; no pending build or test is being reported as successful.
Both input SHA256 values match the pre-edit baseline/modifier recorded above;
both complete raw Ares outputs also `cmp` byte-identical against pre-edit
`case-9OeTWF`/`case-5HaKnL` (two exit0, no normalization). This stage changes
the internal template consumer contract and adds behavioral coverage; it does
not implement the missing printing-body transitions.

Independent read-only six-axis revalidation closed this consumer-stage repair
list after checking the terminal evidence. It explicitly does not approve an
independent commit, complete dual-tool parity or the global printer/option
objective. Keep the coupled implementation uncommitted until the remaining
source-owned producer/transition consumers below are closed and reviewed.

Still coupled and required before the dual-tool closure: actual writer-used
tool collection including support/interface, actual initial tool, all-tool
machine-start write-back, per-filament conversion/configuration installation,
source ToolOrdering/ToolOrderUtils schedule,
per-tool object/island/region buckets, complete `set_extruder` transitions and
all-tool final statistics. Current object-only registry initialization and
initial0 are temporary compatibility wiring, not a source-complete schedule.
The unchanged inherited-filament baseline/modifier whole-file assertion must
become GREEN against both references before that coupled implementation is
committed. Fresh workspace/platform checks and six-axis review remain gates.

### Relative-E axis ownership continuation

The existing unchanged public dual-tool RED remains the tracer for this
coupled milestone; no private implementation test seam is added.
`Extruder.cpp::extrude` (30–49) resets only the current E axis before adding
each relative delta, retaining signed absolute usage. `Extruder::retract`
(58–85) also resets that axis when the remaining retraction is zero.
The Rust destinations are `motion/extrusion.rs::coordinate` and
`motion/travel/retract_core.rs::plain_retract`; firmware retraction already
calls `coordinate` even for zero remaining length. The active frame represents
the shared physical axis under SEMM and the private axis otherwise, so this
does not reset inactive-tool private counters or statistics. Relative-axis
resetting does not reset signed absolute usage; complete-output assertions
remain unchanged.
Independent review identified the layer-start duplicate retraction path and
the wipe pre-call zero-delta branch. Layer start now consumes the existing
`GCodeWriter::_retract` compatibility shell `plain_retract`, including its
firmware fake-length counters, rather than directly assigning retraction.
`GCode.cpp::retract` (7653–7683) owns whether the pre-wipe and trailing calls
actually happen: zero-delta calls still reset relative E; a suppressed tail
with no wipe makes neither call. A nonsuppressed firmware tail still calls
retract after wiping. Do not put this behavior into `reset_e`, which has a
distinct relative-E no-op contract.
The before-wipe call installs proportional restart extra; only an actual
trailing call reinstalls full restart extra. No-wipe suppressed-tail calls
preserve the previous extra. Firmware retraction consumes the caller's extra
instead of replacing it, and layer-start firmware now updates signed usage
through the existing fake-length counter path rather than assigning only
the retraction amount. Independent static six-axis re-review closed these
repair findings; terminal runtime evidence is still pending.
Initial controls/Clippy/parity-build logs are intermediate because these
review repairs were made while compilation was running; final verification
must rebuild the frozen repaired source and use distinct log names.
This is a source-owned counter prerequisite, not a complete ToolOrdering
producer or successful dual-tool output claim. Controls and native replay
must reach terminal before recording validation outcomes.

Terminal continuation evidence (frozen repaired source):

- `/tmp/ares-relative-e-axis-final-controls.log` exit0:127 PASS/39.130s,
  6981 outside the filter. Includes writer registry, motion, spiral-vase
  and multi-object controls; not a fresh full-workspace run.
- `/tmp/ares-relative-e-axis-final-clippy.log` exit0:256 core lib/272
  lib-test diagnostics, below the accepted257/273 baseline, no dead-code/
  never-used/never-read match or added suppression. Elapsed6m11s includes
  build-lock queue. Fmt and diff checks exit0; touched functional modules
  are below400 LOC.
- `/tmp/ares-relative-e-axis-final-wasm.log` exit0 for core and browser
  adapter wasm32 compilation,5m25s including lock queue. No fresh browser,
  Windows or macOS runtime claim.
- Unrestricted final parity build exit0 preceded CPU0 runtime. Official
  `/tmp/ares-relative-e-axis-final-official.log` exit100/9.718s and seeded
  `/tmp/ares-relative-e-axis-final-seeded.log` exit100/11.361s each ran
  2 tests:single-tool infill-first complete baseline/modifier PASS; dual-tool
  complete baseline/modifier DIVERGENT, unchanged byte186/line7 and lengths.
  No fixture, comparator, guard or ignored-test change.

Official dual cases `case-PH5pPi`/`case-QSabZJ` under
`/tmp/ares-relative-e-axis-official-artifacts`, seeded
`case-IuzM3Q`/`case-su0zss` under the corresponding seeded root, retain whole
inputs and outputs. Both input hashes match the recorded baseline/modifier.
Four raw `cmp` checks exit0:official Ares outputs versus the preceding
registry stage, then seeded versus official. No normalization was used.
The internal counter prerequisite therefore does not repair the missing
printing-body transitions or convert the unchanged dual-tool RED to GREEN.

Preserve intermediate evidence:initial controls exit0/127 PASS/38.303s;
initial Clippy exit101 exposed too-narrow `plain_retract` visibility
(E0364/E0603). Its visibility was corrected to the owning `motion` module,
not public API, before final checks. Initial parity build also exited0, but
only the final repaired checks above are accepted as continuation evidence.
Every started process handle reached terminal. No independent coupled commit
or global printer/option completion is claimed.
Independent six-axis revalidation checked the terminal evidence and closed
this relative-E prerequisite repair list. It approved neither an independent
coupled commit nor milestone/global completion. Next required implementation
is the actual ToolOrdering producer and initial-tool consumers, followed by
per-tool buckets/transitions and unchanged complete dual-tool GREEN.

### Actual layer-tool producer

Continue the same public dual-tool RED, without changing its acceptance seam.
Port `ToolOrdering.cpp::collect_extruders` (661–815),
`generate_first_layer_tool_order` (517–582), `handle_dontcare_extruder`
(188–315), `tools_for_layer` and the used-ID union from
`collect_extruder_statistics` (921–953) to `gcode_emit/tool_ordering/`.
Build `initialize_layers` from original physical/support/raft Z values,
independently of the G-code schedule's already-paired Z values. Writer registration and empty-layer
emission consume this result immediately; delete the old object-config-only
registration and geometry-presence tool predicate.

Capture per-region extrusion-role presence before island flattening, borrowing
`LayerFillEntities` without copying paths. Upstream collection reads the first
entity of each fill collection, not every later path. Thin fills count through
the collections appended by `Fill.cpp:1361–1369`. Carry only that source-owned
role annotation through existing island/order compatibility shells. Raft
adapter roles enter the support collector, not object sparse-infill selection.

Included now: materialized per-region walls and four fill-role classes,
support/interface selectors and current-tool inheritance, source nonsoluble
support candidate selection, area-based first-layer preference, configured
first-layer sequence and one-based/don't-care normalization, actual used IDs.
Deferred within the still-open coupled milestone: ToolOrderUtils flush-cost
reordering/custom later-layer assembly, initial-tool runtime installation,
per-tool buckets and transitions. Adjacent per-copy purge overrides and
custom colour-change tool overrides remain the spec's separate scope; this
stage must not claim their parity. Sorted collected IDs are not a final
printing schedule, and the producer stage alone cannot be committed as
complete dual-tool output.

Independent review rejected the initial paired-Z reuse:original
1,1.00009,1.00013 can form two ToolOrdering groups but one twice-averaged
G-code group. The producer now directly traverses each object's original
layers and queries tools by their original Z, also eliminating repeated
whole-merged-list scans per object. Source parentheses
`-0.2 * (width / scale)` and `sqrt(matrix_len) + EPSILON` are preserved.
Initial checks/control/Clippy/parity builds remain intermediate because these
repairs were applied during compilation. Initial Clippy reported263 core
lib diagnostics (seven new nesting diagnostics); source-owned target-layer,
area-collection, first-layer preference and nonsoluble-candidate functions
remove the new nesting instead of suppressing it. Final frozen-source checks
must establish the warning count and runtime outcomes anew.
The next Clippy snapshot removed all seven new nesting warnings but exposed
one new raft tuple `type_complexity` diagnostic (257 lib/273 test). Reuse the
existing `RaftSchedule` by reference rather than inventing a wrapper or
suppressing it; widen only its `gcode_emit`-local consumer visibility.
Source `Config.hpp:1284–1285` additionally fixes percent width resolution to
`nozzle * value / 100`, not a rearranged product. Independent static review
closed those repairs. Only `repaired-*` frozen runs are final continuation
evidence; earlier `final-*` names are intermediate, not acceptance results.
Initial `check` exited101 (import and distinct classic/public role types);
corrected direct import and exhaustive role conversion `check-retry` exited0
in17.08s. These compile outcomes are not the public RED/GREEN result.

Frozen producer continuation: `repaired-clippy` exited0 (256 core lib /
272 core test diagnostics, no dead-code or new type-complexity diagnostic);
`repaired-controls` exited0,303/303 PASS in50.869s,6805 outside the filter,
including the unchanged spiral-vase Z-ramp regression. `repaired-wasm`
exited0 for core and adapter compilation only; no browser-runtime claim.
Unrestricted `repaired-parity-build` exited0 before either CPU0 reference
runtime started. Fresh rustfmt and diff checks exited0.
The original-worktree temporary checkout around12:20 is not evidence for
this isolated worktree; rerun public `infill_first` rather than relying on
that window. Independent review closes static/quality prerequisites and
the finite303-test evidence, not actual-results while reference runs remain
live, nor the coupled milestone or an independent commit.

Reference terminal results: official48 tests46 PASS/2 FAIL in236.074s;
seeded48 tests47 PASS/1 FAIL in278.491s (both exit100,281 outside filter).
Fresh `infill_first` PASS4.743s/6.480s. Dual baseline/modifier remain RED at
byte186,line7 with unchanged lengths; official casesrnUFxu/Y61wld and seeded
Ht8w9N/pjVYLZ. Their input SHA256s remain78eca1c5…/30a532e0…, and both
official RAW Ares outputs equal previous relative-E outputs by `cmp` exit0;
seeded RAW Ares outputs equal the official pair by `cmp` exit0.

The extra official Strong FAIL is preserved, not relabelled PASS:
case-qS6g9O,byte31174,line1218 (402612 expected/actual), initial native travel
X113.388 Y113.135 versus Ares X113.244 Y113.297. Seeded Strong passed,
case-1jnpDm; its RAW Ares output equals official RAW Ares (`cmp` exit0).
Diagnosis uses the unchanged whole-output seam; no geometry or assertion
change. Three direct unmodified official CPU0 runs of the *same saved input*
(SHA256 d86d29e64e19f6dc0541ae4a8b3f3c8f1a6252dd1e051960a0a52ff9fcf85097)
exit0, directories `/tmp/ares-strong-official-replay.uiqxXI`, `.fnEkMC`,
`.EKQrtE`. The first replay starts at X113.244 Y113.297 instead: original
versus replay first non-generator difference remains line1218. Therefore
single-CPU alone does not eliminate all official reference nondeterminism.
All three full streams passed the existing generator-only ordered-byte
comparator: `/tmp/ares-strong-strict-replay-artifacts/replay-summary.json`
inventory3/compared3/passed3, sweep exit0 in0.963s. Symlink inventory
`/tmp/ares-strong-strict-replay.MLBDVt` points each case at that identical
saved3MF and its respective native output. This is cached-stream evidence,
not three additional native test invocations. The original public Strong
focused rerun also PASS1.956s (summary1.957s). Initial failure remains in
the official48-test result; neither this finite evidence nor seeded47 PASS
closes the dual-tool milestone or the all-printer/option goal.
Independent six-axis revalidation independently verified all three identical
input SHA256s, unchanged comparator/replay source,3/3 manifests PASS and
focused public PASS, and closed this finite producer repair list with no new
must-fix. Reference provenance in cached replay manifests remains unknown;
do not infer the internal cause of official variation. Dual-tool RED and
remaining consumers stay open; no independent commit or milestone approval.

### Source flush-cost ordering

Continue the unchanged public inherited-second-filament RED (last frozen
official/seeded results above). Port `ToolOrderUtils.cpp:381–778`
greedy/Hamilton/one-layer forecast and two-group sequence assembly, with
`ToolOrdering.cpp:1235–1397` matrix selection/multipliers and custom sequence
precedence, `ParameterUtils.cpp:6–29` flattened layer-range interpretation.
Destination: `gcode_emit/tool_ordering/reorder/`, immediately consumed by
the materialized `ToolOrdering` after don't-care normalization. Preserve
source f32 costs, greedy integer truncation, strict DP/permutation ties,
128-bit source cache keys, earliest custom layer's group parity and reverse
range precedence. Non-BBL ordering uses one group even with physical
multiple nozzles. Included: output-affecting sequence ordering. Deferred
within this coupled milestone: initial tool, buckets/transitions/statistics.
Adjacent `FilamentGroup` automatic dual-BBL map optimization and tower
partition insertion retain the spec's explicit separate boundary; do not
claim their output parity. No new private test seam or weaker comparator.

Independent source review found no new algorithm must-fix within the valid
finite matrix/tool-ID domain. Initial check exited0; first Clippy snapshot
exited0 but reported261 lib diagnostics (five new nesting/int-plus-one
warnings), so it is not accepted. Move per-nozzle matrix construction to its
source-owned function and preserve C++ do/while pruning through explicit
permutation advance before continue. Compute the signed layer number before
range comparison. Re-review confirmed unchanged operation/permutation order.
Use source-width i32 predecessors in the flat DP cache, not wide Option
entries. Source greedy must select an unvisited candidate; no default-index
fallback. Only frozen `repaired-*` runs count as final evidence:
repaired Clippy exited0,256 lib/272 test, no new reorder/dead-code/type-
complexity diagnostic. Earlier `final-*` controls303 PASS53.786s and builds
exit0 remain intermediate because repairs overlapped their compilation.
Frozen rustfmt and diff checks exited0. Repaired WASM core/adapter compilation
exited0 (8m03s including queue; not browser-runtime evidence). Repaired
unrestricted parity build exited0 before CPU0 official/seeded48-test runs.
Repaired controls exited0,303/303 PASS52.393s,6805 outside the filter;
unchanged spiral-vase Z-ramp PASS9.552s. Independent review verified these
terminal quality/control/compile results, not reference results still live.

During official reference execution, painted manual Slim failed
case-0Y90Mn,byte40689,line1577 (516460 expected/actual bytes), native
X113.388 Y113.135 versus Ares X113.244 Y113.297. Preserve the first FAIL.
Saved input SHA2564be0ccab9b52a43b6fc56072610e9daaeeca67a94f7b77c4e16b3c7e8d876e1b
and RAW Ares output both equal the preceding producer case-nsDowv by
`cmp` exit0; RAW Ares also equals seeded case-WV0uGR (`cmp` exit0).
Three direct CPU0 unmodified official runs of this saved input exit0,
directories `/tmp/ares-reorder-manual-replay.VZw19r`, `.FlzRqC`, `.9OzJKK`.
Original versus first replay's first non-generator difference is line1577:
the two solid-fill paths swap order. No internal cause is inferred.
The unchanged complete comparator accepted all three streams, report
`/tmp/ares-reorder-manual-strict-artifacts/replay-summary.json`: inventory3,
compared3,passed3; sweep exit0 in1.117s. The input symlinks in
`/tmp/ares-reorder-manual-strict.vmd7qj` each target the identical saved3MF;
each reference links its respective native stream. Replay manifests retain
unknown producer provenance, so this is cached-stream comparison evidence.
Original public focused rerun exit0,1PASS8.298s (test8.296s). Neither focused
PASS nor three replays replace the initial official whole-batch failure.
Both48-test handles reached terminal exit100: official46PASS/2FAIL240.399s,
seeded47PASS/1FAIL283.423s,281 outside the filter. `infill_first`
PASS4.782s/6.422s. The dual baseline/modifier remain byte186,line7 RED with
unchanged lengths. Official cases5kmsT6/4wBrWO, seeded4oZg5Q/USV5Ac retain
the earlier input SHA256 pair78eca1c5…/30a532e0…; all four RAW Ares outputs
equal the preceding producer-stage counterparts by `cmp` exit0. No fixture,
comparator or scope gate changed. All started build/runtime handles reached
terminal; references were compiled unrestricted before CPU0 execution.
This stage adds source ordering production, not complete dual-tool emission.
Independent six-axis revalidation closed this finite ordering repair list
with no new must-fix after independently checking the saved-input hashes,
three unchanged-comparator replay manifests and retained unknown provenance.
The selected public tracer uses two logical filaments; these runs are not
public runtime coverage of the >5 Hamilton or >20 greedy branches, automatic
dual-BBL grouping, or the all-printer/option inventory. Static source audit
and successful compilation do not establish that broader output parity.
No independent coupled commit or milestone completion is approved. Next:
source initial-tool installation and all-tool startup writer/template state,
then per-tool buckets/transitions and statistics, unchanged full dual GREEN.

### Registered writer state before machine-start

Source `GCode.cpp:2791,2829,3118,3895–3933` registers actual ToolOrdering IDs
and initializes `PlaceholderParserIntegration` before the machine-start
render. Replace the adapter's late writer initialization and single-slot
`StartOutcome`/`StartRetractState` with that source ordering. Destination:
`gcode_emit.rs` writer initialization, `machine::append_start` consuming
the live `motion::EmitState`, existing `motion/writer_state.rs` refresh and
write-back. Machine-start uses every registered private counter, the source
relative-E/vector contracts, and ordered partial write-back; remove duplicated
slot0 state application and obsolete start-position plumbing through layers.
The unchanged public dual3MF complete-output RED remains the tracer; no new
private seam or weakened acceptance. Included here: startup writer ownership
and all-tool counter/template consumer. Initial tool/temperature/role options,
first/non-support placeholders and body buckets/transitions remain the next
coupled source slices, not silently claimed finished. Empty machine-start
retains its existing no-output path; no runtime guard or fallback is added.

The first check/control compilations exited101 because start_config still
imported the deleted parent value alias. Import the existing value module
directly; do not restore an unused compatibility alias. Those runs contain
no runtime evidence. Freeze after this repair and rustfmt. Final fmt/diff
checks exited0; final Clippy exited0,254 lib/270 test diagnostics, with no
dead-code/never-used/never-read diagnostics. Final WASM core/adapter check
exited0 (compilation only). Final unrestricted parity build exited0 before
CPU0 references. Final controls exited0,303/303 PASS53.433s,6805 outside
the filter. Independent review verified terminal quality/build evidence;
official/seeded48-test complete-output checks remain pending. No initial-tool,
body-emission, commit or milestone approval is implied.

Both reference handles subsequently ended exit100: official46 PASS/2 FAIL
236.818s, seeded47 PASS/1 FAIL282.280s,281 outside the filter. Preserve these
whole-batch results. Infill-first passed4.757s/6.420s. Unchanged dual baseline
and modifier remain byte186,line7 RED with the same182130/173811 and
270578/226756 normalized lengths. Official cases-iTrs8R/wxcHsS and seeded
cases-SEwCRT/wmulHL retain the78eca1c5…/30a532e0… input SHA256 pair. All four
RAW Ares outputs equal their preceding reorder-stage streams (each cmp0).

Additional official nested painted Slim failure case-LHmsQ1: byte13170,
line495,449764 expected/actual normalized bytes; native M73 P4 R24 precedes
G1 Z.2 rather than following it. Saved input SHA256
02de532e5b68744fc389e42817dcefb0b376f0dcd207b96fbf045c73e6a558ec and
RAW Ares both equal previous reorder case-pphKjQ; RAW Ares also equals
seeded passing case-NfGgf0 (cmp0). Three direct single-CPU official runs of
this identical saved input exited0: /tmp/ares-startup-nested-replay.7jbqsT,
.sUJI5s,.Cr6jYY. Original versus first replay's first non-generator
difference is line495, the M73/Z ordering above; subsequent solid-fill
path ordering also differs. No internal cause is inferred. Original complete
ordered-byte comparator accepted all three replay streams; report
/tmp/ares-startup-nested-strict-artifacts/replay-summary.json records
inventory3,compared3,passed3. The replay input links in
/tmp/ares-startup-nested-strict.nvt5fJ all target the saved input; references
link each respective native stream. Unknown cached producer provenance is
retained. Public focused rerun also passed; neither replaces the initial
official whole-batch FAIL. No fixture, comparator, coverage guard or private
test seam changed. All started build/runtime handles reached terminal.
Initial-tool installation, first/non-support placeholders, per-tool motion
options, body buckets/transitions and all-tool statistics remain open; this
is finite startup-state verification, not coupled milestone completion.
Independent six-axis revalidation closed this finite startup ownership repair
list with no new must-fix after checking the terminal logs, saved-input hashes
and three strict full-stream comparisons. It does not approve the coupled
commit or milestone; the remaining source consumers above are still required.

### Ordered startup filament metadata

Use the unchanged dual3MF RED from the preceding frozen official/seeded runs.
Port ToolOrdering.cpp:1000–1059 cal_non_support_filaments and GCode.cpp:
2776–2806,2834–2857 initial/first/non-support metadata into the existing
machine/first_filaments module, replacing configuration-ID scanning with
actual ordered layer tools. Expose a borrowed iterator from ToolOrdering;
do not clone geometry or create a second schedule. Store the resolved
metadata once in EmitState and insert it into every existing live template
refresh, preserving physical remap and source early termination. Initial
non-support falls back to initial printing filament only as upstream does.
Included: metadata production and its machine/layer/role/end template
consumers. Deferred within this same coupled milestone: physical initial
tool installation, tool-indexed motion/temperature options, body transitions
and final statistics. This does not substitute a header-only repair for
complete output. Keep existing mapping behavioral tests; add no private seam.
Independent static review confirms ordered first-per-slot scanning, source
early exit, all-support/no-map behavior and Integer placeholder types. The
Type2 wipe-tower skipped-priming branch at GCode.cpp:2776–2783 selects
all_extruders.back(), rather than the ordinary first ordered tool; retain this
explicit adjacent tower boundary for subsequent initial-tool wiring. No claim
of complete tower initial metadata or physical activation is made here.

First check exited101 on printer_model's old field path. First Clippy
exited101 on an unconstrained hotend closure index (E0282), before runtime.
Use printer.remaining.printer_model and an explicit usize index; preserve the
existing next-tool compatibility values rather than inventing a new
next-hotend mapping. Check retry exited0, but all overlapping final-* runs
remain intermediate (controls303 PASS55.616s, builds0). Only repaired-*
frozen runs count: controls303/303 PASS51.266s,6805 outside the filter,
spiral-vase Z-ramp PASS9.825s. Clippy exited0,254 lib/270 test diagnostics;
fmt/diff0; WASM core/adapter compilation0 (not browser runtime). Unrestricted
parity build exited0 before both CPU0 official/seeded48-test executions.
Reference results are pending; no independent coupled commit approval.

Both reference handles reached terminal exit100: official47 PASS/1 FAIL
238.222s, seeded47 PASS/1 FAIL281.564s,281 outside the filter. Infill-first
passed4.790s/6.423s; nested painted Slim passed6.648s/8.911s. The sole
failing public dual tracer still has byte186,line7 divergence with unchanged
normalized baseline182130/173811 and modifier270578/226756 byte lengths.
Official cases-N5ytGg/dc7VKU and seeded cases-qQuaMu/6bxAVA retain the prior
input SHA256 pair78eca1c5…/30a532e0…. All four RAW Ares streams equal the
preceding startup-writer counterparts (each cmp0). No new reference failure
needs diagnosis; no fixture, comparator or coverage guard changed. All started
compile/control/reference handles are terminal and reaped. This finite
evidence does not exercise every first-tool ordering, Type2 tower priming,
all-printer/option matrix or physical initial/body transitions. Next coupled
slice: tool-indexed MotionOptions and physical initial selection, then body
buckets/transition templates and all-tool statistics; unchanged dual full
output must become GREEN before coupled milestone commit approval.
Independent six-axis revalidation verified the terminal quality/control and
both complete reference results and closed this finite startup metadata
repair list without new must-fix. It explicitly leaves physical selection,
tool-indexed motion, Type2 priming, body transitions and statistics open;
no independent commit or milestone completion is approved.

### Tool-indexed motion and ordinary initial selection

Continue the unchanged dual3MF RED from repaired initial-filament runs.
Port Extruder.cpp:143–235 logical filament getters, physical travel_slope,
GCode.cpp FILAMENT_CONFIG consumers and GCode.cpp:3144,3227,7710–7746
ordinary initial writer selection into motion/options/build and the existing
first-chunk preamble. Filament-specific fields must consume the selected
logical ID; nozzle geometry/offset and travel slope consume its mapped
physical ID. Remove first-element-only consumers rather than keeping a
fallback copy. Preserve numeric order, get_at out-of-range-first semantics,
nullable override inheritance, machine-start unselected write-back, BBL
pre-render init versus compatible post-render selection and source E reset.
Included: selected-tool motion parameters and ordinary initialization;
deferred in this coupled milestone: multi-tool body buckets/transitions,
temperature sequences and final accounting. Type2 priming/tower ordering
remains the documented adjacent boundary. Public fixture/comparator unchanged,
no new private seam. New functional modules stay below400 lines.
Skirt's existing first-non-Nil planning consumer and its behavioral assertion
remain unchanged; selected motion instead uses get_at and Nil inheritance.
The preamble is moved into layer_gcode/startup.rs before adding tool selection
so the owning module remains below400 lines. Do not reinterpret the BBL
pre-render non-support initialization as its later set_extruder handoff when
initial/non-support IDs differ; that handoff needs the full coupled transition
consumer. No support-first or tower initialization parity is claimed here.

Independent static review found compatible normal startup ordered T/G92
before PA, contrary to GCode.cpp:7739–7747. Move PA before compatible
selection/toolchange only when the source multiple_extruders predicate is
false: GCodeWriter.cpp:78 tests largest registered ID>0, not tool count.
Retain late PA for BBL and the multiple branch (GCode.cpp:8048). Re-review
closed that concrete ordering issue. Overlapping final-* checks remain
intermediate: Clippy254/270 exit0, WASM0. Initial check exited101 when the
still-used skirt helper export was removed; restore that separate unchanged
consumer, not a selected-motion fallback. Check retry/final-check exited0;
remove the newly orphaned parent tags import after the startup split.

The existing exact KSR MotionOptions assertion actually failed at tests.rs:63
(final-controls log): source value0.05235987755982988, old expected
0.05235987755982989. Extruder.cpp:235 evaluates3*PI/180, whereas to_radians
multiplies by pre-divided PI/180. Replace only that expected expression with
the independently worked exact literal0.05235987755982988; keep strict
equality, source production operation order and the public fixture/comparator.
No new private seam is added. Only repaired-* runs after this final freeze
may establish terminal acceptance; prior overlapping runs do not.

Repaired frozen quality gates are terminal: fmt/check and git diff --check
exit0; Clippy exit0 with254 lib/270 test diagnostics; WASM core/adapter
compile exit0 (not browser runtime). The repaired unrestricted public
parity build exited0 before starting both CPU0 reference batches. Repaired
controls and official/seeded48-test full-output batches remain pending;
independent revalidation confirms the strict literal assertion and PA repair
but does not close runtime acceptance before those handles terminate.

Repaired controls reached terminal exit0:303/303 PASS55.220s,6805 outside
the filter. Exact KSR motion assertion PASS4.971s; spiral-vase Z-ramp
PASS9.156s. The official dual tracer has already retained byte186,line7
divergence with the unchanged baseline182130/173811 and modifier270578/226756
normalized lengths. Cases-ylGuRC/BI0g6t retain input SHA25678eca1c5…/
30a532e0…; both RAW Ares outputs equal the preceding ordered-metadata
cases-N5ytGg/dc7VKU (cmp0). Both48-test reference batches are still pending;
do not convert partial control passes into a complete reference result.

Official batch terminated exit100:46 PASS/2 FAIL256.801s. Besides the
unchanged dual tracer, Organic manual case-yNEibv diverged at byte6778,
line263: M73 and the next move exchanged order; normalized lengths both
1110784. Keep that initial failure, not a retrospective47/1 label. Its
input SHA256 is d131533c1b6e3db58359a954e27c372a7de6ab59e4e924f9d848e16f5babdfe2.
RAW Ares output equals previous ordered-metadata case-EEAPFq (cmp0).
The manifest explicitly reports runner_cache/unknown producer provenance;
this evidence alone does not identify an official native nondeterminism cause.
Diagnosing-bugs uses the unchanged public full-output RED and captured input;
no fixture minimization or comparator alteration is appropriate to this
complete-output contract. Ranked probes distinguish native output variation,
Ares regression, and reference-cache provenance without changing production.
Three direct official CPU0 runs on that same saved input all exited0 in
replay directories cqI1SA/TLS8Yq/ktSE0P. Original cached versus direct output
first differs at the same M73 order (excluding only generator date), with
later fill-order differences too. Existing strict replay sweep compares all
three complete outputs: inventory3/compared3/passed3, exit0,14.693s, report
/tmp/ares-tool-indexed-organic-strict-artifacts/replay-summary.json. Original
public Organic manual focused rerun PASS12.881s. No source, fixture or
comparator change was needed; retain the cached failure and unknown provenance
alongside these stronger saved-input results. Seeded batch remains pending.

Seeded batch terminated exit100:47 PASS/1 FAIL300.373s,281 outside the
filter; Organic manual passed. Its only failure is unchanged dual output.
Cases-MBjJqC/gd7nZ4 retain the same input SHA256 pair and both RAW Ares
streams equal preceding cases-qQuaMu/6bxAVA (cmp0). All four official/seeded
dual RAW comparisons are unchanged. All started controls, quality builds,
reference batches, three direct Orca handles, strict replay and focused
public rerun are terminal and reaped. Final diff --check exit0. No fixture,
comparator, coverage guard or production change occurred during the repaired
freeze. Independent six-axis terminal revalidation is requested for this
finite prerequisite, not coupled commit approval. Remaining same-milestone
work: full initial multi-tool set_extruder branch (including BBL non-support
handoff), per-tool body/object/island/region emission, transition templates,
temperature context and all-tool accounting. Type2 tower priming remains an
explicit adjacent source boundary. Neither all-printer/option coverage nor
full initial/body parity is established by these finite48-test results.

Independent six-axis terminal revalidation closed this finite prerequisite's
repair list without new must-fix: selected logical/physical ownership,
registry/options/offset synchronization, PA ordering and strict source-domain
travel-slope equality were verified alongside actual terminal logs. The
review retains official46/2 and seeded47/1, confirms Organic strict3/3 only
for those supplied streams, and states WASM evidence is compile-only. It
explicitly does not approve a coupled commit, complete milestone or full goal;
the unchanged public dual RED drives the next source-cited transition slice.

### Shared writer tool selection

Continue the frozen public dual RED (official/seeded input hashes unchanged).
Port GCodeWriter.cpp:547–585 toolchange_prefix/toolchange and Extruder reset_E
ownership into motion/tool_selection.rs, replacing startup-local installation
and command assembly. Use a shared registry/options/offset selection for both
BBL init_extruder and compatible toolchange, without cloning EmitState or
geometry. Preserve source max-ID multiple_extruders predicate, declared-
diameter compatible command gate, flavor/manual/BBL prefixes, comments and
forced reset_E semantics. Included: writer selection and its actual startup
consumer; deferred within this coupled milestone: GCode::set_extruder's
template/retract/temperature ordering and body tool buckets. Do not equate
the writer operation with the full GCode::set_extruder transition or claim
the public dual test is GREEN. Existing303 controls and unchanged dual/public
Normal/Tree references remain the acceptance seam; no new private test seam.

Writer module80 lines, startup185, registry238; no new geometry copies or
public API. Independent static six-axis review found no new must-fix in the
writer boundary, including no-command installation, max-ID predicate,
prefix priority, forced E reset and preserved PA order. Check exit0; frozen
controls303/303 PASS54.208s,6805 outside filter; Clippy exit0,254 lib/270 test
diagnostics; fmt/check and diff0; WASM core/adapter compile0. These are not
browser-runtime or full transition proofs. Unrestricted parity build remains
live; reference runs must start only after that specific handle exits0.

That unrestricted parity-build handle terminated0 before official/seeded
CPU0 runs. Both48-test batches terminated100: official47 PASS/1 FAIL235.020s,
seeded47 PASS/1 FAIL278.293s,281 outside filter. Only unchanged dual tracer
failed at byte186,line7, baseline182130/173811 and modifier270578/226756.
Official cases-Kc2AKB/phtd2B and seeded9Ops6R/4Fhbhf retain input SHA256
78eca1c5…/30a532e0…; all four RAW Ares streams equal preceding tool-indexed
options counterparts (cmp0). Organic manual passed this time; prior cached
failure remains recorded above. All started handles are terminal and reaped.

User explicitly requests local logical checkpoint commits now rather than
holding every prerequisite for dual full GREEN. This supersedes the earlier
no-coupled-commit gate only for local verified prerequisite checkpoints, not
mainline integration, push or a parity-completion claim. Split by compile
dependencies: layer-tool role producer/order/empty-layer consumer, registry
plus live template ownership and selected-tool options, then shared writer
tool selection. Reconstitute index-only intermediate files when adjacent
hunks share a file; never discard or rewrite the final working-tree content.
Check each staged snapshot independently and document its actual gates,
without assigning the final frozen runtime result to an untested snapshot.
Every commit includes source-cited documents and explicitly says public dual
complete output remains RED. Body buckets, full transition templates,
temperature context and all-tool final statistics remain incomplete.

Local checkpoint1 staged snapshot (producer) independently passed
cargo check --workspace --all-targets, exit0,53.53s; fmt --check and cached
diff --check0. Independent review matched all17 staged files to the exported
snapshot and found no new must-fix or unused writer-only scaffolding.
This snapshot has compile/format evidence, not a newly executed48-test
runtime claim. Later registry/metadata getters and writer installation are
staged only with their actual consumers. Dual complete output remains RED.

Checkpoint1 is local75699e7b. Checkpoint2 combines the actual registry,
relative-E/private-counter migration, common template refresh/write-back,
machine-start lifecycle, ordered initial metadata and selected-tool options
because the shared EmitState fields and template callers must change together.
It retains the previously verified startup-local compatible command shell;
checkpoint3 replaces that shell with the shared writer implementation. Its
exported registry snapshot is independently being checked; fmt0. Final
working-tree source remains untouched by index-only reconstruction. No claim
that this intermediate snapshot ran the final48-test batches is made.

Checkpoint2 registry snapshot reached terminal workspace/all-targets check
exit0,53.16s, with no warnings. Independent review matched all44 source files
to the snapshot, verified local startup has no dependency on the unstaged
writer module and found no new must-fix. Snapshot fmt0 and cached diff0.
Its freshly executed evidence is compile/format, not full runtime parity.

Checkpoint2 is localf07ce210. Checkpoint3 restores the exact frozen shared
writer-selection source, replacing the temporary startup-local shell. Only
motion.rs module registration, motion/tool_selection.rs and startup.rs differ
from checkpoint2; the remaining source already matches the final frozen
controls303/303 and official/seeded47/1 integration runs above. Export and
check the final staged snapshot too; no push or mainline integration is
authorized by this local checkpoint request. All unfinished body/transition/
temperature/accounting behavior remains explicitly open, not silently skipped.

Final writer staged snapshot independently passed workspace/all-targets
check exit0,53.35s, without warnings; snapshot fmt0/cached diff0. Independent
review matched its three source files and plan to the export, confirmed the
first two commit boundaries and found no new must-fix. The final shared writer
source is byte-identical to the previously frozen, runtime-verified integration
source. All three snapshot-check handles have terminated and been reaped;
none was restarted on a timeout. Local checkpoints preserve incomplete dual
full-output status and do not authorize push, main integration or full-goal
completion. Next implementation remains the actual per-tool body buckets and
GCode::set_extruder transition consumer, followed by all-tool accounting.

### Per-tool object/island/region body buckets

Continue the unchanged public dual complete-output RED from both writer
reference batches; fixture, assertion and comparator remain untouched.
Port GCode.cpp:4864–5130 ObjectByExtruder bucket construction, support
dontcare selectors, mixed-wall separation, and5241/5322/5434 per-tool
instance/island traversal, plus ToolOrdering.cpp:102–141 LayerTools::extruder
role selection. Rust destination: layers/tool_buckets with functional
object/support modules and actual layers.rs/object_entry/support_entry/
raft_entry consumers. Move owned paths into buckets; keep object/support/
raft geometry shared, and preserve canonical region IDs and island slots.
Filter support roles exactly as extrude_support6178–6242; do not emit the
same support body on every tool. Writer selection uses the already ported
writer operation at the outer tool boundary, not ad-hoc T words in entities.
Included in this slice: role-owned object/raft buckets and support role
dispatch, actual outer tool traversal and regional four-stage consumption.
Deferred within this same incomplete milestone: the higher-level
GCode::set_extruder template/retract/temperature side effects, per-tool
skirt/brim allocation, subset-specific instance chaining and final accounting.
These are explicitly not waived for complete dual parity. Wipe overrides,
tower integration and Type2 priming remain adjacent source boundaries.
Existing single-tool controls must remain passing; both full dual comparisons
remain the final closure criterion, even if this slice changes body bytes.

Independent six-axis review found three source-contract fixes before acceptance:
FillCollection dispatch now combines every child infill/solid predicate and
the collection role (the first-child rule belongs only to collect_extruders);
raft reuses the same support dontcare selectors and selected-role/ironing
passes; writer tool selection invalidates last_object_copy so the destination
object configuration is reinstalled even for consecutive tools on one object.
Read-only review confirmed these repairs statically. The initial controls303
run passed52.208s, but the four initial check/build handles overlap these edits;
all are intermediate receipts, not repaired frozen-source acceptance. Reap
each handle and rerun controls, quality and full public references after the
repair freeze. No comparator, dual fixture or assertion changed.

The mechanical extraction then exposed missing imported parent modules and
two stale timelapse_gate references; those failed check receipts are retained
as intermediate, not waived. Repair the imports and pass Input's timelapse
value. layers/tool_entry is the actual per-tool instance dispatch, not an
unused compatibility facade. Fixed [Option;2] wall dispatch avoids a new
small Vec allocation on every ordinary wall collection. Final static review
found no remaining must-fix in these repairs. Functional files are282/206/250
LOC or smaller. Repaired freeze: core check0; fmt0; diff0;303 controls all
PASS55.021s; workspace/all-targets clippy0 with253lib/269test warnings, no
dead-code diagnostics (prior254/270); WASM core+adapter check0. Both public
48-case batches and workspace nextest are still running, not accepted yet.
The unchanged dual assertion remains RED byte186,line7 in both references:
baseline182130expected/175043actual, modifier270578expected/233743actual.
Official artifacts M8Lafv/JJy2n6 and seeded cOjnk1/kNgoIj retain inputs and
complete files. Corresponding raw Ares files cmp0 across both references;
the modifier now emits97T0/T1 commands, as does native, but this is only a
body-dispatch diagnostic, not complete transition or output parity.

The first workspace run exposed sortable_fill_collections_flatten_before_
island_print_ordering at tool_regions.rs44 (region0 into an empty vector).
The compiled test executable reproduced exit101 in0.00s. Diagnosis compared
the test's collection_ranges[region0] with the actual fill producer at
fill_entities.rs129–142: the producer installs perimeter_ranges and
thin_fill_ranges for every canonical region, even when empty. This test
omitted that metadata. Supply the same empty region0 ranges, retain every
flattening assertion and add assertions for its preserved internal-solid
tool usage. No production fallback/defensive guard, no erased coverage and
no changed public3MF fixture or full-output assertion. The initial workspace
failure remains recorded; rerun focused+full tests and quality before commit.

Both frozen production reference handles now terminated and were reaped:
official48=47PASS/1FAIL290.398s; seeded48=47PASS/1FAIL395.665s. Only the unchanged
dual full-output test fails in both; all47 prior single-tool/region/support/
raft/tree controls preserve complete output. The region-range repair only
changes unit-test construction, not production/reference source. Workspace
initial run and repaired focused/full/clippy handles still need terminal
receipts. Do not accept an in-flight handle or erase the first workspace RED.

Initial workspace handle terminated/reaped:7555run,7554PASS37slow/1FAIL9skip,
264.377s; its only failure is the reproduced unit-test region-range omission
above. No other regressions; repaired full rerun remains mandatory/in flight.

Focused repaired nextest terminated/reaped:1/1PASS0.023s (test0.013s),7107
outside selection. Original flatten geometry/order assertions and new region
usage assertions all pass. Full repaired workspace and clippy still in flight.

Repaired region-metadata workspace terminated/reaped:7555/7555PASS35slow,
9skip260.126s. KSR initialized ordered bytes and spiral-vase body both pass.
The matching clippy run ended0 but269→271test warnings from the two new
single_range_in_vec_init literals; not accepted as final quality. Replace
those literals with equivalent once(0..0).collect() (no lint exemption),
then rerun full workspace+clippy on that final test-source freeze. Production
source/reference receipts are unchanged. Both final acceptance handles still
in flight: /tmp/ares-body-buckets-acceptance-workspace.log and -clippy.log.
No per-tool bucket commit yet; complete their receipts and independent review
before making the user-authorized local logical checkpoint.

Final acceptance clippy terminated/reaped exit0 (no dead-code diagnostics);
only final acceptance workspace handle remains live. Final static/runtime
review and logical local commit remain pending, not full transition closure.

Final test-source freeze acceptance workspace terminated/reaped exit0:
7555/7555PASS35slow,9skip255.465s. Final clippy0,253lib/269test, no new/dead
code warnings; fmt0/diff0. Production WASM0 and reference47/1 receipts remain
applicable (only equivalent test-range construction changed afterward).
All handles begun by this slice have terminated and been reaped; no timed-out
job was restarted. Request final independent six-axis receipt review before
the local logical checkpoint. The full dual assertion remains untouched RED;
GCode::set_extruder, per-tool skirt/brim, subset instance chaining and all-tool
accounting are still mandatory next slices, not waived or claimed complete.

Final independent six-axis review found no remaining must-fix and accepts
only this local checkpoint after fmt0 (terminated/reaped). Include all
bucket modules, registrations, consumers, metadata test repair and docs in
one logical commit; record both47/1 references and open boundaries in its
body. This does not authorize push or full dual/milestone/goal completion.

### GCode::set_extruder transition lifecycle

9bc65b17 is the verified local body-bucket checkpoint. Resume the unchanged
public dual RED: focused official nextest1FAIL2.460s, byte186,line7, retained
in /tmp/ares-set-extruder-red-official.log. Minimise diagnosis by observing
the first printing T0→T1 in the same public two-nozzle overhang scenario;
do not minimise away baseline/modifier complete-output acceptance.
Ranked hypotheses: missing set_extruder side effects/templates; incorrect
per-tool skirt/brim/instance ordering; incomplete usage/header statistics.
Native body before T1 contains old-tool wipe/retract, change_filament template
and new-tool temperature/PA; Ares still calls writer selection alone.

Upstream boundary GCode.cpp7710–8060 set_extruder and7666–7704 retract,
GCodeWriter.cpp toolchange/retract, PlaceholderParserIntegration3900–3933.
Destination gcode_emit/tool_transition functional modules plus shared template
render consumer and actual layers/startup callers. Included next vertical
slice: old filament_end and new filament_start render/write-back with correct
old physical/new logical IDs, live writer state, layer placeholders, EOL and
template failures. Next same-milestone slices: actual toolchange retract/wipe,
change_filament dynamic values/template, counters, manual/custom-T handling,
temperature/PA, cursor invalidation, then per-tool skirt/brim and accounting.
Adjacent tower/ramming/ooze planning remains source-cited deferred, not common
behavior waived for the Snapmaker fixture. Existing startup/template helper
code is a compatibility shell to share/replace, not a second transition owner.

Confirmed test seam remains public3MF export/import→slice output. Add a
comment-only filament-lifecycle tracer on the same native two-nozzle scenario
to isolate ordering around the first body T1. Its native-derived behavior
assertions are a diagnostic, not a complete-output comparator substitute;
the original baseline/modifier full assertions remain unchanged and RED until
all coupled source behavior is ported. Obtain tracer RED before production
edits, then port this actual lifecycle and rerun both references/controls.

Tracer compile attempts1/2 failed from an assumed ParityCase.actual field
and unavailable multi-thread Runtime::new; repair the test to call the public
slice_project API with the same deterministic metadata/context and the
existing current-thread runtime. These are setup failures, not behavior RED.
Unrestricted tracer build3 exit0; official tracer1FAIL2.790s and seeded
tracer1FAIL2.961s both pass native ordering assertions before failing Ares's
missing old-tool end before first printing T1. Retain complete raw/reference
artifacts D67TYd/PcYo9K and strict full-output DIVERGENT183192/175281bytes.
The original dual complete assertion remains unchanged RED. Tracer clippy0.
Initial actual lifecycle consumer core check0,17.53s; shared writer alone is
now wrapped with actual old-end/new-start render/write-back. Reproduce physical
ID mapping directly from Print.cpp3280–3286 (missing mapping returns0, not
get_at element0). Re-freeze after this source correction; any build started
before that correction is intermediate only, not final acceptance.

The initial lifecycle CLI build handle has terminated/reaped exit0 but its
source overlapped the mapping fix; keep it intermediate. Frozen native CLI
build,303controls,workspace/all-targets clippy and WASM core+adapter handles
are confirmed live, waiting/compiling on the shared target directory. Do not
restart on observation timeout. Await frozen build0, then run both lifecycle
tracers and original dual full-output comparisons before acceptance; independent
six-axis review is in flight. No new lifecycle commit, push or closure yet.

Independent review found two must-fix template-consumer contracts before
acceptance: (1) base_config's cut/EC singular aliases use index0; restore all
four for the current logical filament (old end/new start), matching
GCode.cpp8008–8013. (2) legacy [vector_option] expansion in template.rs263
also uses index0, whereas PlaceholderParser.cpp915 uses the explicit logical
filament argument to placeholder_parser_process. Pass an immutable logical
filament context through actual rendering; do not infer it from the end
template's physical filament_extruder_id, nor alter explicit modern indices.
Extend the public native tracer with differing per-tool alias/vector values
and obtain behavior RED on this lifecycle source before repairing those
contracts. Current comment-ID tracer alone cannot prove them. Frozen checks
already begun remain intermediate until these review fixes and final reruns.

Intermediate lifecycle quality receipts terminated/reaped: clippy0 with
253lib/269test warnings; WASM0; fmt0; unrestricted frozen CLI build0. These
do not close the two template-context review issues. Core303 handle54672
and official/seeded lifecycle+original-dual handles93567/77913 are confirmed
live and must be reaped before their results are assessed; their logs use
ares-tool-lifecycle-frozen-controls and ares-tool-lifecycle-intermediate-*
under /tmp. Keep unfinished coupled code/tests/docs in the worktree, not a
commit or push, until repair and independent revalidation.

### Local checkpoint status after the current reference round

The user requested logical local commits without waiting for complete dual
parity. The four existing checkpoints are 75699e7b (layer tool producer and
ordering), f07ce210 (live extruder registry and machine-start write-back),
5b2da82a (shared writer selection), and 9bc65b17 (actual body tool buckets).
Each commit message names its verified scope and explicitly retains the full
dual RED. Independent read-only review confirms no message amendment is
needed. The branch is four commits ahead of its own remote; none is pushed.

Intermediate handles54672/93567/77913 terminated and were reaped: core
303/303 PASS; each reference round reported the original dual RED and a
lifecycle tracer failure. Inspecting the saved complete Ares stream proves
the tracer failure was a test-boundary false negative: old end is immediately
adjacent to T1, and body[..tool] excludes the newline opening the T1 match.
Replace newline-containing substring matching with exact line matching in
that same pre-T1 interval. Marker contents and ordering remain unchanged;
neither full comparator nor original dual fixture/assertion is changed.
Independent review confirms this correction, subject to real reruns.

The unrestricted corrected-tracer build exited0. Fresh official and seeded
two-test runs are retained in /tmp/ares-lifecycle-line-{official,seeded}.log
with full artifacts under the corresponding -artifacts directories. These
handles58336/25536 terminated/reaped exit100: corrected lifecycle tracer PASS
3.761s official /3.396s seeded; unchanged original full dual FAIL5.001s
official /6.739s seeded. Each round has one PASS and one FAIL. Fmt and diff
checks exit0. These are intermediate lifecycle evidence, not final acceptance: the scalar alias
and immutable legacy-vector context review fixes remain open. Keep those
coupled files uncommitted; do not amend or combine the four verified commits.

### Selected filament aliases: public diagnostic loop

Extend the confirmed public3MF export/import seam with distinct EC values
3/7 and false/true. Retain native-first assertions on old end, new start and
the later old end after tool1 was selected; preserve full stream artifacts
and the original complete-output acceptance unchanged. The initial cut probe
exports printer vectors10/18 and false/true but native templates normalize
them to10/10 and false/false with the default cut override level. This is
fixture setup evidence, not a production RED. The calibrated native run
passes all native assertions, then fails Ares old-end bool rendering (0
instead of false),1.864s, /tmp/ares-alias-tracer-red2.log, full case-yHpKVs.

For distinct cut values, additionally select source EnableFilament(2) and
set per-filament cut overrides10/18,false/true. Verify effective project
configuration before using native output as the oracle. Do not infer runtime
values solely from the printer vectors or waive any alias assertion.

The EnableFilament public fixture passes all native assertions, then fails
Ares old-end aliases1.853s, /tmp/ares-alias-filament-red.log, case-t63Gc1.
This is the valid four-alias RED. Production now shares alias selection
between base context and live writer refresh, selecting the logical filament
and preserving bool output rather than footer numeric serialization. Initial
build failed only on helper re-export visibility (E0364/E0603); corrected
module visibility, reaped exit101, and rerun unrestricted build2. No behavior
GREEN, final verification, commit or push is claimed by this compile attempt.

Independent six-axis alias review found no new production must-fix, but
requested effective project read-back of EnableFilament(2) and both filament
cut override vectors. Add those checks before final runtime validation. This
fixture proves selected alias values/types, not override priority (printer
and filament cut inputs coincide). Legacy-vector indexing stays open. Build2
handle15920 and intermediate clippy96390 remain confirmed live; no restart
on timeout and no GREEN acceptance before terminal runtime results.

Unrestricted build2 handle15920 terminated/reaped exit0 after the effective
cut configuration assertions were added. Fmt/diff checks pass. Actual nextest
runs are started and confirmed live: official44435 and seeded89978 select
both lifecycle tracers plus the unchanged original dual; core controls80168
and workspace/all-target clippy96390 share the build directory. Logs are
/tmp/ares-alias-green-{official,seeded}.log, /tmp/ares-alias-controls.log and
/tmp/ares-alias-intermediate-clippy.log. These handles currently wait/run;
reap them, do not restart on timeout. No final GREEN or new commit/push yet.

Alias round receipts terminated/reaped: clippy96390 exit0,253lib/269test,
CLI31; official44435 has lifecycle3.731s PASS and aliases3.747s PASS, original
dual4.956s FAIL; seeded89978 lifecycle4.812s PASS, aliases2.842s PASS, original
dual6.657s FAIL. Each reference round is2PASS/1FAIL. Core303 handle80168 is
still confirmed live. Do not attribute these receipts to the following new
legacy-context test source.

Next vertical slice, same confirmed public seam: add native-first legacy
vector comments with EC3/7, explicit modern indices0/1, and an assignment
of mutable current_extruder=0. Validate first old end, new start and later
old end; the implicit legacy index must remain the immutable logical tool
argument, independent of that mutable symbol. Source boundary remains
PlaceholderParser.cpp::legacy_variable_expansion915 and GCode.cpp7773/8042.
After valid RED, pass the explicit logical context through rendering and its
recursive nodes; keep explicit modern indices unchanged. Existing default
render callers are not claimed to close remaining startup/end lifecycle
ports. Unrestricted legacy tracer build61755 currently waits on the shared
target while core controls80168 run; reap before native replay.

Independent review closes the finite four-alias repair after inspecting both
runtime logs and effective override read-backs, but not the overall lifecycle
or regression gate. Legacy tracer is statically valid; its context is only
independent of ordinary current_extruder assignment. Upstream explicit
filament_change() may modify parser context (PlaceholderParser.cpp1719–1722),
and is not exercised or claimed implemented by this slice. Await valid RED
before its render implementation; do not declare global context immutability.

Core30380168 terminated/reaped exit0,303PASS53.699s; original alias code
passes those controls. Legacy build61755 exited0, but the assignment probe
fails natively with process156 /Cannot modify a read-only variable/ for
current_extruder,0.962s in /tmp/ares-legacy-context-red.log. This invalid native
input is setup evidence, not a valid production RED. Correct the static review
assumption: current_extruder is a read-only symbol, not a writable ordinary
variable. Remove that illegal assignment; retain logical tool1's legacy7 and
modern explicit3/7 observations at start and later end. Legal tracer build
20641 exited0. Full contract still passes an explicit logical argument, not
the end-template physical filament_extruder_id. No render implementation
before the legal tracer's native-first RED is observed.

Legal native-first tracer9941 terminated/reaped FAIL1.865s: all native
assertions pass, Ares new tool1 legacy value is tool0's3 instead of7 while
modern explicit indices stay3/7. Retain complete case-umdixi and strict
DIVERGENT in /tmp/ares-legacy-context-legal-red.log. Implement explicit logical
filament context in template::render_for_filament and recursive rendering;
actual tool_transition passes its logical old/new argument. Existing render
callers retain their prior tool0 call context; remaining startup/end lifecycle
ports are still open, not a new fallback or completion claim. Unrestricted
green build is running; await runtime GREEN and independent revalidation.

Independent read-only six-axis review finds no additional must-fix in this
finite legacy index slice; actual template346LOC and test212LOC, no new
copies/cache/suppressions. Existing tool0 render call sites and explicit
filament_change() are not silently claimed ported. Final unrestricted CLI
build91937 terminated/reaped exit0. Frozen fmt/diff0; live checks queued/run:
clippy27940 (/tmp/ares-lifecycle-context-final-clippy.log), WASM32562
(/tmp/ares-lifecycle-context-final-wasm.log), workspace84768
(/tmp/ares-lifecycle-context-final-workspace.log). Start official and seeded
four-test nextest rounds selecting all three lifecycle diagnostics plus the
unchanged original dual; logs /tmp/ares-lifecycle-context-final-*.log and
full artifacts under the matching -artifacts directories. These process
handles must be reaped before asserting GREEN; no new commit/push yet.
Official65742 and seeded81696 are confirmed live with the three quality
handles27940/32562/84768; source is frozen while these checks run or await
the shared build lock. Observation timeout is not terminal or a reason to
restart any of these five processes.

Frozen quality handles terminated/reaped: clippy27940 exit0,253lib/269test
(CLI31),2m16s; WASM32562 core+adapter check exit0,2m33s. Workspace84768
and focused official65742/seeded81696 remain confirmed live. Keep source
frozen. Broader unchanged Normal/Tree/region finite references are additionally
started with all twelve activation flags and strict complete comparisons:
official22081 /seeded9538, logs /tmp/ares-lifecycle-context-controls-*.log,
full artifact roots with the same prefix and -artifacts suffix. This batch
includes the new lifecycle diagnostics but does not redefine original dual
full-output acceptance or imply all-printer/all-option coverage. Await all
terminal results before independent runtime review and any local commit.

Focused official65742 and seeded81696 terminated/reaped exit100, each3PASS/
1FAIL. Official lifecycle15.240s, legacy16.038s, aliases15.458s all PASS;
seeded21.672s/19.756s/15.820s all PASS. Original full dual remains unchanged
FAIL17.365s official /25.365s seeded, still header byte186,line7. These are
actual activated native-first diagnostics, not the workspace's env-gated
early-return receipts. Independent runtime revalidation requested for the
two finite context fixes only. Workspace84768 (7558 tests) and broader
51-case controls22081/9538 remain confirmed live; do not commit or claim
global parity from these focused windows.

Independent reviewer inspected the actual focused logs and closes both
finite must-fixes (all four aliases and legacy logical default index), with
the lifecycle window also PASS in both references. Approval is explicitly
not all GREEN: each round3PASS/1FAIL, clippy retains253/269 warnings and WASM
is compilation only. No commit or full dual closure until workspace84768
and both broader51-case batches22081/9538 terminate and are revalidated.

Frozen workspace84768 terminated/reaped exit0:7558/7558PASS,36slow,9skip,
260.337s, /tmp/ares-lifecycle-context-final-workspace.log. Keep the activated
native-reference receipts separate from env-gated workspace tests. Both
51-case reference handles22081/9538 still run; their terminal totals and
independent runtime review remain required before a local checkpoint.

Final broader reference handles22081/9538 terminated/reaped exit100:
official50PASS/1FAIL378.832s, seeded50PASS/1FAIL425.235s. Both batches
preserve all47 existing strict complete-output controls and pass all3 new
native-first lifecycle diagnostics. The only failure is the unchanged public
dual baseline/modifier complete assertion,10.850s /14.108s respectively.
Do not label the diagnostic3 as complete-output parity cases. Final fmt/diff
checks exit0, all current validation handles are terminal/reaped, and the
source remains frozen. Submit these final receipts for independent six-axis
approval of a local lifecycle checkpoint only; no push/full dual closure.

This checkpoint includes old-end/new-start ordering, physical end/logical
start IDs, live template write-back, per-tool cut/EC aliases and explicit
logical legacy-vector context. Required remaining source slices still include
startup/end callers, full toolchange retract/wipe/counters/custom change and
temperature/PA handling, per-tool skirt/brim and subset instance ordering,
and all-tool output accounting. They stay open in this milestone; a local
checkpoint does not waive them or close all-printer/all-option scope.

Final independent six-axis review found no new must-fix and approves only
this finite code+docs local checkpoint. Commit body must retain full dual RED
and the remaining startup/end/full-transition/per-tool allocation/ordering/
accounting scope. No push/main or complete multi-tool/all-printer claim.

## Printing toolchange retract/wipe slice

The lifecycle checkpoint is committed locally as 9604a0b5. Continue the same
milestone, retaining its original full baseline/modifier assertion. Port
`GCode.cpp::set_extruder` 7751–7755 and `retract` 7652–7704,
`Wipe::calculateWipeRetractionLengths` 310–358,
`GCodeWriter::retract_for_toolchange` 1015–1023, and
`Extruder::retract_length_toolchange` / `retract_restart_extra_toolchange`
205–212 into the existing motion/travel sequence and tool_transition consumer.
The two toolchange getters use physical extruder identity, unlike ordinary
filament retraction. Reuse the live counters and wipe geometry; do not swap
options temporarily or copy motion state. Included: old-tool retract/wipe,
restart-extra assignment, reset_E, lazy lift and unconditional wipe-path
reset before old-end. Subsequent eager/change template, temperatures/PA,
startup/end and allocation/accounting remain open required source slices.

TDD seam is the previously approved public exported 3MF -> slice bytes with
native-first observations; no private-state injection. A new tracer changes
only public toolchange lengths to 6/9 and extras to 0.25/0.5, verifies their
export, then reads old-tool counters through legal end-template placeholders.
Strict full artifacts are retained as DIVERGENT rather than replacing the
original full-output acceptance. Require native-first RED, then actual GREEN,
unmodified controls and independent six-axis review before checkpointing.

Initial command31287 terminated exit100 before invoking Orca because the
artifact variable was misspelled; setup failure, not RED. Corrected command
15235 invokes native output but fails the second native assertion: exported
6/9 and 0.25/0.5 become runtime6/6 and0.25/0.25 in this printer variant.
Retain case-APsYec and explicit native config assertions; this is not unequal
per-nozzle coverage. Calibrated command13592 terminated/reaped exit100,
1.870s, /tmp/ares-toolchange-retract-calibrated-red.log, case-IiB4n3. Native
assertions all pass; Ares old-end reports0/0 instead of6/0.25. This is valid
public-seam RED before production changes. Implement mode-aware shared
retract/wipe, physical toolchange getters, old-tool lazy lift and unconditional
wipe reset; subsequent eager/custom change remains open.

First GREEN85701 terminated/reaped exit0, native-first test PASS1.877s.
Independent review found a required semantic fix: Hilbert suppression reads
`GCode::on_first_layer` (`GCode.hpp:670`), not m_layer_index. Carry the actual
current layer's id/bottom-Z projection in EmitState, initially from the
source-leading object-or-support chunk layer (`GCode.cpp:4559–4591,4664`),
then refresh at each object/support/raft emission matching m_layer changes.
Use this only for the new toolchange Hilbert gate; lift enforce's chunk-index
contract remains unchanged. The current public tracer is not evidence of
mixed-object/elevated-support Hilbert coverage. Quality checks88026/59052/
84353 were interrupted deliberately and reaped exit130 before this fix;
do not count them as completed validation. Rerun final checks on frozen fix.

Independent static revalidation closes the on_first_layer fix with no new
must-fix. Frozen official focused30599 terminated/reaped exit100:4PASS/1FAIL
5.233s, new retract tracer4.636s. Failure is solely the unchanged original
dual complete comparison; all three previous template diagnostics also pass.
Final clippy15789 terminated/reaped exit0,2m12s,253lib/269test warnings and
no dead_code matches; final WASM82924 core+adapter check terminated/reaped
exit0,2m29s. fmt/diff checks exit0. Workspace26732, seeded focused67346,
official broader37466 and seeded broader13903 remain confirmed live. Each
broader batch includes the47 prior strict complete-output controls,4 limited
native-first diagnostic tests, and unchanged original dual contract. Await
terminal results and independent actual-log review; no full/parity or commit
approval from these partial receipts.

Concurrent workspace26732 has a real FAIL in the unmodified command-lifecycle
test timeout_returns_before_descendant_closes_inherited_output_handles:
bounded descendant did not finish (5.864s). Preserve /tmp/lifecycle-Ydrjsu,
command-m7Bcnu; direct child SIGKILL/reaped after500ms, all snapshot/live
streams0bytes, no direct-child.pid or descendant finished marker. Independent
read-only review supports a helper startup-path blockage hypothesis but does
not establish scheduling as its cause or prove no descendant existed. Do not
label this known or count this full round as PASS. Single-case diagnostic21960
is running with original assertions. After all current handles terminate,
rerun unchanged full workspace without external Orca batches; preserve both
the initial result and clean rerun evidence, and require revalidation.

All first-round handles are now terminal/reaped. Workspace26732 exit100:
7558PASS/1FAIL,279.307s, solely the lifecycle helper above. Isolated21960
exit0/PASS3.065s with unchanged assertions; this supports environment
sensitivity, not erasure of the initial FAIL. Seeded focused67346 exit100:
4PASS/1FAIL257.291s, solely original full dual; new retract196.914s.
Broader37466/13903 exit100: each46PASS/6FAIL273.987s/280.284s. In addition to
the original full dual, default auto Organic and manual default/organic/grid/
snug styles differ. These first invocations mistakenly restricted only the
Orca child, not the entire replay; they do not implement the already-approved
whole-process CPU0 plan above. Ares Organic generation workers() reads
available_parallelism; merge_areas.rs chooses2- or4-leaf buckets from that
count. Previous saved PASS case-OGDQvB and new case-1BbebX have identical
project_settings.config and Orca body (after date header), but Ares support/
skirt geometry differs. This is evidence of an execution mismatch, pending
the actual corrected replay, not permission to waive output assertions.

Source remains frozen. Start corrected whole-process taskset -c0 official/
seeded52-test controls, retaining both original and corrected logs/artifacts
at /tmp/ares-toolchange-retract-{controls,cpu0}-{official,seeded}*. No new
reference patches or production single-worker clamp; both consumers run at
the previously approved same one-CPU replay boundary. After these handles
terminate, run full unchanged workspace without concurrent external batches.

Corrected whole-process CPU0 official36432 terminated/reaped exit100:
51PASS/1FAIL250.346s. All47 prior strict complete-output controls and4
native-first diagnostic windows PASS; only original full dual FAIL5.048s.
Each of the five Organic cases that failed under the mismatched execution
domain passes without source/input/assertion changes. Seeded89586 remains
live, with those five cases already PASS; await its complete terminal totals
and the subsequent clean full-workspace run before local commit approval.

Corrected seeded89586 also terminated/reaped exit100:51PASS/1FAIL296.345s,
only original dual FAIL6.706s. All47 strict complete controls and4 diagnostic
windows pass; the five Organic domain mismatches are resolved in both
references without production changes. Keep the mismatched domain failures
as evidence, not reference nondeterminism or ignored tests. All native batch
handles are now terminal/reaped. Rerun the unchanged lifecycle test alone,
then the entire unchanged workspace without any external Orca batches.

Independent actual-log revalidation closes all five suspected Organic
regressions in the approved shared CPU0 domain only; it does not establish
multiworker determinism. Clean lifecycle22443 terminated/reaped exit0,
PASS3.026s with original assertions after all native batches ended. Now run
clean full workspace97216 alone, /tmp/ares-toolchange-retract-clean-workspace.log.
No source changes since the reviewed first-layer fix; final Clippy/WASM/fmt
receipts still apply. Do not replace the earlier 7558PASS/1FAIL workspace
record or call the remaining live full process complete.

Clean full workspace97216 terminated/reaped exit0:7559PASS,9SKIP,
35slow,256.465s. All tests ran with unchanged configured parallelism and
without concurrent external Orca batches. Log:
/tmp/ares-toolchange-retract-clean-workspace.log. This closes the current
checkpoint's workspace gate; retain the earlier overloaded 7558PASS/1FAIL
record and both unchanged lifecycle reruns rather than recasting that run
as successful. Final fmt76559 exit0 and git diff --check exit0; final frozen
source Clippy15789 exit0 (253lib/269test warnings, no new dead code) and
WASM core/adapter82924 exit0 remain applicable.

Independent static six-axis revalidation found no new must-fix and confirmed
the real first-layer versus lift-counter separation. Final runtime approval
is requested against the actual clean workspace receipt. The local code+
documentation checkpoint remains limited to printing toolchange retraction,
wipe reset and lazy normal lift. Official/seeded shared-CPU0 each51PASS/1FAIL
does not close full dual-tool parity: the original complete-output comparison
remains RED. Startup/end callers, subsequent transition behavior, ordering/
accounting and the global printer/option matrix remain open; no push or main
merge is claimed by this local checkpoint.

Independent final six-axis review verified the actual clean terminal log
and approved this finite code+docs local checkpoint with no new must-fix.
Approval excludes push/main merge, complete transition/accounting parity
and global goal completion; original full dual output remains RED.

## Scheduled filament header IDs

Current source is checkpoint1f15f498, clean before this slice. Fresh unchanged
full baseline/modifier native replay3273 exit100,2.472s; byte186,line7 expects
`; filament: 1,2`, emits `; filament: 2`. Retain
/tmp/ares-tool-transition-current-red.log and its artifacts. Hypotheses:
initial-only selection, first-layer-only selection, or lost logical identity.
Inspection falsifies those: header.rs serializes the configuration usage
COUNT, not IDs. Upstream `GCode.cpp:2623–2632` serializes the actual
`Print::get_slice_used_filaments(false)` IDs, populated by
`Print.cpp:2514` from `ToolOrdering::all_extruders`.

Same approved public 3MF import/slice/output seam: add a native-first header
tracer using the unchanged two-filament export helper. Obtain its RED before
changing production; then borrow schedule.tool_ordering.all_extruders() in
header.rs and format one-based logical IDs. Include only scheduled-ID header
output; existing config inventory is not removed or reused as geometry truth.
Statistics/transition/all-printer/all-option coverage remain deferred/open.
Require both references' tracer GREEN, original full comparisons retained,
workspace/fmt/clippy/WASM and independent six-axis checkpoint review.

Header tracer4807 terminated/reaped exit100,1.778s: native header assertion
passes first; Ares emits `2` instead of `1,2`. Log
/tmp/ares-filament-header-red.log. This is valid public-seam RED before
production edits. Borrow the existing Schedule in append_header, retaining
five arguments and avoiding a new too-many-arguments warning. Keep max-Z,
layer-count and header-disable behavior unchanged; serialize the schedule's
borrowed all-tools list directly, no new ID inventory or geometry copies.

Source frozen after implementation. Build73535 exit0,1m15s; fmt16401/88722
exit0; diff check0. Clippy43533 exit0,2m27s, unchanged253lib/269test warnings;
WASM core+adapter89995 exit0,2m43s (compile, not browser runtime).
Independent static six-axis review found no new must-fix and confirmed ID
ownership, unchanged count/header-disable behavior and no new argument warning.

Official shared-CPU0 broad65918 terminated/reaped exit100:52PASS/1FAIL,
254.196s, /tmp/ares-filament-header-official.log and matching artifacts.
New header diagnostic PASS3.601s, all47 strict complete-output controls and
the4 prior limited diagnostics PASS. Only original full dual FAIL4.969s:
baseline's first difference advances to byte998,line33 total_toolchanges16
versus0; modifier first differs at byte718,line25 M73 P0 R17 versusR16.
Header correction does not close these remaining runtime/accounting gaps.
Seeded64723 remains live; require its terminal totals and a subsequent full
workspace alone before local commit approval. No comparator/fixture changes.

Seeded64723 terminated/reaped exit100:52PASS/1FAIL,300.331s. Header tracer
PASS4.725s; all47 strict complete controls and4 prior diagnostics PASS.
Only original full dual FAIL6.686s, with the same new first differences.
Log /tmp/ares-filament-header-seeded.log and matching artifacts retained.
All external batches are terminal. Start unchanged full workspace alone,
/tmp/ares-filament-header-workspace.log; source remains frozen. Do not claim
workspace GREEN or final checkpoint approval before its terminal receipt.

Full workspace14113 terminated/reaped exit0:7560PASS,35slow,9SKIP,
257.593s. Original configured num-cpus parallelism, no concurrent external
Orca batches, no assertion changes. /tmp/ares-filament-header-workspace.log.
Frozen-source fmt/diff/clippy/WASM gates and both actual52/1 reference rounds
are complete. Request final independent six-axis approval for this header-ID
code+test+docs checkpoint only; full dual remains RED and no push/main merge
or global printer/option completion is implied.

Independent final six-axis review verified workspace's actual terminal
receipt and approved the finite header-ID code+test+docs local checkpoint,
with no new must-fix. Approval excludes push/main merge and complete dual
or global goal closure. Next original full-output gap is schedule-derived
total_toolchanges and remaining transition/time/extrusion accounting.

## Non-tower scheduled total tool changes

Start checkpoint11742228, clean. Fresh unchanged public full replay18814
exit100,2.473s, /tmp/ares-tool-count-current-red.log and matching artifacts:
baseline byte998,line33 total_toolchanges16vs0; modifier byte718,line25
M73 R17vsR16. Hypotheses: uninstalled prediction, missing cross-layer changes,
or physical/logical confusion. Inspecting placeholders.rs confirms prediction
still defaults to0; upstream owner is `GCode.cpp` DoExport's
total_toolchanges_from_ordering2289–2300/resolve_total_toolchanges2304–2309.

Same approved public 3MF-to-output seam, native-first diagnostic in
compatible_regions/tool_templates.rs: comment templates expose integer
total_toolchanges and division-by3, and compatible summary must report16.
Obtain actual RED before changing production. Port one-pass ordered logical
transition counting (skip first/repeated IDs, preserve last across layers).
Compute once in existing InitialFilaments immutable export shell, install
Integer through shared metadata refresh and reuse value in positive-only
finish.rs summary. Keep source sequential empty global ordering prediction0;
no new support guard or output assertion relaxation. Tower count precedence,
dynamic toolchange_count and other accounting remain deferred/open.

Independent read-only boundary review agrees with this metadata owner and
requires the above logical/sequential/tower distinctions. Verify tracer and
prior strict full-output controls against both references, retain original
dual RED, then workspace/fmt/clippy/WASM and final six-axis local review.

Red build20940 exit0,11.42s. Native-first tracer99828 terminated/reaped
exit100,1.848s, /tmp/ares-tool-count-red.log and case-n5Et0N. All native
template and summary assertions pass; Ares emits `0 0` instead of `16 5`.
Implement only after this valid RED: ToolOrdering one-pass prediction,
InitialFilaments immutable Integer installation and shared positive-only
compatible tail summary. Aggregate weight/cost arguments into their existing
pair to retain five finish parameters without new argument-count warnings.

First implementation patch matched the earlier first_non_support call rather
than the struct initializer: fmt37464 exit1 and build69908 exit101 (parse
error/missing field), /tmp/ares-tool-count-build.log. Both terminal before
correction. Move the initializer to its intended struct; these are compile
failures, not a new behavioral RED or a verified source snapshot.

Corrected fmt5841 exit0 and build88964 exit0,1m15s. Independent static
six-axis review finds no new must-fix and confirms logical transition
semantics, sequential0, shared Integer/summary owner and deferred tower
precedence. Source frozen. Clippy13091 exit0,2m17s, unchanged253lib/269test
warnings; WASM core+adapter79595 exit0,2m33s (compile only). Final fmt56876
and diff checks exit0. Prebuild all workspace tests55201 with unrestricted
CPU before approved whole-process CPU0 replays, avoiding single-CPU compile
cost; this build remains live and is not yet accepted as complete.

Prebuild55201 exit0,2m47s. Official84634 terminated/reaped exit100:
52PASS/2FAIL257.274s; seeded50673 exit100:53PASS/1FAIL303.498s. New total
diagnostic PASS3.748s/4.918s, all prior windows pass. Original dual remains
RED: baseline now first differs byte1726,line124 (native M73 P6 R14 versus
Ares G92 E0); modifier remains byte718,line25 M73 R17vsR16. Actual logs
/tmp/ares-tool-count-{official,seeded}.log and matching artifacts retained.

Official also fails manual Snug painted at byte6778,line263: M73 P3 R28
falls before rather than after the next motion. Do not erase this FAIL or
report official53/1. Case-mz42Be current FAIL and prior header slice official
case-0cJAce PASS have byte-identical input.3mf (SHA9802720d91bebeb8c8789462512e99e1f31699fb0c2eb04d6462adc1b0b0171d)
and byte-identical Ares.gcode (SHAb55b8c274069dfb712f0333c84d6f1eeb0015b9a83c41eae7d0f7d4a322431e1),
both cmp exit0. Native outputs differ after excluding generated-date line:
first M73 position, then reordered small solid-fill blocks around original
line2143, followed by more shifted M73 positions. Thus not an Ares-output
regression from this slice; native cause is not established. Current seeded
PASS case-urSYzY has the same Ares SHA. Preserve all raw artifacts/provenance
limits; do not alter geometry or filter M73 in the acceptance comparator.
Run unchanged public Snug official replays alone, then full workspace alone.

Independent review confirms byte-identical input and Ares output against the
prior PASS, and actual native differences; do not infer a specific native
mechanism. Unchanged official public Snug replay72604 exit0/PASS3.149s,
case-Ab0lbo; replay86788 exit0/PASS3.135s,case-wE1Fbd. Both replay input.3mf
and Ares.gcode cmp exit0 against failing case-mz42Be; original full comparator
PASS without filtering M73 or geometry. Logs
/tmp/ares-tool-count-snug-official-replay{1,2}.log and artifacts retained.
These separate exact-input PASS receipts do not change official batch52/2.
All external batches/replays terminal/reaped; start full unchanged workspace
alone, /tmp/ares-tool-count-workspace.log. Source still frozen. Await actual
terminal summary and final independent six-axis approval before checkpoint.

Full workspace84987 terminated/reaped exit0:7561PASS,35slow,9SKIP,
258.345s, /tmp/ares-tool-count-workspace.log, unchanged configured num-cpus
parallelism and no external batches. Source stayed frozen; fmt/diff/clippy/
WASM receipts remain applicable. Request final independent six-axis local
code+test+docs approval. Keep official52/2 and seeded53/1 plus the separate
exact-input official Snug2PASS receipts; complete dual remains RED and neither
reference internal cause nor tower/dynamic/full accounting is claimed closed.

Independent final six-axis review verified actual7561PASS and all retained
reference/replay receipts; approved this finite non-tower prediction
code+test+docs local checkpoint with no new must-fix. Approval excludes
push/main merge, complete dual parity, tower precedence, dynamic counters
and global printer/option completion.

## Printing pressure advance restoration

Start clean b1e2558c. Fresh unchanged full dual replay63049 exit100,
2.467s, /tmp/ares-transition-current-red.log; baseline case-LgubvM byte1726
line124 M73 position, modifier case-hO9U0g byte718 line25 M73 remaining
minutes. Three hypotheses distinguish absent custom transition blocks,
writer lift/motion state, and independent timing errors. Raw baseline diff
shows all three remain plausible: missing printer custom block, SpiralLift,
temperature and pressure-advance commands plus changed motion speeds.
Do not fix or filter M73 independently to claim closure.

Select the independently bounded fixed PA command consumer first:
GCode.cpp::set_extruder8048–8053 and GCodeWriter.cpp::set_pressure_advance
370–391. Existing startup.rs formats it correctly for this fixture but
tool_transition never calls it. Move that existing formatter/getter unchanged
to one shared functional module, preserve startup callers, and append after
printing filament-start. Adaptive PA processor state, custom change template,
eager lift, temperature and complete dual accounting remain open.

Public native-first outputs helper, unchanged public 3MF seam: insert a
comment-only filament-start marker and assert each of sixteen printing
markers is followed by M900 K0.02 before its object body. Build93646 running;
obtain behavioral RED before production edits. Original complete comparator
still runs and retains its divergent artifacts. Verify both references,
unchanged finite complete controls, workspace/fmt/clippy/WASM and independent
six-axis read-only review before a finite local code+test+docs checkpoint.

Red build93646 exit0,11.79s. First replay8142 exit100,1.812s is NOT valid
behavioral RED: native scanning also matched template text in trailing config
and unwrapped a missing object. Production still untouched. Limit this new
diagnostic to LAYER_CHANGE..EXECUTABLE_BLOCK_END, rebuild47274 exit0, then
replay53064 exit100,1.822s: native all16 windows and count pass first, Ares
fails missing pressure advance. /tmp/ares-transition-pa-red2.log and
case-hdYfHr retain actual RED. Only then extract existing helper mechanically
and call it after new printing filament-start.

Independent preflight approves this limited consumer boundary, not acceptance.
Review identified pre-existing formatter scientific-notation gap (e.g. valid
PA1e-5), decimals=0 trimming (large values require input reachability evidence),
and BBL-first flavor priority mismatch. Keep them explicitly open for a
subsequent source-cited writer-format slice; this checkpoint proves J1 PA0.02
and unchanged prior startup controls, not all PA values/flavors. No adaptive
PA reset implementation or full dual closure is claimed.

Source frozen after mechanical extraction and consumer. Static six-axis
review has no new finite-scope must-fix, requires actual runtime receipts.
Clippy70660 terminated/reaped exit0,1m20s, unchanged253lib/269libtest and31
CLI parity warnings; WASM86997 exit0,1m37s, core+adapter compile check only.
Final fmt/diff checks exit0. Start full workspace prebuild with unrestricted
CPU after quality checks, then approved whole-process CPU0 reference replays.

Full native test prebuild53801 terminated/reaped exit0,2m47s. Start official
42698 and seeded82586 unchanged broad controls plus new tracer:55 tests each,
all12 explicit reference activation flags, approved whole-process CPU0.
Logs /tmp/ares-transition-pa-{official,seeded}.log and matching artifacts.
Do not infer completion from live tail output; await both terminal receipts,
then run workspace alone without native references competing for CPU.

The CLI-only replay target nevertheless rebuilds its ares_core feature
artifact after workspace prebuild; batch logs initially show compilation,
not test progress. Preserve live handles. Validated rustc1568815 alone is
compiling this worktree's ares_core (feature set empty), affinity0. Expanded
only that compiler and its threads to CPUs0–63 to finish the build; cargo,
nextest and future reference/Ares runtime children remain constrained to0.
No production worker clamp or replay-domain change. Future prebuild must
also match the exact CLI target feature set, not assume workspace cache hit.

Both reference batches terminated/reaped: official42698 exit100,
54PASS/1FAIL261.269s; seeded82586 exit100,54PASS/1FAIL308.969s. New PA
diagnostic PASS3.670s/4.808s, all7 limited diagnostics and all47 prior strict
complete controls pass both references. Original complete dual remains the
sole RED: baseline byte1726,line124 M73 placement and modifier byte718,
line25 M73 remaining minutes; e.g. official case-n4HtYF/case-G7DQfn raw
outputs retained. Expected/Ares lengths baseline182130/175924 and modifier
270578/237223; no comparator relaxation. Earlier official Snug uncertainty
receipt remains historical, not erased by this batch's passing control.

Verified both live nextest parents1568795/1568821 affinity0 after compiler
exception. Reference runtime domain stayed single CPU. All external batches
ended before full unchanged workspace starts alone, with configured num-cpus
parallelism, /tmp/ares-transition-pa-workspace.log. Source remains frozen;
await actual full terminal receipt and independent final six-axis approval.

Full workspace81077 terminated/reaped exit0:7562PASS,35slow,9SKIP,
255.408s, /tmp/ares-transition-pa-workspace.log. No external reference
batches overlapped this run. The prior spiral-vase ramps-Z regression test
passes10.572s. All changed Rust files remain below400lines. Source stayed
frozen; retained fmt/diff/clippy/WASM receipts apply. Request final independent
six-axis finite local checkpoint approval; not push/main or complete dual,
all-printer/option, adaptive PA or formatter-domain closure.

Independent final six-axis review verifies actual7562PASS and reference54/1
receipts, approves finite shared fixed-PA consumer code+test+docs LOCAL
checkpoint with no new must-fix. Approval excludes push/main merge, full dual
or global completion and all explicitly deferred formatter/transition work.

## Custom change-filament lifecycle

Start clean0e039a63. Current unchanged full dual replay88488 exit100,
2.458s, /tmp/ares-custom-change-current-red.log, case-fqfXrG/case-JLy3IO.
First differences unchanged (baselineM73 placement/modifierremainingminutes).
Ranked falsifiable hypotheses: template unconsumed (no injected marks),
wrong dynamic context (marks but wrong IDs/counter), or state/command
suppression (correct marks but duplicate T/incorrect Z at new start).

Public native-first child tracer tool_templates/change.rs injects a minimal
custom comment with previous/next IDs, runtime count, layer and relative-E;
one whitespace-prefixed T[next_extruder] and G1 Z{toolchange_z+2}. Compare
all17 native blocks (initial+16printing), first dynamic records, first new
start Z2.2, exactly1 T per block and fan-resume flag. Unchanged complete
comparator still runs. Red build45726 running; obtain valid native-first RED
before production. Same previously approved public 3MF→slice byte seam.

Source boundary set_extruder7749/7858–7980/7991–7997,
custom_gcode_changes_tool241–264, retract7652–7704/eager_lift652–681.
Port coupled context/render/select/startup consumer, not a counter-only
checkpoint. Runtime count owned by generator state; initial included.
Reuse live template owner and shared writer tool selection. Required eager
retract executes before optional render even for empty/manual-skip templates.
Custom supplied command suppresses emitted writer string but not writer
installation/reset. Last custom Z propagates to next filament-start.
SEMM purge/tower, ooze temperatures and remaining output timing stay open;
do not claim original full dual closure until both complete outputs pass.

Initial45726 build and76242 replay both exit101 missing child module;
fixed path attribute before any production change. Build36143 exit0.
Replay14735 exit1001.854s is native expectation failure, NOT behavioral
RED: machine-start selectsT0, so source previous_extruder is0 rather than-1
(GCode.cpp3141/7817, GCodeProcessor.cpp::get_gcode_last_filament2932).
Correct native literal accordingly; build50925 exit0. Replay80611 exit100
1.866s also NOT valid RED: final body cooling consumes the internal
FORCE_RESUME marker, whereas initial marker survives. Assert initial marker,
compare remaining native flags rather than requiring internal markers in
all final body blocks. Rebuild in progress; production remains untouched.
These oracle calibrations affect only the new diagnostic, never original
full comparator/fixture. Implement machine-start last-filament tracking as
part of the cited context lifecycle; don't hardcodeinitial0.

Corrected diagnostic build53021 exit0. Replay93624 terminated/reaped exit100,
1.836s, /tmp/ares-custom-change-red4.log and case-mVXdy7: all17 native block,
ID/count/layer/start-Z and single-T checks pass; Ares blocks are empty.
This is the valid behavior RED. Then implement shared initial/body lifecycle,
runtime state count/start-filament, dynamic common fields, live render/write-
back and custom-command suppression. Required injected eager SpiralLift
reuses motion's retract/lift; split spiral output function into a child mod
to avoid growing lift.rs beyond400. Existing eager callers retain previous
hop-type dispatch; the custom call forces source SpiralLift. Build64423
running after fmt. No GREEN/complete dual claim yet.

Implementation patch retries (duplicate target/order mismatch) failed before
applying the combined eager-call patch; spiral mechanical extraction was
already applied in its preceding successful patch. Corrected patch completed
before build64423 began. No compilation failure counts as behavioral RED.
Coverage remains only public common identity/layer/command/Z lifecycle:
purge-volume dynamic fields/SEMM state, all other dynamic values, eager
trajectory controls and manual-first-skip need their own source/output
evidence before broad transition claims. Full original acceptance stays open.

Build64423 exited0 (1m17s). Probe62814 exited100 (1.885s): all17
custom contexts/counts/single-T/new-start-Z match, but Ares retains every
body FORCE_RESUME_FAN marker while native consumes them. This is still RED,
not a complete lifecycle checkpoint. Static review required ordinary (not
toolchange) injected wipe/retract, missing-map hardware0, clearing eager
zero-target lift/pending state, and source signed-u32 custom T scanning;
fixed all four before build4912, which exited0 (1m17s). No running build
was edited. Reviewer readback confirms those fixes, not runtime acceptance.

Fan consumer boundary: CoolingBuffer.cpp543,890–898,1000–1005. Resolve
force events in the printing-layer cooling flush in the same ordered pass
as role requests; retain baseline vs current effective speed and auxiliary
resend. Existing priority-normalized role markers are a compatibility shell
for active requests. Initial custom event remains outside CoolingBuffer and
must survive. Do not strip globally or weaken the native-first flag checks.

Independent source review gives a concrete remaining wider-domain gap: an
Overhang request from toolA can remain active after switching to toolB whose
control is disabled; source END only clears when control is enabled, unlike
the existing normalized BASE shell. SupportInterface/Ironing identities also
remain absent. Explicitly defer these source consumers, not their eventual
acceptance: the finite same-fan-configuration tracer cannot prove them.

Fan build92120 exit0. Official public replay38625 exit0,1.880s,
/tmp/ares-custom-change-fan.log: all17 native/actual contexts, single-T,
new-start-Z and retained-startup/consumed-body event flags match. Neither the
original comparator nor its input/assertions was relaxed. Begin retained
official/seeded controls and quality/full-workspace gates; original dual
complete output and global printer/options closure remain open.

Official broad19185 exited100 after default fail-fast:7PASS/1FAIL,
48unrun; retained full dual baseline byte1726/line124 and modifier byte718/
line25 remain RED. This is not a completed broad receipt. Clippy74470
exit0 (1m19s) had254lib/270test, one more than baseline due the new
DoubleEndedIterator::last scanner; WASM60406 exit0 (1m36s, compile only).
After both terminal, replace last with next_back, preserving last-valid
selection without unnecessary iteration. Re-run final gates, and use
--no-fail-fast for both broad references so known RED does not hide controls.

Final frozen-source receipts: fmt/diff check53374 exit0; clippy44978 exit0,
1m32s, /tmp/ares-custom-change-final-clippy.log, unchanged253lib/269test
and CLI31 warnings. WASM9055 exit0,17.57s,
/tmp/ares-custom-change-final-wasm.log (core+adapter compile only).
Prebuild70605 exit0: workspace4m14s (includes lock wait), then exact CLI
1m16s; no CPU0 compilation during the final replays.

Official99476 exited100 after all56 tests (132.102s), seeded93400 exited100
after all56 (179.427s), /tmp/ares-custom-change-final-official.log and
/tmp/ares-custom-change-final-seeded.log. Each:55PASS/1FAIL, with47 strict
complete-output controls and8 limited diagnostics passing. The new custom
17-block tracer passes1.874s/2.427s. Only inherited_filament's original
complete dual comparator fails: baseline byte1726/line124, modifier
byte718/line25, unchanged first-difference positions. All outputs/artifacts
retained; do not call these whole-suite reference GREEN results.

Final workspace26803 ran alone after both reference processes terminated:
exit0, /tmp/ares-custom-change-final-workspace.log,7563PASS (36slow),9skip,
256.857s. Spiral-vase ramp regression is PASS. All changed/new functional
Rust modules are below400LOC. No original full-output assertion, fixture or
warning suppression changed. Request final independent six-axis approval
for this coupled common-template LOCAL code+test+docs checkpoint only.
Full dual output, differing-tool fan/request identities, SEMM/tower/ooze,
all-printer and complete option-domain acceptance remain open; no push/main
or global milestone completion is authorized by these finite receipts.

Independent final six-axis review read the actual terminal receipts and
approved this finite coupled custom-lifecycle code+test+docs LOCAL checkpoint
with no new must-fix. Approval covers common context/render/write-back,
writer selection, eager and same-fan-configuration cooling consumer only.
Differing-tool request identity, SEMM/tower/ooze, complete dual and global
printer/option domains remain open; no push/main/global completion approval.
