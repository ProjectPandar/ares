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
