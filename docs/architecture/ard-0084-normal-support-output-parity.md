# ARD0084 — Normal support complete-output parity

Status: public regression RED witnessed; support flow/gap prerequisites in progress;
ordinary-support generation and merging remain pending.

## Contact-pattern and connection-state checkpoint

The contact selector (`SupportParameters.hpp:134-144`) and native
`FillConcentric`/`FillGrid` call contracts are now wired into raft contact
plans. `/tmp/ares-normal-support-ard84-contact-enums.log` is terminal100
(9.836s): eight of nine complete-output cases PASS, including all five
effective `support_interface_pattern` values, zero-top-gap,
first-layer-support-width and synchronized-gap-rounding. Automatic-width
case-jwFwAY remains DIVERGENT at line479. Each case retains its full input,
native/Ares output and manifest; runner-cache producer identity is still
unknown. These are bounded active raft-contact checks, not ordinary-support
or complete numeric-domain coverage.

Read-only native memory captures in `/tmp/ares-ard84-fill-probe-3.log`
establish that all four `FillSupportBase::fill_surface` calls agree with
Rust's spacing, angle and density. The first three native path prefixes
also agree exactly. The contact call returns two native paths versus one
Rust path. The unmodified-process read-only capture's fresh native output
at `/tmp/ares-ard84-fill-probe-output-3/plate_1.gcode` also equals the saved
case-5xeRha native stream except generator (`cmp` terminal0); this does not
retroactively establish the saved cache's producer identity.
`FillBase.cpp:2464-2466` declares `cost_low`, `cost_high` and
`cost_veryhigh` as function-local `static const double`: they retain the
first call's `line_spacing` rather than using the current call's value.
The captured costs stay515781.6944064484,793510.2990868436 and
1190265.4486302654 across all four calls. Rust currently recomputes them
in `fill/base_support/link.rs` and `caps.rs`.

A deliberately modified-process diagnostic overwrites only those three
costs before subsequent native fill calls to reproduce Rust's current
per-call calculation. Its full output equals the saved Ares output after
removing the generator line. Script/log/output are
`/tmp/ares-ard84-fill-cost-counterfactual.gdb`, its sibling `.log`, and
`/tmp/ares-ard84-fill-cost-counterfactual-output/plate_1.gcode`.
This is causal diagnostic evidence, **not a reference or parity GREEN**;
the immutable native executable was not edited and the canonical original
reference remains divergent. No production diagnostic prints remain.

The source-cited repair must preserve these three first-call costs in
private task-local `SlicerContext` state, initialized at the actual native
selection phase. Do not freeze per-call `min_arch_length`, trim lengths or
cap costs, and do not approximate startup costs from configuration. All
actual support connection calls in one job must share the state, including
all objects. Native process-global lifetime is mapped to one Ares job
because the accepted reference workflow launches a fresh native process
per input; this does not claim equivalence for native long-lived jobs or
arbitrary parallel first-call scheduling. Verify A→B→A task isolation at
the complete-output seam. Independent read-only review endorses this
state boundary conditionally, not completion. Ordinary-support generation
and its full-output regression remain unresolved.

The contact-pattern snapshot passes `cargo fmt --check`, `git diff --check`
and workspace/all-target Clippy (terminal0,2m05s including shared build-lock
wait, existing warnings) in
`/tmp/ares-normal-support-ard84-contact-clippy.log`. Its default-gated full
workspace Nextest run remains in flight at this checkpoint; do not inherit
the earlier prerequisite workspace result as proof for these new fillers.
That contact-only run subsequently finishes terminal0:7462 PASS/9 skip,
256.594s. It predates the cost-lifetime production repair below.

### Task-local cost-lifetime repair

`SlicerContext` now owns a private optional triple of first-call costs.
The same mutable Context crosses the public slicing pipeline into skirt
generation and raft schedule materialization, then each actual support
connection. Initialization occurs after vertical-arch consumption at the
native cost selection phase. Arch selection uses cached low/high, both
T-joint directions use cached high and long arches use cached very-high;
per-call min-arch, trim and cap quantities remain unchanged. The first
implementation missed the T-joint consumer; self-review found it and the
final verification reruns include that correction.

`/tmp/ares-normal-support-ard84-all-cost-consumers.log` is terminal0
(28.625s under concurrent workspace load): all eleven full-output cases
PASS. This includes all five effective contact-pattern values, zero gap,
automatic width, first-layer support width, synchronized gap, verified
`skirt_loops=0` automatic width (case-TtOw0o), and exact-input task reuse
after different-spacing jobs. Initial automatic case-jcXLdK and repeated
case-jXFh55 reuse the same owned input/reference bytes; both saved 3MF and
full Ares streams are equal (`cmp` terminal0). Earlier repeated exports
differed in ZIP metadata and were replaced by this stronger replay.

The previous combined gated run
`/tmp/ares-normal-support-ard84-first-cost-isolation.log` is terminal100
(12.908s), dependencies PASS but ordinary support enabled case-Vaut6F
still DIVERGENT101/75; disabled case-jK377w PASS. That output snapshot
predates the T-joint correction and does not certify its final consumers.
No ordinary-support generator has been added. Multi-object native first-call
ordering and arbitrary parallel native scheduling remain unverified;
the bounded single-object/fresh-native-process contract is not a claim of
those behaviors. Final production cost-repair workspace is terminal0:
7462 PASS/9 skip,264.955s in
`/tmp/ares-normal-support-ard84-cost-workspace.log`. Final all-target Clippy
is terminal0 (1m39s, existing warnings) in
`/tmp/ares-normal-support-ard84-all-cost-clippy.log`; fmt/diff checks pass.
The independently enabled eleven-case output probe above covers the
strengthened no-skirt/exact-input tests separately from default gating.
Independent six-axis read-only review verifies all eleven manifest file
sizes/hashes and complete streams, closes the no-skirt and T-joint repair
items, and finds no remaining code blocker for this limited cost repair.
It does not approve ordinary-support completion. Actual builds succeed
for Linux44.58s, Windows1m27s and macOS2m08s (core library), and browser
WASM3m02s (release adapter), with logs
`/tmp/ares-normal-support-ard84-cost-{linux,windows,macos,wasm}.log`.
These are target builds, not foreign-OS runtime test claims.

Real Chromium executes six complete support/raft byte comparisons:
automatic width before/after a different-spacing job, first-layer width,
automatic width without skirt, concentric and grid contact patterns.
All PASS in `/tmp/ares-ard84-browser-support-generator-only.log`
(terminal0); each full input/native/browser output is retained under
`/tmp/ares-ard84-browser-support-generator-only/`. The before/after
streams are identical excluding the generator line. The initial temporary
driver mistakenly compared raw generator timestamps across browser jobs;
its terminal1 log and all outputs under `/tmp/ares-ard84-browser-support`
are retained. Raw diff proves only11:33:24 versus11:33:25 in that one
generator line; all six canonical native comparisons already passed.
Correcting this driver assertion changes no production code and does not
expand the approved generator-only exclusion. Ordinary-support exit
criteria remain unmet, so no task-completion approval or commit/push is
issued for ARD84.

The final production snapshot's explicitly enabled ordinary regression is
also rerun: `/tmp/ares-normal-support-ard84-all-cost-ordinary-red.log`
terminal100,3.542s. Disabled case-Zr0qSY is complete PASS; enabled
case-C8g8p6 remains DIVERGENT101/75 (264791/180867 generator-normalized
bytes). Both complete inputs/streams/manifests are retained. Thus the
final workspace GREEN and eleven prerequisite GREENs do not conceal or
replace the still-missing ordinary support generator/merge output.

## Non-negotiable decisions

Port the ordinary-support owner, not an Ares-designed support algorithm:
`PrintObject.cpp:850::generate_support_material` and
`:4289::_generate_support_material`,
`Support/SupportMaterial.cpp:374::PrintObjectSupportMaterial::generate`,
`Support/SupportLayer.hpp::SupportGeneratorLayer`,
`Support/SupportCommon.cpp` interface/layer/toolpath generation, and
`GCode.cpp::collect_layers_to_print(PrintObject/Print)`.
Rust destination is a source-cited `project_slice/support_material/` module
and the existing `project_slice/gcode_emit/layers/` scheduling/emission
boundary. Existing object-layer and raft-only scheduling is a temporary
compatibility shell, not a complete implementation of native support layers.
Do not repair the header count alone while omitting actual support paths.

Use the approved project-input/complete-byte-output seam. Acceptance remains
strict ordered bytes excluding only the generator line. No implementation
structure or source-text pinning, semantic fallback, tolerance, or passing
unimplemented classification replaces actual parity. Preserve full input,
native/Ares streams and failure artifacts before assertions.

## Evidence

The immutable8a52 option scan records three identical-input failures at
`/tmp/ares-options-8a52b2d2-initialized/case-{t83Wfc,hHpRdD,gXsngL}`:
`enable_support=true`, input SHA256
`2bbb896808e1b19b24999b769f69d80a1fc7247d4f40ba2ca0f43d4a206bc329`.
The first mismatch is the header count: native101 vs Ares75 layers.
Native contains actual `;TYPE:Support` paths. Cached reference producer
identity remains unknown in those manifests.

Two direct initialized native runs exit0 and reproduce that saved native
stream except generator, with full G-code under
`/tmp/ares-support-8a52b2d2-fresh/run-{0,1}/plate_1.gcode` and sibling
`/tmp/ares-support-8a52b2d2-fresh-retry-{0,1}.log` command logs.
They use seed0/CPU0 and the existing separately initialized replay binary.
The initial invocations failed before slicing because the output parent
directory was absent; those logs are not producer/replay evidence.

Effective fixture settings are normal(auto), grid style, independent
support height enabled, top/bottom gap0.15/0.2, top interface3 and bottom
interface-1. This is ordinary support, not TreeSupport. Native replay
is stable; seed control cannot manufacture missing Rust support output.

The new public both-Boolean regression witnesses the actual output RED in
5.591s (`/tmp/ares-normal-support-ard84-output-red.log`, terminal100).
Disabled support matches completely in case-QNJajG; enabled support diverges
at the same101/75 header in case-MTeLqU. Both cases retain input and native/
Ares streams plus manifests. Effective enable/type/style assertions pass,
and enabled native output contains positive relative-E support X/Y motion.
The initial test used a JSON Boolean in the literal application verifier,
which compares native serialized strings: false vs "0" failed before
slicing. That setup failure is not the witnessed output RED; the corrected
test requests/verifies native "0"/"1" serialization explicitly.

The enabled pair contains2415 native positive relative-E support X/Y G1
instructions and28 support role comments; Ares has zero of each. Production
search finds support entity construction only in the existing raft port,
and `gcode_emit/layers/schedule.rs` merges object layers plus that raft
schedule, not ordinary support layers. This excludes a header-only cause:
the ordinary `PrintObjectSupportMaterial::generate` owner is missing and
the support-aware merge must be ported after it. The new exported input
has SHA25649b6ead6a36d327d339db4ec44b271c35a563ac9318001da9627b4ca3c901da4;
do not conflate its archive identity with the earlier scan's input hash.

## Rewrite sequence and limits

### Dependency checkpoint

The immutable8a52 all-option run is terminal100 (3570.031s):386/650 bounded
domains complete,1661 generated,1519 strictly compared,1 rejected. These
are modified smoke-context results, not untouched-default or current-WIP
verification. The previous current-worktree workspace check is terminal0:
7461 tests pass,9 skip,255.614s, before the prerequisite production edits.

The strengthened public ordinary-support regression verifies all seven
effective context settings and collects both Boolean output results before
asserting. `/tmp/ares-normal-support-ard84-context-red.log` is terminal100
(4.828s); disabled case-qzJ9M2 is PASS and enabled case-asBa93 is DIVERGENT,
with an active positive-E support-motion witness.

Prerequisite source owners are `Flow.cpp:232-274` (base, first-layer and
interface support flows) and `Slicing.cpp:86-190` (asymmetric contact gaps,
support-nozzle layer-height limits and synchronized-gap rounding). Rust
destinations are `project_slice/perimeters/flow/support.rs`, reused by
`raft/bridge/project.rs`, and the existing `project_slice/parameters.rs`.
The first-layer width chain is initial width, then support width, then
generic width; automatic support width is one nozzle diameter, not the
perimeter's1.125 multiplier. Preserve native f32 flow storage and f64 gap
rounding with upstream EPSILON1e-4, not machine epsilon.

Public dependency probes retain complete raft output in separate modules.
The pre-edit run at `/tmp/ares-normal-support-ard84-dependencies-red` records
three byte divergences and a fourth native producer error. The WIP run at
`/tmp/ares-normal-support-ard84-dependencies-wip` makes
first-layer-support-width case-PTufju fully PASS. Zero-top-gap case-WkrwUH
now has the correct maximum Z but still diverges in commands/toolpaths;
automatic-width case-E9ESH2 still diverges after the raft. Synchronized-gap
rounding still fails in native slicing (exit156/native return-100), so it
has no output-parity credit. These incomplete probes cannot certify the
entire prerequisite boundary or close ordinary-support parity.

The rejected0.299999 synchronized-gap input's authoritative per-input
`result.json` reports an unprintable empty layer between1.1 and1.7mm
(native return-100), not a random-stream or exception failure. Preserve that
input and result as native-rejection evidence, not output parity. The
current probe instead uses0.099999 to exercise the same EPSILON-sensitive
rounding boundary with a smaller gap; its result must be established by
actual execution, not inferred from the equation. The rejected larger-gap
context and the broader numeric domain remain unverified for output parity.

The smaller-gap run is now witnessed: terminal100 overall (4.625s), with
all four native streams generated. Under
`/tmp/ares-normal-support-ard84-valid-gap`, first-layer-support-width
case-nUfYU7 and synchronized-gap-rounding case-Gs6sei are complete PASS,
including active relative-E support XY extrusion. Zero-top-gap case-30UCYS
and automatic-width case-iF2jB8 remain full-output DIVERGENT. Thus two
prerequisite cases are repaired, not four, and ordinary support remains RED.

Production-prerequisite workspace verification is terminal0:7462 pass,
9 skip,255.511s (`/tmp/ares-normal-support-ard84-prerequisites-workspace.log`).
The existing nozzle/profile test helper's old ordinary-gap placeholders0
were updated to the independently configured default0.2; no nozzle/layer
checks were removed. The later gated probe input change from0.299999 to
0.099999 does not alter production code and was separately executed above.
Workspace/all-target Clippy is terminal0 (45.76s), with existing warnings,
in `/tmp/ares-normal-support-ard84-prerequisites-final-clippy.log`.
The independently enabled ordinary-output run is still terminal100:
case-qsJXSC disabled PASS and case-F2O1Ir enabled DIVERGENT101/75 under
`/tmp/ares-normal-support-ard84-prerequisites-current`; preserve that RED
alongside the default-gated workspace GREEN. No commit/push or completion
approval has been issued for this WIP slice.

The source `SupportGridParams::expansion_to_propagate` is-3 lattice units
(`SupportMaterial.cpp:624-625`); do not substitute the tentative-5 value
from investigation. Actual ordinary-contact geometry remains to be ported.

Independent read-only prerequisite review found no new flow arithmetic or
nozzle-index error, but gave no completion approval. Before independent
support-height/contact generation consumes these parameters, retain native
`max_suport_layer_height` separately from the final object/nozzle-clamped
maximum, and preserve `zero_gap_interface_top/bottom/raft` source flags.
Their interface semantics cannot be inferred from a gap's numeric zero.
The zero-top probe's later paths also expose the missing native contact
fill selection: `SupportParameters::contact_fill_pattern` can select
concentric for zero-gap auto interfaces, while the raft shell always uses
the plain raft interface pattern. That owner needs its own actual output
repair; matching maximum Z alone is not a solution.

Follow `PrintObjectSupportMaterial::generate`'s actual dependencies:
buildplate-covered and top contacts; bottom contacts and projected support
areas; raft/intermediate layer heights and object trimming; base regions;
contact overlap trimming and interfaces; final support layers and paths;
then native object/support Z merge and emission. Keep each Rust file below
400LOC and tests in separate modules. Reuse existing geometry/fill/flow
ports only where their contracts match the cited upstream callers.

Include normal(auto)/grid on the existing overhang fixture and both Boolean
enable values first. Adjacent tree/organic support, explicit enforcers and
blockers, multi-material interfaces, broader styles, full numeric domains
and untouched printer defaults remain subsequent source-cited slices.
They remain required by the global goal; this bounded task cannot certify
all support options or all-printer/default completion.

Exit requires witnessed public RED/GREEN, actual support motion when enabled,
no support paths when disabled, paired full artifacts, shared consumers,
workspace Nextest, fmt/Clippy, Tier-1 evidence and independent six-axis
review/fix/re-review before code/docs commit and push.
