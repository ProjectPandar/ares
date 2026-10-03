# Normal support execution history — ARD0084

Historical checkpoints, including superseded results, are retained here for
artifact provenance. The current architectural decisions are in
[ARD0084](../../architecture/ard-0084-normal-support-output-parity.md).
This history is not an acceptance or merge approval.

Status: public regression RED witnessed; support flow/gap prerequisites in progress;
ordinary-support generation, paired scheduling and emission are publicly active;
complete-output parity remains RED.

## Paired support emission slice

Port `GCode.hpp::LayerToPrint`, `GCode.cpp:1718-1892::collect_layers_to_print`
and `GCode.cpp:6178-6235::extrude_support` into
`gcode_emit/layers/schedule`, `layers/support_entry` and `motion/support`.
The schedule must retain optional object/support indices, pair at EPSILON,
average paired Z, and merge plate groups at their minimum/maximum midpoint.
Use retained planned object print Z rather than reconstructing a height sum.
Support-only layers must actually emit; header/count consumers must use this
same schedule. Global chunk numbering/previous Z are distinct from native
active-layer ids used by motion. Retain role, orientation, NoSort groups,
MultiPath joined wipe, and ironing-last dispatch. Do not convert support to
object infill. The existing raft adapter remains a temporary rewrite shell
only for raft-only objects not yet entering ordinary support generation;
never regenerate that shell for an object with installed support layers.
Tree support, painted brim inputs and multi-tool dispatch remain open adjacent
behavior, not completion claims. The agreed seam is the existing public
normal-support complete-output Boolean test; unchanged RED revalidation is
terminal100/2.059s, enabled case-diQtjg under layer-pair-red-output (101/75).

Source clarification before review repairs: `GCode.cpp:2513-2527` counts raw
object/support Z using exact unique then adjacent EPSILON differences, distinct
from min-anchored paired chunk grouping. Retain both native computations from
the same real layer inputs. `GCode.cpp:5339-5345,5386-5440` initializes the
planner with pair-selected layer geometry, while temporarily switching active
layer to support. Pair labels start only once. Port `GCode.cpp:7618-7632`
support-island travel containment as separate active-support geometry, not
planner geometry; preserve prior external/overhang-wall forced retraction.
MultiPath adaptive PA remains an explicit deferred behavior.

The first emitted-support complete output (terminal100/6.143s, case-RuYAxx)
has 101 layers and actual support paths, but differs first at M73 time and
then at skirt geometry. Extend the same public vertical slice with
`Print.cpp:2646-2694::_make_skirt`: collect actual support extrusion points
through the native global skirt-height Z, into `gcode_emit/skirt.rs`. The
installed support carrier replaces the raft reconstruction for active normal
support objects; the declared raft-only shell remains for inactive objects.

Precision re-review requires retaining scaled integer travel endpoints and
rounding only reconstructed routed intermediates. Global chunk index feeds
role-change layer_num, CoolingBuffer role fan scheduling and bottom-only hop;
active layer id continues to feed path kinematics. The new no-fail-fast run
also exposes a zero-height triangle metadata test entering an empty schedule:
do not panic or fabricate a first layer. `PrintObjectSlice.cpp:866-867` rejects
no detected layers; propagate that source behavior as an input error, leaving
the unchanged positive fixture regression visible for follow-up.

### Paired emission terminal checkpoint

All launched handles collected. Latest endpoint/precision/counter code:
Clippy terminal0/47.13s (255 library warnings, predominantly existing);
WASM check terminal0/52.56s, compile-only; fmt/diff checks pass. Paired public
run terminal100/6.155s, two FAIL: ordinary enabled101/101 now actually emits
support, but complete bytes first differ at byte514/line19 (M73 R21/R23).
Artifacts: `/tmp/ares-normal-support-ard84-layer-pair-end-output/case-z5ATqm`,
complete input/native/Ares/error/manifest retained. Ares raw266232 bytes,
SHA256 `5ca4f81eddf55f3cec8288089d1ddf29b31794300e614a807b3f91fce28194df`.
The manifest correctly keeps cached reference producer identity and Ares
revision unknown; this is not official-producer coverage proof. Raft dependency
cached references retain previously witnessed direction differences.

No-fail-fast workspace (pre-final precision/counter/error-return snapshot)
terminal100/265.526s,7462 run:7460 PASS2 FAIL9skip38slow. Failures are tree(auto)
header output and zero-height triangle metadata positive fixture. This does
not validate the final counter/error-return changes. Preserve both public
inputs/assertions; obtain same-input native zero-layer evidence before changing
that fixture's expectations. Independent read-only re-review closes labels,
planner/active geometry, retained integer travel endpoints and counter split;
does not approve completion/commit. No source/helper pinning tests added.

Next source boundary: `GCode/ToolOrdering.cpp:137-148::calc_max_layer_height`
and `1407-1448::mark_skirt_layers` into `gcode_emit/layers/skirt_schedule`.
The complete output reveals an extra skirt at support-only Z0.55; implement
native object/extrusion flags, max-layer-height gap marking and skirt_done Z
height differences, not a blanket support-layer skip. Tree behavior,
MultiPath adaptive PA, painted brim inputs, independent native reproducible
reference provenance and all-printer/all-option inventory remain open.

LOC gate: paired schedule integration grows gcode_emit.rs to413 lines. Move
its existing ProcessorLimits config assembly verbatim into the owning
`processor/envelope.rs` bridge (`GCodeProcessor.cpp::apply_config`), with
unchanged option operands and no new behavior. Recheck lint/WASM/complete
output after this required functional module split.

Split validation collected: Clippy88990 terminal0/48.32s255library warnings;
WASM87497 terminal0/53.27s compile-only; paired14803 terminal100/6.223s,
ordinary enabled case-4iWqs4 under layer-pair-split-output. `cmp` verifies its
complete Ares bytes equal pre-split case-z5ATqm exactly. Independent read-only
review confirms all moved config operands/fallbacks and call position unchanged.
Fmt41284 terminal0 and diff checks pass; every modified/new Rust source is
under400 (main359, processor/envelope138, skirt395, motion/state399).
All current handles are terminal. Full workspace has not been rerun after
precision/counter/error-return/split; scoped output remains RED. No commit/push.

## Native input adapter review checkpoint

Public activation boundary: `PrintObject.cpp::generate_support_material`
calls the support generator after object fills are available. Ares will call
the native-input adapter immediately after `fill_entities::prepare`, before
object entities move into extrusion islands, and move the generated native
SupportLayer carrier into `PreparedPostIslandPrintOrder`. On generation
failure, use the existing explicit predecessor disposal path rather than
recursively dropping the boxed stage chain. This is retained production
state for the upcoming `GCode.cpp::collect_layers_to_print` and
`GCode::extrude_support` consumers, not a helper-only computation whose
results are discarded. Activation alone does not complete G-code scheduling.

The first actual generation run exposes support-grid overflow on empty
contact polygons (`SupportGridPattern::new`, public Boolean test). Native
`Polygon.cpp::get_extents(Polygons)` returns the zero-initialized undefined
`BoundingBoxBase` for an empty collection, whereas Rust's old bounds helper
retains integer MAX/MIN sentinels. Preserve native undefined-box semantics
(`BoundingBox.hpp:21,35`, `BoundingBox.cpp::align_to_grid`) rather than
saturating arithmetic or rejecting valid empty internal geometry. The same
public complete-output regression remains the verification seam.

The public repro repeats the overflow twice in 1.718s/1.99s. Preserving
native zero bounds removes that panic (2.031s public run reaches the
existing101/75 output mismatch). Complete the same source boundary by
aligning only a defined box's minimum; upstream leaves its maximum
unchanged (`BoundingBox.cpp:248-255`). Empty bounds remain explicitly
undefined, so even minimum alignment is skipped after the20-unit expansion.
Do not replace these semantics with general overflow guards.

Before scheduling, the public optimization phase must consume the retained
support paths as `Layer::simplify_support_entity_collection/path/multi_path`
(`Layer.cpp:329-375`, `PrintObject.cpp:951-970`). Preserve Path, NoSort and
MultiPath grouping. Arc fitting uses the upstream0.0375mm support resolution
(`libslic3r.h:81-82`), while non-arc simplification uses configured printer
resolution. Reuse the existing Polyline3 simplification machinery by
borrowing the actual path buffer; do not clone or relabel support as walls.

The make-fills input adapter now preserves `Flow::scaled_width()`'s integer
truncation before float conversion for external and support widths
(`SupportMaterial.cpp:1240,2441`). Its bridging predicate checks only loops
and returns the point-count result for the first nonempty overhang path,
including the closed-path distinction (`SupportMaterial.cpp:1127-1147`).
Bottom contact height is read from the actual bottom interface Flow:
`with_flow_ratio` for the zero-gap/non-thick branch and float-narrowed
`Flow::bridging_flow` otherwise (`SupportParameters.hpp:95-97`). The existing
Rust Flow implementation is reused rather than duplicating its cross-section
calculation. Independent read-only re-review closes these three static defects.

Painted brim remains an explicit input coverage gap. Native
`Format/bbs_3mf.cpp::_extract_brim_ear_points_from_archive` reads
`Metadata/brim_ear_points.txt` into `ModelObject::brim_points`; the current
Ares project loader does not retain these points. The adapter's current
`has_brim` predicate must not be described as a complete port of
`PrintObject::has_brim`, especially for Painted with zero configured width.
Real model-state ingestion and output validation remain required.

The repair-only snapshot below predates public generation activation.
Static review closure and successful compilation do not prove ordinary-support
output parity; no completion or commit approval is implied.

Repair-snapshot verification: workspace/all-target Clippy finishes terminal0
(1m12s, existing warnings), and rustfmt/diff checks pass. The paired public
output run `/tmp/ares-normal-support-ard84-review-fixes-parity.log` finishes
terminal100/17.308s: raft dependencies PASS/17.302s, ordinary support
FAIL/5.059s at byte88/line3 (101 versus75 layers). Complete streams, inputs,
errors and manifests are retained under
`/tmp/ares-normal-support-ard84-review-fixes-output`; enabled case-yNpdHb
remains DIVERGENT. Cached native producer identity remains unproven even
though this invocation explicitly selects the official extraction path.
The adapter was uncalled in that snapshot, so this is not runtime validation
of its repairs.
WASM compile check finishes terminal0/3m15s including build-lock wait; it
does not establish browser runtime parity. Full workspace Nextest is still
running at this checkpoint; do not substitute an earlier snapshot's results.

That repair-only workspace subsequently finishes terminal0/257.641s:
7462 PASS,9 skip,35 slow. It still predates public generation activation.

### Public generation and optimization checkpoint

The public pipeline now calls the adapter after make-fills, moves all native
SupportLayer carriers into the ordered object stage, and simplifies their
actual paths before G-code export. Generation failure uses explicit fill
stage disposal. The stale whole-module dead-code staging expectation was
removed. Independent read-only re-review finds no new deterministic defects
in activation, native empty/min-only bbox semantics or support simplification.
Support G-code scheduling and emission are still unconnected; preserving
generated entities is not completion or parity GREEN.

The initial public run is terminal100/6.055s: ordinary support panics on empty
bounds, and one raft no-skirt reference diverges. Public empty-bounds repair
reruns no longer panic; the native-bounds snapshot's paired run finishes
terminal100/6.135s, ordinary101/75 RED and a raft reference divergence.
The no-skirt Ares files in case-XUps3q (public-generation output root) and
case-d3OTCm (review-fixes root) are byte-identical (`cmp` terminal0), while
native references differ; do not attribute that failure to activation.

Final public-simplification snapshot: Clippy terminal0/47.72s (256 existing
or newly exposed warnings, no new blanket suppression), WASM compile
terminal0/2m16s including build-lock wait. Default-gated full workspace and
paired public output checks were launched as handles58652 and45481. All logs share
`/tmp/ares-normal-support-ard84-public-simplification-` prefix. No source
pinning/private-helper tests were introduced and no commit is approved.

Both handles subsequently finish terminal100. Workspace:96.689s,
5697/7462 run,5696 PASS/1 FAIL/9 skip/1765 not run after fail-fast. The
unchanged public header-width test enables support on the KSR project,
whose imported `support_type` is `tree(auto)` and style `default`; activation
now correctly exposes the unported tree generator as
`UnsupportedProjectFeature("support_type/support_style")`. This is a real
regression/open requirement, not full workspace GREEN. Do not rewrite its
fixture to normal/grid or weaken its output assertions to conceal the gap.
Tree generation remains required by the all-options objective.

Paired run:6.170s, ordinary FAIL/2.041s at101/75 byte88/line3 without the
former overflow; raft dependency FAIL/6.169s with automatic-width reference
ordering differences. Inputs/full output/error/manifests are under
`/tmp/ares-normal-support-ard84-public-simplification-output`, ordinary
enabled case-Ipb0SR. Generator execution/retention/optimization now have
actual public runtime coverage, but native geometry/path equality and
support motion emission remain unproven. No completion or push is warranted.

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

Uncommitted contact rewrite staging after dependency commit d364da4f:
`project_slice/support_material/{contact_layer,contact_placement,contact_polygons}.rs`
now contains the source TopContact data subset, top-gap synchronization,
normal/zero-gap/raft/thick-bridge contact placement, conservative close-layer
merging, and the grid contact projection/dense-interface polygon branch.
These are not yet consumed by the ordinary generator or G-code emitter.
They have no behavior-completion credit and must not be committed as a
completed normal-support task. The existing public output seam is the
regression gate; do not add internal structure/source pinning assertions.
Current HEAD full-output replay is terminal RED101/75 (3.526s), with disabled
PASS and enabled native support extrusion witnessed, retaining complete
artifacts in `/tmp/ares-normal-support-d364da4f-red`. Grid rotation/snug
behavior, overhang annotations, height assignment, base/interface/path
generation and final scheduling are still required before scoped GREEN.
Early independent read-only review caught the missing `layer_id > 0`
predicate on the enforcer grid-union branch; staging now retains it while
still saving the original enforcer polygons unconditionally when nonempty.
Independent static re-review confirms that predicate fix and the contact
placement early-return/bridge-guard flattening preserve the cited finite
source arithmetic. Fmt/diff checks pass; staged core Clippy is terminal0
(54.78s, existing252 warnings) in
`/tmp/ares-normal-support-ard84-contact-clippy-3.log`. None of these checks
prove the still-unconnected contact rewrite produces correct full output.

Further uncommitted staging adds `detect_contacts`, `layer_input`,
`overhangs` and the grid-only serial construction partition `top_contacts`.
The sources remain `SupportMaterial.cpp::detect_contacts`,
`buildplate_covered`, the region/first-layer/cantilever substeps of
`detect_overhangs`, and the final serial half of `top_contact_layers`.
`ExPolygon.cpp:155-184` supplies overlap semantics: boundary-polyline
intersection plus reverse containment, not merely positive-area overlap.
`MultiPoint.hpp::distance_to` measures nearest vertex, not nearest edge.
Independent early review identified the degree conversion arithmetic;
staging now preserves `Geometry.hpp::deg2rad`'s `PI * angle / 180.0`.
It also confirms that the namespace-scope sharp-tail length/area thresholds
are initialized at the native default1e-6 scale and do not change with
later printer scale changes. Local `scale_` operations still use the
current scale by division; no CLI large-bed output verification is claimed.
Core Clippy is terminal0 (14.24s, existing252 warnings), fmt/diff pass;
the earlier contact-staging workspace is terminal0,7462 PASS/9 skip,
256.990s, not verification of these later additions.

The public normal-support gate for the later pre-threshold-fix compilation
snapshot is terminal100/3.515s under
`/tmp/ares-normal-support-ard84-top-contact-wip-red`: case-4pVjcx disabled
PASS, case-1Ian85 enabled DIVERGENT101/75, complete artifacts retained.
Static re-review closes the staged arithmetic and serial-contact items;
it does not approve a commit or behavior completion. Full per-object-layer
overhang arrays must retain empty layer slots. Blockers, bridge removal,
thin-island filtering, serial sharp-tail propagation and cluster removal
are now staged in the source-owned computation chain, but actual generator
entry/height/path/scheduling integration remains open.

Further uncommitted staging composes the full grid-only `top_contact_layers`
outer chain and adds the `detect_bottom_contacts`, bottom-gap synchronization,
`project_support_to_grid` and reverse bottom-contact projection partitions
from `SupportMaterial.cpp:1679-1719,2380-2730`. Bottom-interface negative
layer counts inherit top-interface counts; asymmetric zero-gap detection
is not inferred from the rounded numeric gap. Bottom contact snapping
preserves the strict minimum-height comparisons, and scale-dependent
epsilon offsets are divided by the current coordinate factor. Independent
review caught an unconditional upper-layer access in the nonzero-gap
branch; it now occurs only in the native zero-gap branch. Static re-review
closes that item and confirms projection ordering and consumption semantics.
Projection reuses the existing `Polygon.cpp::remove_sticks` computation;
no internal pinning tests are added. The reverse projection deliberately
does not claim the final `trim_support_layers_by_object` operation yet.
Blocker-induced sharp-tail polygon/height correspondence still needs real
input/output evidence; indexed upstream traversal is not silently zipped.

Latest public full-output replay, including this reverse-projection staging,
is terminal100/3.518s at
`/tmp/ares-normal-support-ard84-bottom-projection-wip-red`: disabled
case-8ShYMx PASS, enabled case-EiFGfo DIVERGENT101/75. Complete project,
native/Ares G-code, error and manifest artifacts are retained. These unused
computations cannot earn behavior credit from static review or this RED.
All new Rust modules remain below400 lines; no commit is approved until
actual scoped support output passes the public seam and completion gates.
Final core Clippy is terminal0/14.08s with the existing252 warnings at
`/tmp/ares-normal-support-ard84-bottom-projection-clippy-final.log`;
the initial reverse projection introduced two nesting warnings, now
removed by flattening the enforcer-area merge and independently reverified.
Fmt and diff checks pass. That bottom-projection workspace snapshot is
terminal0/253.422s,7462 PASS/9 skip at
`/tmp/ares-normal-support-ard84-bottom-projection-workspace.log`.
It does not cover the later object-trim, height and base-layer additions.

Further uncommitted staging ports `SupportMaterial.cpp:1174-1215` bridging
perimeter areas and `:3106-3205` object trimming, including the distinction
between `bottom_print_z()` and stored `bottom_z`, original Point3 closure,
Square/open-butt offsets and double constants initialized from `0.2f`.
The bottom-contact chain now includes its final object-trim operation.
Sharp-tail state is caller-owned, retained across top detection/propagation
and subsequent object trimming, not discarded at the top-contact return.
`Slicing.cpp:117-133`/`Slicing.hpp:73` supply saved zero-gap flags and the
separate support maximum height: capture it after the support nozzle limits
but before object nozzle limits and regular-height clamping.

`SupportMaterial.cpp:2734-3103` supplies top/bottom contact trimming,
raft/intermediate height allocation and base-layer polygon construction.
`SupportCommon.hpp:116-139` supplies descending cached layer lookup; keep
its original direction and source caller ordering, not a differently
ordered search. Independent heights retain EPSILON-biased ceil, dynamic
top-contact extremes and exact final-Z alignment; synchronized heights
follow the actual object layers. Base conversion explicitly changes the
source role from Intermediate to Base after polygon assignment. These
remain unconnected computations, not a public-output completion checkpoint.
Independent static review reverified the numeric boundaries, role change
and flattening of all new nesting warnings. Final `cargo clippy` is
terminal0/20.98s, existing252 core warnings, in
`/tmp/ares-normal-support-ard84-base-layers-clippy-final.log`; fmt/diff pass.
The 25 selected preexisting parameter/profile tests pass0.196s. The public
full-output dependency gate passes11 contexts13.033s, but enabled ordinary
support remains RED101/75,5.368s, in the combined terminal100/13.034s run
`/tmp/ares-normal-support-ard84-base-layers-wip-output`. Full paired files
and provenance manifests are retained; cached reference producer identity
remains unknown. That base-layer snapshot's workspace Nextest completed
terminal0/256.958s,7462 PASS/9 skip at
`/tmp/ares-normal-support-ard84-base-layers-workspace.log`. It does not
cover subsequent interface allocation or generator composition.

Further staging rewrites `SupportCommon.cpp:47-241` grid interface
allocation and `SupportCommon.hpp:79-105` ascending cached lookup.
Preserve empty base slots in the layer-count windows, independently
regularize top/bottom projections, and subtract interfaces from base
before trimming transitional base-interface polygons. Interface counts
retain negative bottom-count inheritance and separate zero-gap/soluble
material rules; these branches have no active public-output proof yet.
`SupportMaterial.cpp:374-486` now supplies a composed geometry prefix,
retaining shared sharp tails through detection and trimming. Its annotation
inputs must already contain real sliced support volumes and projected
custom facets; blockers are unioned and expanded by float(1000*SCALED_EPSILON)
as in `:1326-1344`. Buildplate coverage is gated by has_support AND
buildplate_only. Independent static review found no required correction;
Clippy completed14.87s with existing252 warnings. This prefix still has no
public pipeline caller and therefore receives no behavior completion credit.

`SupportCommon.cpp::generate_raft_base:288-312,364-402` supplies the staged
non-multilayer-raft first-layer support expansion. Preserve first-slot
selection by bottom_print_z, float expansion steps, default Miter/3 column
expansion, Square/0 object/interface offsets and brim subtraction from all
four eligible layer kinds. Brim generation, the multilayer-raft branch,
actual generator inputs, toolpaths and G-code layer scheduling remain open.
This function is not yet connected. Clippy completed14.64s with existing252
warnings; no new private/source-pinning tests were introduced.
Independent static review found no required correction in that branch;
its caller must pass the source brim trimming area and enter only for
raft_layers<=1. The latest combined public-output run is terminal100,
12.978s (dependency11 contexts PASS, ordinary-support boolean gate FAIL
5.397s). Full paired artifacts are retained at
`/tmp/ares-normal-support-ard84-grid-prefix-wip-output`; enabled case
`case-w5OXq2` differs at byte88/line3, native101 versus Ares75 layers,
normalized264791 versus180867 bytes. Reference cache producer identity
remains unknown. This rerun demonstrates preservation of completed
dependencies, not support behavior credit for the unconnected code.

The following staging now completes the `generate_raft_base` geometry
boundary (`SupportCommon.cpp:244-402`) and composes it after interfaces.
Brim trimming uses source contour/holes and Round tolerance, not the area
covered by emitted brim paths. `has_brim` must come from PrintObject/model
state, including painted brim points. Raft<=1 enters the reviewed first-layer
branch; larger rafts preserve base/interface allocation and optional column
contact polygons. The first layer does not receive contact polygons; later
raft interface layers do. The raft contact layer is not allocated twice.
Support first-layer gap and integer Flow::scaled_width inputs remain actual
adapter contracts, not fabricated values. Independent read-only review
found no required logic correction. The first compile exposed a returning
reference closure lifetime error; replacing that closure with ordered
iterators fixes it without changing geometry. Two new brim nesting warnings
were flattened. Final Clippy is terminal0/21.32s, existing252 warnings, at
`/tmp/ares-normal-support-ard84-adhesion-clippy-final.log`. New files remain
under400 lines. No public generator caller or toolpaths exist yet, so no
ordinary-support completion or commit is approved.
The prior grid-prefix workspace finished terminal0/258.772s,7462 PASS/9 skip
at `/tmp/ares-normal-support-ard84-grid-prefix-workspace.log`; its compile
snapshot excludes these later adhesion additions.
Latest adhesion-tree combined full-output regression is terminal100/13.000s
at `/tmp/ares-normal-support-ard84-adhesion-wip-output.log`: dependency11
contexts PASS12.999s, ordinary-support boolean gate FAIL5.383s. Paired
complete files and manifest are in
`/tmp/ares-normal-support-ard84-adhesion-wip-output`; no geometry-only
success replaces the missing public-output GREEN. Fmt/diff checks pass.
The adhesion-tree full workspace run has started separately at
`/tmp/ares-normal-support-ard84-adhesion-workspace.log`; its own terminal
result is required and must not be inferred from the preceding snapshot.
That adhesion snapshot has now completed terminal0/257.302s,7462 PASS/9 skip;
it excludes the following later layer-installation changes.

`SupportCommon.cpp:1331-1411` and `SupportLayer.hpp::operator<` now supply
support-layer installation at the end of the geometry composition. Keep
raising Z/decreasing height/bridging-first sort, fixed first-Z+EPSILON
groups, endpoint midpoint (not arithmetic mean), minimum height including
empty source slots, and Z updates to every grouped source layer. Install
only nonempty groups. Interface counting includes the base-interface
membership rule; contacts-only groups do not advance the global interface
counter. Native reverse lookup has no break, so the oldest matching
installed layer determines its interface ID.
Rust borrows each independently owned source once instead of duplicating
native pointers. Independent review initially flagged possible equal-key
source-list ordering exposure. `SupportMaterial.cpp:523` discards that list
in release builds; toolpaths use the original arrays and cached Z lookup
(`SupportCommon.cpp:1590-1630`). Remove the unused source tags/list rather
than introducing an artificial ordering rule. Static re-review found no
required correction in the resulting installation-only boundary. Final
Clippy is terminal0/20.40s, existing252 warnings, at
`/tmp/ares-normal-support-ard84-layer-install-clippy-2.log`. The initial two
nesting warnings were flattened; fmt/diff pass. The new module is152 lines.
No public caller or actual support toolpaths exist yet; none of this static
evidence proves ordinary-support output or authorizes a completion commit.
The installation-tree combined public-output run is terminal100/13.000s:
dependency11 contexts PASS12.999s, ordinary-support boolean FAIL5.395s.
Full paired artifacts are in
`/tmp/ares-normal-support-ard84-layer-install-wip-output`, enabled
`case-bPoHOA`; ordered bytes still differ at byte88/line3 (101/75 layers,
normalized264791/180867 bytes). Cached reference producer identity remains
unknown. The installation-tree workspace has started at
`/tmp/ares-normal-support-ard84-layer-install-workspace.log`; collect its
own terminal result rather than reusing the adhesion snapshot.
The installation snapshot has now completed terminal0/260.692s,7462 PASS/
9 skip. It excludes the following toolpath-parameter changes.

The next staging slice rewrites `SupportParameters.hpp:64-144` parameter
selection for actual toolpaths. Save the original raft interface Flow
before replacing ordinary interface Flow with base when top count is zero;
split top/bottom densities and preserve ironing's top-spacing override.
Use `Geometry.hpp:301-305` PI(float)*angle/180(float) order. Do not merge
the ordinary contact pattern selector with the existing raft-contact
top=0 override. Reuse the existing FillPattern subset (currently named
RaftFillPattern) through parent-only module visibility, not a separate
Ares-designed support pipeline or a duplicate enumeration.
`PrintObject.cpp:3784-3804` returns zero-based object extruders, but the
native can_merge_support_regions branch compares its sole value directly
to max configured selectors. Preserve that behavior and supply the true
deduplicated object extruder set. Do not silently normalize this comparison.
Bottom bridge Flow, ironing Flow, raft parameter completion and region
cache merging remain deferred; this is not the full SupportParameters
constructor and is not yet consumed by actual support paths.
Independent review found one compile visibility error (E0603), fixed with
parent-only access to the already-shared pattern enum. Final Clippy is
terminal0/14.65s, existing252 warnings, in
`/tmp/ares-normal-support-ard84-toolpath-params-clippy-final.log`; fmt/diff
pass. New module102 lines. Public paired output remains terminal100/
13.059s, dependencies PASS and ordinary-support boolean FAIL5.386s at
`/tmp/ares-normal-support-ard84-toolpath-params-wip-output`. This does not
give behavior credit to the unconnected parameter slice. Its own workspace
run has started at `/tmp/ares-normal-support-ard84-toolpath-params-workspace.log`.
That parameter snapshot finished terminal0/257.916s,7462 PASS/9 skip;
it excludes the following cache additions.

`SupportCommon.cpp:755-830` supplies the staged extruded-layer cache:
source polygons determine empty(), while an optional remaining-to-extrude
vector overrides its fill region. Preserve None versus Some(empty).
Merging performs remaining safety union, ordered extrusion moves, then
source polygon safety union and source clearing. Cache absence is an
outer Option; a populated cache always identifies a real source layer.
Source indices refer to the same job's stable vectors; no whole geometry
copy or retained mutable pointer is required. Cached move assignment must
use Option::take() to leave its source absent, matching native pointer reset.
The entity payload is generic and only moved/appended: the current classic
perimeter materialization role enum cannot represent support roles and
must not be reused by falsely relabeling support paths as perimeters.
`SupportCommon.cpp:1594-1630` supplies five ascending layer lookups with
>= target-EPS and strict < target+EPS, preserving the first matching slot
even when its original polygons are empty. This is selection and merge
staging, not the full toolpath cache/consumer. Loop interface processing,
actual fill generation and output integration remain required.
Independent read-only review found no required correction. Final Clippy
is terminal0/15.00s, existing252 warnings, at
`/tmp/ares-normal-support-ard84-toolpath-cache-clippy-final.log`; fmt/diff
pass. New modules122/97 lines. Actual behavior has not passed the public
seam and no completion commit is approved.
Latest cache-tree paired-output run is terminal100/12.987s at
`/tmp/ares-normal-support-ard84-toolpath-cache-wip-output.log`: dependency
test PASS and ordinary-support boolean FAIL5.373s. Complete input/native/
Ares/error/manifest files remain in
`/tmp/ares-normal-support-ard84-toolpath-cache-wip-output`; enabled output
still differs at byte88/line3,101/75 layers. Reference cache producer
identity remains unknown. The cache-tree workspace has started separately
at `/tmp/ares-normal-support-ard84-toolpath-cache-workspace.log`; the prior
parameter snapshot does not cover these cache additions.

The cache snapshot finished terminal0/260.478s,7462 PASS/9 skip. It
excludes the subsequent contact-loop geometry and extrusion binding.
Those port `SupportCommon.cpp::LoopInterfaceProcessor:831-1058`,
`Point.hpp`'s radius lookup and `Polyline.cpp` split-point stitching.
Preserve rounded hexagon points, truncated ray points, four-cell lookup,
strict radius comparisons, cumulative circle insertion and original
contact polygons; only the separate remaining-to-extrude region changes.
The supported Boolean configuration enables zero or one loop; unreachable
multi-loop openings are not a replacement pipeline.

`Flow.hpp:96-100`, `Flow.cpp:200-206,219-231`,
`ExtrusionEntity.hpp:592-602` and `Polyline.hpp:296-301` now supply the
height-specific Flow and actual support-interface extrusion payload.
Reuse integer Point3/Polyline3 storage with Z=0 and empty fitting; keep
SupportMaterialInterface, can_reverse, volume, width and height. The caller
must supply nonbridging interface Flow, matching the source precondition.
This binding returns ordered paths plus remaining polygons for the next
cache consumer; contact merging, fill generation and public integration
are still open. No stage-only result earns ordinary-support parity.

Independent read-only review closed the new argument-count warning after
grouping the two geometry outputs; no further static must-fix was found.
Final binding Clippy is terminal0/45.23s with existing252 warnings at
`/tmp/ares-normal-support-ard84-loop-extrusions-clippy-final.log`; fmt and
diff checks pass. An earlier public run failed to launch Orca (127) and
is retained at `/tmp/ares-normal-support-ard84-interface-loops-wip-output`;
it is environment evidence, not a G-code comparison.
The subsequent real paired run is terminal100/12.909s at
`/tmp/ares-normal-support-ard84-loop-extrusions-wip-output.log`: dependency
contexts PASS; ordinary-support Boolean FAIL5.323s, byte88/line3,
101/75 layers and normalized264791/180867 bytes. All paired files remain
under `/tmp/ares-normal-support-ard84-loop-extrusions-wip-output`.
Reference-cache producer provenance is still unknown. Ordinary support
remains RED and the unconnected WIP is not approved for commit.

Next rewrite `SupportCommon.cpp:1619-1658` into
`support_material/toolpath_cache/contacts.rs`, consuming the existing
source-indexed caches and loop extrusion binding. Included: source raft
contact classification, raw top/bottom count zero predicates, merge versus
move assignment, ironing footprint capture before loops, and ordered
interface/contact merges for ordinary grid support. Caller supplies the
actual per-layer interface angle, not a guessed constant. Preserve source
indices after merge (source polygons become empty), while move assignment
clears the source cache. Organic keepout trimming and the actual ironing
and fill toolpaths are deferred. These helpers remain a temporary
compatibility shell until the public support generator invokes them; no
compile result or gated default test can satisfy the complete-output gate.

The prior loop-binding workspace is now terminal0/260.861s,7462 PASS/
9 skip (36 slow) at
`/tmp/ares-normal-support-ard84-loop-extrusions-workspace.log`; it excludes
the contact-cache addition. Independent review of the new merge slice
found three new lint warnings; flattened only the relevant conditions,
preserving short-circuit order, and replaced unwrap with the same-selection
index invariant. Re-review closed all three findings, with no further
static must-fix. Final Clippy is terminal0/39.44s, existing252 warnings,
at `/tmp/ares-normal-support-ard84-contact-cache-clippy-final.log`.
New module125 lines; fmt/diff checks pass.
The new public paired run is terminal100/12.917s, dependencies11 contexts
PASS12.916s and ordinary Boolean FAIL5.358s at
`/tmp/ares-normal-support-ard84-contact-cache-wip-output.log`.
Its enabled `case-3DVhH2` retains complete input/native/Ares/error/manifest
files; ordered bytes remain different at byte88/line3,101/75 layers,
normalized264791/180867 bytes. Ares raw output SHA256 remains
caf49d07be7dd48b43b2b28da82b8c4e221e00935f5716201f204d0dbc7c3e3c,
consistent with no public invocation of the new helper. Cache producer
provenance is unknown; this evidence does not establish global inventory
completion or the new helper's runtime correctness. Do not commit staging.
Next actual interface fill must follow SupportCommon.cpp:1693-1762,
including bottom-interface density, per-layer extrusion Flow versus fixed
filler spacing, raft contact selection and base-interface material roles.
The source generator's bbox is the fixed symmetric [-1,1] mm box at
SupportCommon.cpp:1434, not the physical model bbox; do not change its
reference origin when wiring the already-shared filler implementation.
The cache-specific workspace run has started separately at
`/tmp/ares-normal-support-ard84-contact-cache-workspace.log`; collect its
terminal result instead of crediting the prior loop-binding snapshot.

Next interface-fill rewrite ports SupportCommon.cpp:1693-1762 into
support_material/toolpath_cache/interfaces.rs. Invoke the existing shared
Fill implementations (currently behind raft/fills.rs compatibility names)
with safety-offset regions, source fresh-filler defaults and origin zero.
Keep actual support-layer vector index separate from SupportLayer::id.
Include top/bottom/interface/base-interface calls in source order, raw
raft top-zero filler override, bottom-interface density and real material
roles. Preserve nominal filler spacing while extrusion Flow adopts actual
layer height; bridging Flow uses that height as diameter and the bottom
interface nozzle. SupportParameters.hpp:119-135,147-175,284-285 supplies
raft density/pattern/angles; Flow.hpp:106 and Flow.cpp:213-231 supplies
bridge spacing and volume. Share the ExtrusionEntity.hpp:592-602 path
payload between loops and fills rather than copying its conversion.
Defer base/flange/sheath generation, overlap modulation, ironing paths and
public input/output integration. No new private/source-pinning tests;
the existing public complete-output RED is the active TDD seam.

The preceding contact-cache workspace finished terminal0/257.551s,
7462 PASS/9 skip (36 slow); it excludes this interface-fill slice.
New modules236/54 lines reuse source Fill and path conversion rather than
introducing a new pipeline. Independent review found bridge-spacing
arithmetic must preserve double addition followed by f32 narrowing
(`BRIDGE_EXTRA_SPACING` is an unsuffixed double constant). Corrected to
`(f64::from(diameter) + 0.05) as f32`; re-review closed the finding with no
further static must-fix. Final Clippy is terminal0/44.92s, existing252
warnings, at `/tmp/ares-normal-support-ard84-interface-fill-clippy-final.log`;
fmt/diff checks pass. Top/bottom/interface/base-interface fills now invoke
the shared Fill implementation inside the staged cache; the public
generator still does not invoke this cache.
The actual paired output is terminal100/12.915s:11 dependencies PASS
12.914s, ordinary Boolean FAIL5.357s. Complete paired input/native/Ares/
error/manifest files are retained under
`/tmp/ares-normal-support-ard84-interface-fill-wip-output`, enabled
case-tZOcfX. Ordered output remains RED at byte88/line3,101/75 layers,
normalized264791/180867 bytes, unknown reference-cache producer. This is
not new helper runtime coverage, a finished support task, or a commit gate.
Continue source base/flange/sheath calls (SupportCommon.cpp:1764-1816)
with real entity collections and no_sort/oriented semantics; do not
flatten collection ordering into a vector of freely reversible paths.
The interface-fill workspace run has started at
`/tmp/ares-normal-support-ard84-interface-fill-workspace.log`; its terminal
result must be collected separately from the previous cache snapshot.

Next base-fill slice ports SupportCommon.cpp:1764-1816 and its
fill_expolygons_with_sheath_generate_paths:705-752 into
support_material/toolpath_cache/base.rs. Share the existing raft Fill
implementation only where it implements that exact upstream helper.
Extend the staged extrusion payload to source Path/Oriented and NoSort
collection semantics; ordered collections must survive cache moves/merges.
Use actual SupportFlows.first_layer for the bottom_z<EPS flange branch,
nominal base spacing otherwise, source density/angle/first-filler selection,
and base-interface merge after generation. Preserve the native fill-block
range beginning and n_raft_layers factory predicates explicitly, not a
guessed pattern for every first-layer shape. Expose the existing typed
SupportFlows to this actual consumer; no new independent flow model.
Included scope remains ordinary grid. Organic hollow trees, subsequent
ordering/overlap modulation, ironing and public integration are deferred.
Test only at the already-established public bytes-to-complete-G-code seam;
unconnected cache behavior cannot be counted as GREEN or committed.

The following ordering slice ports SupportCommon.cpp:1532-1547,1818-1858
into support_material/toolpath_cache/ordering.rs. Keep original cache
enumeration bottom/top/interface/base-interface/base, stable decreasing
height order, original layer polygons for island union, and source overlap
collection scans before stable print_z/descending-height/bridging order
(SupportLayer.hpp::operator<). Represent source-layer references with
indices into owned geometry; capture overlapping references before any
extrusions are moved. Return actual source cache entries, islands and
overlaps for the downstream modulation consumer, not invented public
pipeline data. Modulation, ironing and public integration remain deferred;
the existing public complete-output RED remains the TDD gate.

Collected interface-fill workspace terminal0/256.588s:7462 PASS/9 skip,
36 slow. This snapshot excludes base/entity/ordering changes. Base-fill
Clippy terminal0/14.86s,252 existing warnings. Its public paired run is
terminal100/13.179s:dependency probes PASS13.178s, ordinary Boolean
FAIL5.499s, first difference byte88/line3,101/75 layers,264791/180867
normalized bytes. Full input/native/Ares/error/manifest artifacts remain
under /tmp/ares-normal-support-ard84-base-fill-wip-output,case-2snuYf;
the reference-cache producer is unknown. No public cache runtime credit.

Independent six-axis review identified a shared sheath Flow conversion
error relevant to the new caller: f32 metric multiplied by a double
reciprocal differs from upstream double division before coord truncation
(Flow.hpp:62,69 and libslic3r.h:93). Changed only SheathFlow::new to divide
by scale.factor(); re-review closed that finding. No private arithmetic
or source-pinning test was added. Public complete-output verification is
still required; no broadened parity claim follows from static arithmetic.
Ordering re-review found no static must-fix. Its final Clippy is
terminal0/15.00s with252 existing warnings, at
/tmp/ares-normal-support-ard84-ordering-clippy-final.log; the earlier
ordering compile failure is not a passing result. fmt/diff checks pass.
Fresh ordering workspace and paired-output runs were started separately
at /tmp/ares-normal-support-ard84-ordering-workspace.log and
/tmp/ares-normal-support-ard84-ordering-wip-output.log. Collect their own
terminal results; neither is the interface snapshot nor the base snapshot.
Source ordering is staged only: overlap modulation must preserve connected
ExtrusionMultiPath fragments (SupportCommon.cpp:1084-1328), then ironing
and the actual public support input/output consumer must be implemented.
Ordinary-support completion and commit/push remain gated on full output.

The fresh ordering paired run finished terminal100/29.073s:dependency
probes PASS29.071s, ordinary Boolean FAIL11.421s. Complete artifacts at
/tmp/ares-normal-support-ard84-ordering-wip-output retain the unchanged
byte88/line3,101/75 layers,264791/180867 normalized-byte divergence;
reference-cache producer remains unknown. This run includes the shared
sheath division repair but does not invoke the staged ordinary-support
cache. The ordering workspace remains live and needs terminal collection.

Next modulation slice ports SupportCommon.cpp:1084-1328 and the
1872-1878 caller into support_material/overlap_modulation.rs with
separate fragment chaining/endpoint lookup modules. Consume actual
ordered overlaps, offset unioned overlap regions by half template width,
clip highest overlap first, and derive non-bridging reduced-height Flow.
Keep original endpoint order and source radius7 lookup; consumed fragments
are ignored. Preserve the exact compatible-height/mm3 join rule, connected
ExtrusionMultiPath identity, and source remainder append behavior. The
straight-path invariant is trusted as in native; do not flatten NoSort
collections as an invented fallback. This source-owned modulation is an
internal staged caller until the public driver is connected; ironing and
actual object/support G-code layer scheduling remain deferred. Public
complete-output RED is the sole parity gate; do not add private pinning
tests or count static/helper checks as completed output.

Ordering workspace collected terminal0/266.941s,7462 PASS/9 skip,
35 slow. It excludes modulation. New modulation modules104/92/83 lines
preserve source highest-first clipping, reduced non-bridging spacing/flow,
ordered original endpoint chaining and connected MultiPath entities;
OrderedLayerCache::modulate_and_append now calls that implementation.
The public generator still does not call the ordinary-support cache.
Radius7 uses shift5, four cells, strict squared-distance<49 and consumed
fragment endpoints ignored. Equivalent-key candidate order follows this
GNU reference's newest-first unordered_multimap behavior (PointHash is a
fast hash, gcc15 hashtable_policy small-size threshold0). This is the
Linux reference profile, not evidence of other native STL tie ordering.

Independent six-axis review found one source omission in compatible
fragment joins: Polyline3::append (Polyline.cpp:789-824) skips the new
first point when it equals the remaining last point after the caller's
pop. Repaired only that adjacent first-point rule; no global deduplication.
Re-review closed the finding; it is a static source discrepancy, not a
publicly witnessed cause of the current support divergence. Final repaired
Clippy terminal0/31.22s,252 existing warnings, at
/tmp/ares-normal-support-ard84-modulation-append-clippy-final.log.
fmt/diff checks pass. Earlier255/253-warning passes are not the quality
gate. Modulation workspace terminal0/258.414s,7462 PASS/9 skip,36 slow
excludes the adjacent-append repair; collect the fresh repaired workspace
separately from /tmp/ares-normal-support-ard84-modulation-append-workspace.log.

Repaired modulation paired output terminal100/32.543s:11 dependency
contexts PASS32.541s, ordinary Boolean FAIL13.461s. All input/native/Ares/
error/manifest files remain under
/tmp/ares-normal-support-ard84-modulation-append-wip-output,case-9ADg52.
Output remains RED byte88/line3,101/75 layers,264791/180867 normalized
bytes; reference producer identity remains unknown for runner-cache data.
This does not exercise the staged modulator. The preceding unrepaired
modulation paired snapshot is separate at
/tmp/ares-normal-support-ard84-modulation-wip-output,case-gFzLT9,
terminal100/13.048s; do not credit it as repaired coverage.
A repaired wasm32 core compile check was started separately at
/tmp/ares-normal-support-ard84-modulation-append-wasm-check.log; collect its
terminal result and do not equate a compile check with browser execution.
Next source boundary is SupportCommon.cpp:1881-1914 support ironing and
SupportParameters.hpp:65-68 ironing Flow/spacing/pattern, followed by
the source generate_support_toolpaths driver and actual public geometry /
G-code consumers. Full printer/default/option parity is still incomplete.

Collected repaired wasm32 check terminal0/1m27s; this proves compilation
only, not browser runtime or support output parity. The repaired workspace
is still running; keep its specific process/log rather than restarting on
an observation timeout.

Next ironing rewrite ports SupportCommon.cpp:1881-1914 and
SupportParameters.hpp:65-68 into support_material/ironing.rs. Use captured
top-contact polygons/angle, safety union then subtraction of the immediate
upper support islands when its bottom_z<=print_z+EPSILON. Reuse actual
FillRectilinear and FillConcentric implementations with source layer id,
spacing narrowed to float, link length truncated to scaled coord, fresh
filler defaults, density1 and dont_adjust. Emit actual erIroning path
metadata with original (pre top-zero override) interface Flow height
multiplied in double by0.01 and configured percentage then narrowed to
float. Preserve native lazy flow evaluation per filled component, so an
empty post-trim region does not introduce a zero-flow error. Only the two
PrintConfig.cpp:6565-6575 declared ironing patterns are included here;
other Fill factory patterns remain outside this slice. Shared raft filler
names are temporary compatibility shells around cited native Fill code.
Public ordinary-support integration remains required, not optional;
unconnected paths do not satisfy the public output gate.

The following driver slice ports SupportCommon.cpp:1421-1530,1552-1914
into support_material/toolpaths.rs and toolpaths/raft.rs. Extend the
existing installed SupportLayer carrier with native support_islands and
support_fills, initially empty; do not create a competing support output
model. Consume actual installed/source geometry and saved slicing/flow/
interface-count inputs. Preserve the raft-prefix count and source flange/
base/interface fill branches, source first-filler factory context, all
geometry/cache preparation before modulation/ironing, and immediate upper
islands. Move entity payloads without flattening Oriented/NoSort/MultiPath.
This ordinary-grid source driver does not implement organic hollow-tree
branches or other styles. Its sequential execution is the portable source
rewrite of the per-layer work, not proof of native parallel partition or
public API runtime coverage. Existing raft Fill names remain temporary
compatibility shells. Actual public input adapter and G-code scheduling
remain required next; no source-helper completion commit is permitted.

Driver six-axis static review found a native raft-angle assignment omitted
from the rewrite: SupportParameters.hpp:147-163 initializes raft_angle_base
to zero, and the two/three-layer branch changes only first/interface angles.
Repaired that branch to retain zero. Re-review closed the finding; no other
determinate static must-fix was reported. Nonzero support_angle with two/
three raft layers and contact-column regions still needs public sensitivity
coverage after integration. No commit/completion approval was given.

Final angle-repaired Clippy terminal0/14.58s,252 existing warnings, at
/tmp/ares-normal-support-ard84-toolpaths-driver-angle-clippy.log. The added
raft helper argument warning is removed without a lint suppression.
fmt/diff checks pass; all support Rust modules are below400 LOC (maximum305),
with no include!/include_bytes! splitting. Angle-repaired wasm32 core check
terminal0/1m01s at
/tmp/ares-normal-support-ard84-toolpaths-driver-angle-wasm.log proves only
compilation, not browser runtime.

The pre-angle-repair paired run is terminal100/13.103s: dependency contexts
PASS13.102s, ordinary Boolean FAIL5.431s. Complete exported inputs/native/
Ares/errors/manifests remain under
/tmp/ares-normal-support-ard84-toolpaths-driver-wip-output (enabled case
case-hH7K47; disabled case-fi2Rl8). Enabled output still differs at byte88/
line3:101 versus75 layers,264791 versus180867 normalized bytes. The driver
is not yet invoked from the public generator, so this run does not exercise
it or cover the angle repair. Cached-reference producer identity remains
unproven. Angle-repaired workspace was started as a separate run/log at
/tmp/ares-normal-support-ard84-toolpaths-driver-angle-workspace.log; see
its collected terminal status below. The in-flight build was not counted
as success.

The required public adapter must preserve PrintObject::update_slicing_parameters
(PrintObject.cpp:3732-3739) with original object extruders, and the raw region
backup boundary PrintObjectSlice.cpp:852-864 / Layer.cpp:68-82 before surface
typing. The current planning shell discards slicing parameters, and the
avoid-crossing top-surface map is fill_surfaces, not support's typed slices.
Neither reconstructed merged/raw geometry nor dummy extruder lists may
substitute these source inputs. Carry real installed support entities to
GCode.cpp::collect_layers_to_print and actual role-specific motion emission;
matching the layer-count comment alone cannot satisfy complete output.

Collected angle-repaired workspace terminal0/255.828s:7462 PASS,9 skip,
35 slow, at the separate driver-angle-workspace log above. This regression
run includes the new driver/ironing code in the build, but public tests
still do not call that driver. It is not ordinary-support output GREEN.

The next carrier rewrite retains PrintObject::m_slicing_params
(Print.hpp:419, PrintObject.cpp:3732-3739) directly on PlannedPrintObject,
using the already computed original-extruder parameters, not a support-only
recomputation with empty extruders. Synthetic pre-existing geometry fixtures
must supply explicit test parameters; production must have no missing-state
fallback. The retained compensation object also owns LayerRegion::raw_slices
from Layer::backup_untyped_slices (Layer.cpp:68-82) after actual region/XY/
elephant-foot geometry and slice ordering, before surface typing. These
backups are per region/layer, distinct from Layer::lslices and fill surfaces.
Existing post-compensation/perimeter carriers are compatibility shells for
these native fields, not a separate support geometry pipeline. Actual
support adapter and G-code consumers remain required before completion.

Support input adaptation reads the real post-make_fills object at
fill_entities::PreparedPostFillEntities, matching PrintObjectSupportMaterial
and SupportMaterialInternal::has_bridging_extrusions (SupportMaterial.cpp:
1122-1171). Use typed region slices for Top projections, actual generated
perimeters/fills for bridging detection, and PrintRegion::bridging_height_avg
(PrintRegion.cpp:55-69). Preserve ExtrusionMultiPath/Loop::as_polyline's
shared endpoint rule when collecting perimeter geometry. The compiled
LayerRegion.cpp #if1 external-surface branch does not populate
unsupported_bridge_edges; its only assignment at853 is in #else. Empty
edges therefore reflect this source build, not a substituted detector.
Annotation volumes, other support styles and grid rotation remain explicit
unported boundaries; never silently supply empty annotations for real
enforcers/blockers. The adapter is not complete until its returned support
layers reach the native G-code schedule/motion consumer.

### Raw ToolOrdering skirt eligibility slice

Port ToolOrdering.cpp:137-148, 441-495, 646-657, 660-780 and
1407-1448 into gcode_emit/layers/skirt_schedule.rs. Collect raw object and
support print Z before GCode pair averaging. Merge raw Z by minimum +
EPSILON with midpoint representatives; retain actual object-extrusion and
support-extrusion presence, rather than treating every object layer as
has_object. Calculate the permitted skirt gap from all configured nozzle
maximum heights (zero means 0.75 nozzle diameter) and actual object
configured layer heights. Mark intermediate printing layers only when the
next layer would exceed that gap. GCode.cpp:4280-4332 and 4413-4417 own
the emitted-skirt count and last emitted Z: layer flow must not use the
previous physical support chunk. Existing ordered carriers remain temporary
representations of native entity collections. Multi-tool wiping overrides
and sequential multi-object skirt allocation remain deferred; this slice
does not claim complete ToolOrdering parity. The unchanged public normal
support full-output oracle is the test seam, revalidated RED in
/tmp/ares-normal-support-skirt-red.log (two failures, 6.156s).

Read-only review required two repairs: raw flags are assigned by nearest
representative Z after initialization (not by original group membership),
and global combined skirt emission uses the current actual entry/object,
not object0 presence. Both have been statically re-reviewed without new
must-fix findings. Finite count is the configured skirt height; infinite
count is unbounded, never capped to object-layer count. An unequal-height
multi-object public complete-output regression remains required.

Next rewrite boundary is GCode.cpp:3670-3758/4550-4575/5550 and
GCode/CoolingBuffer.cpp:316-333: retain LayerResult.cooling_buffer_flush
through pressure equalization and accumulate intervening non-raft support
G-code until an object/raft/final result flushes it. Rust destination is
gcode_emit/cooling.rs plus the existing layer result metadata boundary.
Current per-chunk cool_layer behavior is a temporary incompatible shell;
it calculates slowdown for object-only chunks instead of the native
accumulated support/object window. The unchanged ordinary-support public
oracle remains RED; diagnostic removal of M73 lines is only for locating
motion differences, never the comparison criterion. Broader multi-tool
CoolingBuffer grouping is adjacent deferred behavior, not grounds for
claiming complete printer/option coverage.

Skirt slice terminal checkpoint: final clippy exits0/47.28s with warnings,
WASM compile exits0/45.09s (compile only), fmt/check and diff/check exit0.
Final unchanged paired oracle exits100/6.211s, two failures; enabled ordinary
support artifacts are /tmp/ares-normal-support-skirt-final-output/case-musNLU.
Normalized native264791 versus Ares265063 bytes; first difference remains
byte514/line19 M73 R21 versus R23. Previous Ares266225 bytes included the
extra intermediate skirt. Full inputs, native/Ares output, manifests and
errors are preserved; cached native producer identity remains unproven.
Current Rust slice files are 322/107/91/376/389 LOC. Latest entire-workspace
test snapshot predates this slice (7460 PASS, two failures, nine skips);
this checkpoint is not a new full-workspace or all-platform runtime pass.
No commit/push: complete-output and unequal-height multi-object regression
gates remain open.

Cooling slice input contract: GCode.cpp:4557-4579 chooses the first object
layer across the native layer vector, otherwise the first support layer;
LayerResult.layer_id is that active Layer::id(), not m_layer_index. Preserve
this selection before instance print-order sorting in layers/cooling_result.rs.
Raft detection uses the first support layer's id below its retained actual
base+interface raft count (Slicing.hpp:42). Keep (layer_id, flush) together
when PressureEqualizer delays each result (PressureEqualizer.cpp:174-205).
CoolingState owns its byte buffer, appends all pending support results and
rewrites once at flush; all part-fan boundary placeholders in that window
must resolve without leaking into public output. No filesystem or platform
RNG is introduced. Public unchanged paired oracle was revalidated RED
6.255s in /tmp/ares-normal-support-cooling-red.log before implementation.

Full-workspace regression exposed two source-shell corrections required
before completion. PrintConfig.cpp:8829-8845 set_num_extruders expands
extruder vectors with Config.hpp:631-663 first-value semantics; raw retained
max_layer_height may be shorter than nozzle_diameter, so skirt gap reads
the effective expanded value, not unchecked raw index. CoolingBuffer::reset
(:34-44) initializes both fan values to -1; Rust uses None until flush.
The BBL preamble's manual M106/P2 lines are an old compatibility shell:
GCode.cpp:3273-3277 emits only M981 there; native CoolingBuffer owns the
initial M106/P2 state. Remove that duplicate shell when installing actual
unknown initial part/aux fan states. Keep machine.rs's actual print-start
disable-fan behavior (GCode.cpp:2822-2825) unchanged. Complete KSR regression
must prove this change, not a private command-count surrogate.

Cooling regression checkpoint (collected terminal results):
`/tmp/ares-normal-support-cooling-regression-workspace.log` reports 7462
tests run, 7460 passed, two failed, nine skipped (252.961s). The remaining
failures are the zero-height triangle typed-settings fixture and the tree
support header fixture; neither is hidden or changed to make this slice pass.
The separately enabled E2E selection in
`/tmp/ares-normal-support-cooling-regression-parity.log` reports 10 passed
and three failed (18.541s): complete KSR output and all nine CLI regressions
pass, but ordinary support, cooling/equalizer and raft-dependency complete
output comparisons remain divergent. Gated tests in the workspace run are
not evidence of native output parity. Current clippy exits zero with warnings;
WASM check exits zero (compile only), not a browser runtime validation.

Public precision case `case-nGf5hj` under the regression artifact root has
M106 S81 in both complete outputs at lines 1454, 1470 and 1499. This proves
the exercised native/Rust fan result, not that every numeric fan input passes.
Ordinary enabled case `case-FEb4YB` retains input.3mf, both complete G-code
files, error.txt and manifest.json. The first ordered comparison divergence
is byte 59988, line 2563; normalized sizes are native264791/Ares264907.
Its manifest explicitly leaves cached native producer identity unknown.

Read-only motion diagnosis identifies four extra Ares first segments at
support Z8.775/9.133/9.492/9.85: travel ends at X115.739 instead of X115.371,
then extrudes to X115.371 Y115.371 with E.01631. Filament length differs
1161.34/1161.41 mm. Removing M73 lines was diagnostic only; full ordered
output assertions are unchanged. The cause and its relationship to earlier
progress timing differences remain unproven. Do not trim these points.
Next diagnostic boundary is native GCode.cpp::extrude_support/_extrude
(6178-6232/6345 onward), Layer.cpp::simplify_support_entity_collection and
simplify_support_path (329-356), and SupportCommon.cpp sheath/modulation
generation. Compare entity kinds, can_reverse and points before/after
simplification and chaining, preserving untouched official controls. Rust
destinations remain support_material, path_simplification and motion/support;
no new Ares-owned slicing pipeline or production filesystem tracing is allowed.
No commit/push or all-printer/all-option completion is justified by this checkpoint.

Subsequent unchanged Boolean oracle recheck is terminal100/2.093s in
`/tmp/ares-normal-support-path-recheck.log`, reproducing byte59988/line2563
with native264791/Ares264907; complete artifacts are
`/tmp/ares-normal-support-path-recheck-output/case-q6DKCC`. Fmt and diff
checks exit zero. Independent read-only review confirms the checkpoint
matches both terminal logs without new document must-fix findings; it does
not approve parity or a completion commit. Probe entry to extrude_support,
then chaining and actual _extrude input before moving upstream to simplify,
modulation and sheath generation if the entry points already differ.

Read-only debugger boundary captures now narrow the four-segment defect.
The separate native executable is explicitly selected at
`/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2/bin/.orca-slicer-wrapped`,
SHA256 `8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
It is a diagnostic build, not the official baseline identity. GDB only reads
memory; no inferior memory or source is modified. Its complete fresh output
in `/tmp/ares-ard84-support-presimplify-native-output/plate_1.gcode` compares
equal to case-q6DKCC's saved native output after generator-line removal
(`cmp` exit0). This agreement does not certify every other cached case.

`/tmp/ares-ard84-support-presimplify-native.log` captures native
Layer::simplify_support_path entry and GCode::_extrude entry.
`/tmp/ares-ard84-support-presimplify-rust.log` captures Rust
path_simplification::simplify_polyline3 and motion::support::emit_path.
Both processes reach terminal state (native normal exit; Rust's unchanged
complete-output oracle fails). The four affected Rust paths already begin
with (5739329,5370819,0), or Y5370818 on the fourth, before simplification.
Native pre-simplification paths begin with (5370790,5370818,0), or Y5370817
on the fourth; subsequent prefixes agree, including the short intermediate
points simplification later removes on both sides. Rust emitted entity roles
are SupportMaterialInterface and can_reverse=true. Thus the extra leading
point is upstream of simplification and the motion emitter, not introduced
by either. Prior modulation/reconnection and interface-fill generation remain
candidate producers; their cause is not yet identified. Next probe must
compare those producer boundaries, not adjust simplification or crop output.
Temporary tagged GDB scripts/logs live only in /tmp; no production Rust
instrumentation, private arithmetic pinning test, or oracle substitution is added.

Further read-only fill-boundary captures predate overlap modulation.
`/tmp/ares-ard84-rust-fill-params.log` reads the actual returned `lines`
at interfaces.rs:226, immediately before append_paths: Interface(0),
Interface(1), Interface(2) path0 and Top(1) path1 already contain the extra
leading (5739329,...). It records SourceLayer identity, path ordinal and
InterfaceFillPlan. `/tmp/ares-ard84-native-fill-params.log` reads complete
FillSupportBase::fill_surface returns with matching affected prefixes but
without that leading point. Native filler layer_id is unset and z=0, not
the support layer identity: do not represent those fields as Z correspondence.
The SourceLayer labels strengthen Rust mapping but native per-layer identity
still needs a caller-level probe. Fresh native output from
`/tmp/ares-ard84-native-fill-params-output/plate_1.gcode` again compares equal
to saved case-q6DKCC except generator (cmp0). Both final capture runs reach
terminal states without debugger read exceptions; earlier optimized-out or
incorrect-source-line attempts are not evidence.

The exercised plans agree on spacing0.33707964420318604, angle
1.5707963705062866 and effective f32 density0.627615749835968.
`/tmp/ares-ard84-rust-fill-costs.log` reads the cached SupportConnectionCosts
at select_support_arches: low698203.53867580532,
high1074159.2902704696, very_high1611238.9354057044. Native return probes
read the same three static costs. The observed mismatch therefore occurs
inside or before fill connection, not first-call cost lifetime, support
overlap modulation, later reconnection, simplification or emission. This
does not prove every graph input or intermediate point agrees. Next compare
FillRectilinear.cpp::FillSupportBase::fill_surface/make_fill_lines and
FillBase.cpp::connect_base_support phase inputs with raft/fills.rs and
fill/base_support/{connect,extend,link,take}; preserve full output oracle.
No production repair, completion commit, or all-options claim is made here.

Input and initial-graph comparison checkpoint: read-only capture logs
`/tmp/ares-ard84-native-fill-input.log` and rust-fill-input.log each contain
eight selected angle90 surface contours. Complete outer point order and
holes agree as multisets; individual invocation order differs. Native
overlap=0, multiline=1 and anchor lengths1000/1000 were captured. This
does not prove every FillParams field or native caller-layer correspondence.

The stronger connect-function entry captures are
`/tmp/ares-ard84-native-connect-input.log` and rust-connect-input.log.
For the exercised effective density0.627615749835968, each contains54
calls and six unique complete captured inputs. Exact JSON multiset
comparison of spacing, density, integer bbox, all boundary polygons in
order and all infill lines in order returns equal; the retained result is
`/tmp/ares-ard84-connect-input-comparison.log`. Invocation order and per-layer
identity are not asserted equal. Fresh native complete output still matches
saved case-q6DKCC after generator-line removal (cmp0).

`/tmp/ares-ard84-native-graph-state.log` captures the return of native
create_boundary_infill_graph; rust-graph-state.log captures the Rust graph
before connect.rs:34 clones it. Both contain54 selected results. Exact
JSON multiset comparison in `/tmp/ares-ard84-graph-state-comparison.log`
matches full captured boundary points, cumulative contour parameters and
all intersection fields: contour/point indices, parameter, normalized
previous/next indices, remaining lengths, consumed and trim flags.
Native pointers are normalized relative to the intersection-vector start;
Rust Option indices are decoded from GDB values. Native vector stride64
matches ContourIntersectionPoint fields; no read exceptions occur in these
final runs. Native exits normally and Rust reaches unchanged full-output
failure. The native graph-state output again compares equal to saved native
except generator. Initial graph state is therefore not the observed source
of the extra leading points within this captured input set. Next inspect
FillBase.cpp:2263-2697 processing after graph construction against Rust
base_support/{mark,arches,extend,link,take,caps}; do not change graph building
or crop output merely to remove the symptom. Exact complete-output parity,
all printers/options and a completion commit remain unproven.

Post-graph source correction candidate: native midgraph2 and Rust midgraph
logs each contain54 pre-cost-selection captures, but their graph-state
multisets differ (/tmp/ares-ard84-midgraph-comparison.log). The native probe
uses this recorded binary's connect_base_support+0x2e1b, the first cost_low
guard load after arch evaluation; Rust probes connect.rs:95 before cached
cost selection. Native/Rust initial graph equality does not imply this
later state equality. Selected native points are extended and consumed;
Rust retains old endpoint indices and unconsumed remaining lengths.

Owning source boundary is FillBase.cpp:1338-1344
BoundaryInfillGraph::prev_vertical/next_vertical, used by
base_support_extend_infill_lines:1834-1948. Native compares point.x(),
because the fill lines are vertical in the connection frame. Rust
fill/base_support/extend.rs currently compares point.y() in both gates.
Correct only those two gates to compare x; retain all extension thresholds,
traversal, arch lengths, cost lifetime and downstream ordering. Existing
module is already the upstream rewrite destination; no new pipeline or
compatibility shell is added. No private source-shape/axis pinning test:
the agreed public normal-support Boolean complete-output oracle is the
regression seam, already RED in the current actual debugger-run test.
Re-run that strict oracle, cooling/raft dependencies, KSR, clippy, fmt,
WASM compile and workspace, then independent six-axis review. The two-line
source correction remains a candidate until full output verifies it.

Axis repair verified at the unchanged complete-output seam: extend.rs
now compares x in both vertical-neighbor gates. Nextest in
`/tmp/ares-normal-support-extension-axis-parity.log` is terminal100/6.301s,
three tests run, one PASS/two FAIL. The ordinary Boolean test PASS includes
the enabled full output in case-srINeH under the extension-axis artifact root.
The cooling test's baseline fan case-8UGoBY and f32 ramp case-sfCoZ4 also
have complete PASS manifests; equalizer-2 case-9zVRd1 remains DIVERGENT at
byte156726/line6772. The dependency test still fails automatic-width,
without-skirt and after-other-jobs cases. No assertion, reference bytes,
geometry cropping or progress-line filtering was used for these PASSes.
Cached producer identity remains explicitly unknown in manifests.

The rebuilt Rust pre-cost-selection capture in
`/tmp/ares-ard84-rust-midgraph-fixed.log` reaches a passing full-output test.
`/tmp/ares-ard84-midgraph-fixed-comparison.log` compares54 native and54
Rust captured graph states and reports exact multiset equality; the same
comparison before this two-line change was unequal. This supports the axis
fix as the cause of the exercised extra-segment defect. It is not a claim
of every extension branch or invocation pairing. Existing KSR complete
ordered output independently passes (18.381s, extension-axis-ksr.log).
Clippy/all-targets exits0/1m06s with existing warnings; WASM compile check
exits0/30.83s, not a browser runtime pass. Fmt and diff checks pass.
The new full-workspace handle54967 remains verified live compiling; do not
inherit an earlier full-workspace result or restart it due observation time.

Independent review statically accepts both x comparisons and identifies
another pre-existing extension mismatch to assess separately: the native
ternary at FillBase.cpp:1925 has no assignment in its true branch, whereas
Rust clears extend_prev there. The both-directions-eligible/prev-shorter
case needs actual public/native branch evidence before a separate repair.
Do not fold it into the axis repair or claim the entire extension port is
complete. Completion commit/push remains gated on final runtime review and
the unfinished dependent support slices; full all-printer/options work stays active.

### Pressure-equalizer interpolation-origin slice

The extension-axis workspace run is now terminal: 254.857s, 7462 tests,
7460 PASS/two FAIL/nine skipped. The failures remain the omitted-defaults
zero-height model and support-width fixture using unsupported tree support;
neither fixture has been weakened. Log: extension-axis-workspace.log in /tmp.

Fresh unchanged cooling full-output RED is reproducible in 3.157s at
`/tmp/ares-equalizer-fixed-origin-red-output/case-cf7SQn`: byte156726,
line6772, native X111.5/E.09952 versus Rust X112.324/E.12686.
Upstream ownership is PressureEqualizer.cpp::output_gcode_line:578-633,
specifically the original pos_start used for acceleration interpolation and
the steady endpoint. Rust destination is pressure_equalizer/emitter/split.rs
emit_accel_then_decel. Native advances line.pos_start after emitting each
piece, but retains the separate pos_start snapshot until deceleration.
Rust currently advances both. The narrow repair removes that extra snapshot
update, preserving all flow solving, precision, thresholds, formatter and
single-slope behavior. Those adjacent differences remain outside this repair.
This is an upstream rewrite correction, not a new Ares smoothing policy.
The existing public complete-output comparison is the regression seam; no
private/source-pinning assertion or output filtering is added.

Fixed-origin runtime is terminal, not GREEN despite its diagnostic filename:
`/tmp/ares-equalizer-fixed-origin-green.log`, 6.151s, three tests one PASS,
two FAIL. The equalizer case-QwlYNo first difference advances to byte164704,
line7158 (M73 position). Read-only diagnostic comparison excluding M73 also
shows an actual motion difference at Z10.4: extra near-zero acceleration
and trailing zero-extrusion pieces. This diagnostic does not replace the
strict test oracle. Independent review accepts the snapshot repair and
keeps overall completion RED; it also records repeated accel comments as a
separate existing gap requiring a comment-bearing public/native case.

Next narrow threshold slice cites PressureEqualizer.cpp:555-560: the peak
algorithm uses the minimum distance from peak to either endpoint. Because
the peak exceeds both endpoints, this is peak minus maximum endpoint, not
peak minus minimum endpoint as in the current Rust split.rs. Correct only
this endpoint selection; retain peak solving, original-feedrate handling,
precision and comments for separate evidence. Prediction: the nearly-zero
acceleration branch at Z10.4 selects the native single-slope fallback and
the two extra pieces disappear. Validate against the unchanged complete
cooling RED and its three scenarios, not an M73-filtered comparison.

Minimum-delta correction reaches strict GREEN: terminal parity run6741,
6.334s, three tests all PASS. All16 persisted scenario manifests under
`/tmp/ares-equalizer-minimum-delta-output` report PASS, including equalizer2,
automatic width, no skirt and after-other-jobs. Full inputs, native output,
Ares output and manifests are retained. Cached producer identity is still
unknown; this is not a claim of fresh official identity or all options.
The attempted KSR expression in that command selected only the orca_parity
binary, so it does NOT establish a KSR pass. A separate explicit
ksr_fdmtest_v4 test run is pending, together with current-source full
workspace93230, clippy25918 and WASM compile62407. Snapshot-only source
previously passed23 pressure-equalizer tests (20.324s), clippy0 (3m05s
including lock wait), fmt/diff0; do not inherit these as final-source gates.
Independent review statically accepts both emitter corrections and observes
the three full-output PASSes, but final gates and whole-batch review remain.

Final-source explicit KSR run57436 is now terminal PASS42.689s; clippy25918
exits0/2m53s including lock wait (260 lib warnings,276 lib-test warnings,
260 duplicates). WASM compile62407 exits0/29.97s; fmt/diff checks pass.
Workspace93230 remains verified live in test execution, not a completed gate.

A fresh separate reproducible native run61770 exits0 and retains complete
output in `/tmp/ares-equalizer-minimum-delta-native.4chDh1/plate_1.gcode`.
Input is saved case-a9O7Id/input.3mf (equalizer2). The executable is the
previously recorded seed-replay build, wrapped-binary SHA256
8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d.
Generator-only normalized cmp against current Ares output exits0; no M73
or motion filtering. This strengthens this case's reproducible producer
evidence but does not retroactively certify all cached manifests or claim
untouched-official provenance.

Whole-batch independent review still requires: resolve the two public
regressions using same-input native evidence, exercise unequal-height
multi-object skirt output, verify final browser ordinary support and four
platforms, and review an immutable complete patch with scope/status docs.
The two emitter corrections have no new mandatory static findings; comment,
original-feedrate precision and extension ternary remain explicit adjacent
deferred behavior. No scoped commit/push is approved yet, and all-printer/
all-option completion remains open.

### Typed-settings successful-slice fixture migration

Current-source workspace93230 is terminal: 255.914s,7462 tests,7460 PASS,
two FAIL,nine skipped. The original typed-settings failure is reproduced
with exact3112-byte input captured read-only at load_project, retained as
`/tmp/ares-zero-height-same-input.3mf`, SHA256
69a778fe118de759f8e48a3fda6d5dbe51f3c571485408aeaff36b4eae15f964.
The successful capture log is zero-height-same-input-gdb3.log; earlier
optimized-out/symbol lookup attempts are not evidence. Original official
binary SHA64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60
exits139 on the same input. The separate reproducible native backtrace
shows CLI::run string assignment SIGSEGV before slicing (native-debug.log).
Neither result proves a native no-layers runtime verdict.

The owning source boundary is PrintObjectSlice.cpp:866-867: an empty layer
collection is a slicing error, not successful header-only output. Ares
already rejects this model, a single triangle with all vertex z=0. The
typed-settings success test had an invalid geometric premise. Independent
review approves replacing only its leaf geometry with a closed nonzero
tetrahedron, preserving all typed-default and successful output assertions.
Keep the shared ProjectParts::valid/LEAF_MODEL unchanged; add an independent
public slice_project zero-height rejection case with the original partial
settings. This replaces a temporary header-only-scaffold expectation, not
the native error contract. Retain original input/crash evidence and validate
the two public tests, then workspace; tree-support regression remains open.

The closed tetrahedron exposes a second genuine RED, not a migrated GREEN:
typed-settings-geometry-regression.log, three tests two PASS/one FAIL,
0.058s. Original flat rejection passes, but successful slicing panics at
combine_infill/process.rs:70 when an empty top layer-region has no perimeter
input. Preserve this geometric input rather than substituting an easier
model. Source PrintObject.cpp::combine_infill:4170-4174 skips regions whose
combination is disabled or density is zero before reading layer geometry.
Rust process::apply currently constructs every layer config first, even
when all actual regions disable combination. Port that source gate before
layer_configs, reading actual post-region options. Included: all-inactive
region no-op behavior, not synthetic records or defensive fallback. Deferred:
active combination on empty layer-region records needs its own source slice.
The same successful public slice_project test is the RED/GREEN seam; retain
all previous assertions and run existing combination plus full output gates.

Gate run37730 is terminal RED20.780s:17 tests16 PASS/one FAIL. Panic is
gone; all selected combination tests pass and tetra now returns the actual
empty-first-layer error. The shared import fixture has instance translation
Z30; tetra coordinates0..5 place its world bottom at30, not at the bed.
Keep that floating shape as a dedicated public regression returning an
error rather than panic. For the successful typed-settings case, translate
only its leaf vertices to z=-30..-25 so the unchanged instance transform
places the closed tetrahedron on the bed at0..5. No shared transforms,
settings assertions or production first-layer checks are changed.
GCode.cpp:1774-1778 owns the empty-first-layer rejection; automatic placement
or ModelObject::ensure_on_bed is not implemented as an implicit fallback.

Tree-width exact input capture is now successful in gdb4.log,181340 bytes,
SHA63a86b85a908e9fddd854c9beb1cc0d6ac6065427579e34a4adb4fff3ae6a561.
The original official executable slices it successfully (72924 exit0),
result.json return_code0 and complete plate_1.gcode retained under
`/tmp/ares-tree-width-native.tmQ9T4`. Output SHA256
c97985580d06469a5f7b0484b69dea251f0216ba283561d4c59b561f613dc662;
header widths are0.42,0.38,0.50mm in the asserted order, and actual
`; FEATURE: Support`/Support interface extrusion sections exist. Tree
behavior is a real unfinished rewrite, not a fixture to relabel as Grid.

Grounded/floating split closes the typed-settings regression: terminal
run70382,18 tests all PASS21.022s, including successful grounded partial
defaults/header output, flat no-layers rejection, floating empty-first-layer
rejection without panic and all selected existing combination tests.
Changed Rust files are141 and306 LOC. Independent review accepts the source
gate and the three distinct geometric premises without approving the batch.
Final current-source workspace64777, strict ordinary-support parity22844,
clippy39518 and WASM compile are pending; fmt/diff pass. Tree remains the
known full-workspace regression, not silently disabled. Its retained native
artifact folder now includes manifest.json/error.txt with explicit Ares
no-output status and original input/binary/output hashes.

Current-source WASM compile6240 is terminal0/5.74s (not browser runtime).
The current workspace64777 has entered actual7464-test execution; public
strict parity22844 and clippy39518 remain verified live, not completed.
Retain those exact handles and their grounded-* logs for final collection.

### Collected gates and native first-call cache investigation

The final grounded-source runs are terminal: workspace64777 ran7464 tests,
7463PASS/1FAIL/9skipped in269.202s; the remaining failure is the unchanged
Tree-width public regression. Clippy39518 exits0 in5m15s (260 library
warnings,276 library-test warnings including260 duplicates). WASM6240 is
compile-only PASS, not browser evidence. Parity22844 runs3 tests,2PASS/1FAIL
in17.038s: automatic-width and after-other-jobs are RED in the dependency
test. Earlier GREEN checkpoints do not establish a stable native oracle.

The old GREEN and current RED automatic-width inputs have identical content
in all nine ZIP entries; their archive timestamps differ. Their complete
Ares outputs are byte-identical. Native outputs differ at travel ordering
and subsequent feedrate, so this is not evidence that the Ares edit changed
those bytes. `FillBase.cpp:2464-2466` owns three independently guarded
first-call static costs; `Support/SupportCommon.cpp:1441-1515` fills raft
layers in a TBB parallel loop with differing base/interface densities.

Read-only entry/return probes capture cached costs515781.6944064484,
793510.29908684362,1190265.4486302654 and a full native output matching
Ares. An exit-only probe captures724203.515324892,1114159.2543459877,
1671238.8815189814 and a full output matching the unprobed failing native
variant. This strongly supports initialization order as the cause; entry
breakpoints perturb scheduling and are not an admissible stable baseline.
Eight new unmodified official runs all emit the latter variant: no claim
that these eight unprobed runs themselves demonstrate two variants.

Artifacts: `/tmp/ares-support-width-native-replay.2I8Zkh` retains both full
outputs, input and eight official repeat directories. Generator-line-only
removed SHA256 is9f668270e5e2192c1d51ba51819c623f339f52d2b07281e2be3e08d2a108fc9a
for run-1/exit-probe/all repeats and
2ef34a9a22b1e9c2180c5c78ae8ad1dda0838ebfaa64bfb0b33a48322677102b
for probe-output/Ares. Probe logs remain in
`/tmp/ares-support-width-native-costs.log` and
`/tmp/ares-support-width-native-exit.log`. Rust replay costs match the lower
tuple, but its actual test FAILS (0/1) against the unprobed native reference;
the debugger wrapper exit0 is not test success.

Seed control does not fix first-call cache scheduling. Permission for a
separate seeded/initialized native test build does not authorize changing
cache lifetime, threshold formulas or parallel scheduling. Do not select
a convenient variant, retry until PASS or relax movement-byte comparison.
Independent six-axis re-review is requested; commit/push remains gated.

The independent reviewer confirms the differing cached tuples/output
correlation and rejects calling this an unsynchronized data race or UB:
the three statics have separate initialization guards. No mixed tuple was
observed. An additional native determinization-policy question is pending.

Browser validation is now executing the current source, not the old
generated bindings. `cargo build -p ares-wasm --target wasm32-unknown-unknown`
with target-dir `/tmp/ares-lightning-ard83-target` exits0 in45.30s; temporary
web bindings SHA256 for WASM is
7f7e08f85e153f634efa221b4e9cf02231597c988e25092096972389292ee3e4.
Bindings are generated under `/tmp/ares-normal-support-browser-bindings`,
leaving existing repository browser assets untouched. A temporary public
binding harness intercepts fixture fetch with the exact retained contact-grid
input and will compare all ordered bytes except the generator line.
The fresh unmodified official same-input contact-grid run exits0 and matches
host Ares under that comparator; full native output SHA256 is
0dc91a95015d605abcb81c847f09b7718edbf5a02e66d8810adb43b90a2309dc,
retained under `/tmp/ares-normal-support-browser-native-contact-grid`.

Initial browser launch failed for missing shared libraries (log
`/tmp/ares-normal-support-browser-probe.log`), not a slicing verdict.
After providing the needed Nix runtime libraries, session35420 is verified
live with an actively executing Chromium renderer; the public slice promise
has not returned. Log `/tmp/ares-normal-support-browser-probe2.log` and
output folder `/tmp/ares-normal-support-browser-output.HYfrur` currently
retain exact input only. Do not count browser PASS, terminate on observation
timeout, or restart this live run. Collect this same handle next.

Session35420 is now terminal1, not PASS. The renderer disappeared; read-only
Node inspector captured the Page crash-listener stack (`Page._onCrash`,
page.js:100), with closed=false and closed/crashed scope closed=true. Browser
remained connected. Only after proving actual page crash was the crashed
test browser closed; controller then reports page.evaluate target-closed.
No G-code was returned. Evidence `/tmp/ares-browser-inspect2.log`, complete
controller log `/tmp/ares-normal-support-browser-probe2.log`; original input,
manifest.json and error.txt remain in the original HYfrur output directory.
Crash cause is unknown: no assertion of OOM, infinite loop or core defect.

Next diagnosis hypotheses: debug-WASM resource demand (compare identical
input using current-source release), browser-version-specific failure
(compare another installed Chromium only if needed), platform-specific
core behavior (requires persisted release failure and targeted evidence).
Current-source release build97023 is live. A temporary diagnostic harness
now records page crashes and errors explicitly and retains error/manifest
on failure; it does not alter core code or skip full movement comparison.

### Current-source release browser complete-output checkpoint

Release build97023 is terminal0 in3m01s; binding generation64368 exits0.
The new browser WASM SHA256 is
edcf99bb749a2a974b73502fcb70442423dd9193338ccf26adcfaa14cc0b87c9.
Chromium143.0.7499.4 runs the public `sliceProject` binding on each exact
retained input. Each reference is freshly sliced by the untouched official
binary SHA64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60,
not inferred from a cache hit. Full ordered byte comparison excludes only
the generator line; no motion, time, feedrate, cooling or metadata fields
are removed. Cached host build identity remains explicitly unknown, and
the fresh browser-versus-official comparison is independently performed.

All four browser/controller runs exit0 with full output saved:

- contact-grid: session4846,113080 bytes, output folder
  `/tmp/ares-normal-support-browser-output.G0FRih`.
- enable_support=true: session47087,264798 bytes, output folder
  `/tmp/ares-normal-support-browser-output.4rVJDm`.
- enable_support=false: session59940,180830 bytes, output folder
  `/tmp/ares-normal-support-browser-output.JE2ub6`.
- aux-role-fan/equalizer-2/ramp-6-100: session2689,269711 bytes,
  `/tmp/ares-normal-support-browser-output.nua4GZ`.

Each folder retains input.3mf/browser.gcode/orca.gcode/manifest.json with
input, WASM, native executable and output hashes. Native result.json and
complete command logs remain in the corresponding
`/tmp/ares-normal-support-browser-native-*` directories/logs. Explicit
ordinary-support witness: enabled official output has2415 positive
relative-E support XY moves; disabled output has0 support role sections.
This is not merely a raft-only or compile-only browser check.

The original debug page crash remains recorded and unexplained. Release
success distinguishes the observed profile behavior, not proof of a crash
cause or a debug fix. These four inputs also do not close all-printer,
all-option, Tree, multi-object or Windows/macOS runtime requirements.
Read-only crates audit finds3105 Rust files, none over400 LOC. `include!`
matches are test source strings/checker error messages, not source splitting.
Independent six-axis re-review requested before any commit/push approval.

Independent review separately recomputes all four browser/native/input
hashes and generator-only byte comparisons and accepts the limited release
browser checkpoint. It does not approve the combined commit or global
goal: Tree, unequal-height multi-object skirt, native reference cache policy,
final platform evidence and full inventory remain distinct gates.

Current-source platform compile checks are now collected: Windows GNU
`cargo check --workspace --target x86_64-pc-windows-gnu` session13502 exits0
in23.96s; macOS `cargo check --workspace --target x86_64-apple-darwin`
session65391 exits0 in46.58s (including the shared target lock wait).
Logs `/tmp/ares-normal-support-final-windows-check.log` and
`/tmp/ares-normal-support-final-macos-check.log` each show the four-crate
workspace check, with5 core warnings. This is cross-compilation checking
only, not native Windows/macOS execution. `cargo fmt --check` and
`git diff --check` session18987 exit0. These terminal gates do not erase
the workspace Tree FAIL, automatic-width parity FAIL or debug browser crash.

Unequal-height skirt verification uses two distinct bed-grounded public
model objects, not two islands inside one object. The source owner is
ToolOrdering::mark_skirt_layers/calc_max_layer_height plus GCode skirt_done
(matching spec gives exact spans). Reuse the existing rewrite destinations
and public complete-output comparator. Fresh by-layer4mm/10mm reference
must remain50 skirt-bearing global layers even after the short object ends.
No production change is authorized by a guessed fixture outcome, and no
skirt-only comparison can satisfy this gate. Sequential/multi-tool behavior
remains deferred. Preserve complete output on RED as well as GREEN.

Public multi-object runs73923/95652 are terminal100, respectively0.818s/
0.833s. Both official outputs have50 skirt layers; Ares reports
shared_mesh_centering and no G-code. First shared-leaf artifact case-wwGc7e
is retained under `/tmp/ares-multi-object-skirt-output`. Independent-leaf
artifact case-M7j1GT under `/tmp/ares-multi-object-skirt-independent-output`
still has the same error, proving filename separation alone is insufficient.
The preflight incorrectly uses request-global numeric volume IDs, while
bbs_3mf.cpp:744 and4867-4890 use `(model part path,local id)` resources.
Independent review agrees to the minimal tuple-key fix using existing
source_model_path normalization. No fixture ID renumbering or native-output
normalization. Explicit mesh_shared and same-resource repetition remain
declared deferred capability, not an assertion that native input is invalid.

Resource-key repair now uses `(volume.source_model_path(),volume.id())`
with borrowed path strings; no ID rewriting/extra path normalization.
Two old private shared-resource rejection fixtures now use genuinely equal
part paths and IDs. Related preflight/lifecycle run92997 exits0:
21/21PASS0.786s. Whole public run28425 exits100/0.956s, but now produces
complete Ares output instead of rejecting independent resources. Artifact
`/tmp/ares-multi-object-skirt-resource-key-output/case-P6JSh5` retains full
input/native/Ares/error/manifest: Ares166073 bytes SHA
148fa48978b44020bfb6627724a14bc94f9e79bde0e6583ae495758c865dea89;
native165922 bytes SHA
42a000d095628a41f90d7de87fcb66b1d7edc9d0fbf1585914c4f14178eeb6e8.
The fixture's loaded two grounded heights and native50 skirt-bearing layers
pass, but strict whole output remains DIVERGENT (first byte1101/line39,
M73 P2 R12 versus P3 R11). No M73 stripping or relaxed acceptance.

Diagnostic motion-only diff (not acceptance) additionally exposes first
skirt: native extends from X90 throughX130, Ares onlyX90 throughX100.
Wipe paths also shift incorrectly by30mm when changing objects.
`Print.cpp:2794-2803` adds every `PrintInstance::shift` to local occupied
hull points; current skirt.rs collects local points without those per-object
plate offsets. `GCode.cpp:5659-5669::set_origin` rebases m_last_pos and wipe
path; current layers.rs changes origin/offset fields without that translation.
These are the next source-owned rewrite boundaries, not reasons to ignore
time/movement differences or count this multi-object gate GREEN.

Clippy77713 exits0 in3m04s,276 lib-test warnings including260 duplicates;
fmt/diff35176 exits0. Changed/new Rust files185/371/395 LOC. Full workspace
31371 is actually running after this repair; log
`/tmp/ares-multi-object-resource-workspace.log`. Collect that same handle.
Earlier platform/browser checkpoints precede this resource-key change and
are not final-snapshot evidence for the expanded patch. No commit/push.

Independent six-axis review accepts the tuple-key repair but requests
explicit public fixture witnesses. The195LOC test now verifies distinct
leaf paths, local ID1 in both and actual instance XY95/125,110 in addition
to grounded4/10mm heights. Rerun39473 exits100/2.795s, case-2qnoVz under
`/tmp/ares-multi-object-skirt-reviewed-fixture-output`; every fixture/native
skirt witness passes, strict whole output remains the same RED. No changed
option, model ID or comparator was used to bypass it.

Workspace31371 is now terminal100:7465 tests,7464PASS/1FAIL/9skipped,
37slow in255.101s. The sole failure remains original Tree-width. The
environment-gated multi-object case is not activated by that ordinary
workspace command; its separate explicitly activated whole-output run is
RED and must not be hidden behind the workspace aggregate. Workspace was
compiled before the final fixture-only extra witnesses; core production
is identical, and39473 separately checks the expanded test. Fresh Clippy
61158 exits0/1.99s (CLI parity30warnings,9duplicates; no diagnostic naming
the new test module); fmt/diff69266 exit0 after those fixture changes.
All handles launched in this checkpoint are terminal. Commit/push gate
remains open until public multi-object full output and other original
requirements are actually verified.

### Object-origin transition checkpoint

The fresh public baseline38826 again fails whole output in1.012s at
generator-normalized byte1101; case-VZYrRC under
`/tmp/ares-multi-object-origin-red-output` preserves both complete streams.
Port `GCode.cpp:5659-5669::set_origin` into the27-line motion/origin.rs:
translate local scaled cursor and local wipe storage, retaining absolute
writer X/Y and wipe_start. `Point.hpp:280` rounds the double translation.
The direct layers assignment and its now-unused context extruder-offset
field are replaced, without changing RNG or native cache policy.

First build51766 is terminal101 for visibility errors, not a runtime RED;
visibility is corrected before run84024. Run84024 is terminal100 with
genuine complete-output RED0.982s, case-ergIos under
`/tmp/ares-multi-object-origin-rebase-output`. InputSHA remains
c25524af86d95069a9e5919c35f0091ecd2b6b26d2fbfb360cad5ad9deda5738.
AresSHA ec7868e41e4f2036c495caee963a18498d41f12312640769ecd5e79aeebe3421;
nativeSHA63ae2a7d6ef6d08b01a8f2149bb4181cb7880c870ffd2ecc2af34347a42884fc.
The previously30mm-misplaced wipe endpointsX91.444/91.594 now match
nativeX121.444/121.594; this diagnostic is not the acceptance criterion.
Combined skirt hull remains incorrect and complete-output parity is open.

Regression44636 is terminal0 in3.165s: both public test functions pass,
covering five complete-output cases (supportfalse/true, equalizer0 with
ramps11-45 and6-100, equalizer2 with ramp6-100). Each fresh native/control
and Ares stream is retained under
`/tmp/ares-origin-normal-support-regression-output`. Clippy53116 exits0
in58.74s with existing276 core-lib-test warnings (260duplicates);
fmt/diffcheck terminal0. Source sizes are origin27, motion378, layers379,
context15; state remains399. Independent six-axis review and full workspace
68966 are pending at this checkpoint. No commit/push or global completion.

Independent six-axis review subsequently finds no new confirmed mandatory
fix for the origin-transition prerequisite, but does not approve aggregate
completion or commit. Fractional sub-lattice transitions, nonzero extruder
offsets and every scale are not claimed covered by these finite cases.
Workspace68966 terminates100 in90.636s:5700/7465 run,5699PASS,1TreeFAIL,
9skipped and1765not-run due to fail-fast. This is not a full-workspace PASS.
An explicit no-fail-fast full-workspace run is launched afterward; retain
its separate `/tmp/ares-origin-rebase-workspace-all.log` and live handle
until authoritative completion, without replacing the Tree fixture.

Current origin-rebase snapshot cross-checks are terminal0: WindowsGNU
workspace87409 in12.79s, macOSx86_64 workspace31042 in24.26s (including
build-lock wait), WASM core/adapter38606 in22.98s (including lock wait).
Logs are `/tmp/ares-origin-rebase-{windows,macos,wasm}-check.log`.
These are compile-only evidence, not Windows/macOS runtime or browser
full-output parity. Earlier release-browser output results belong to the
earlier source snapshot and are not promoted to current runtime evidence.

No-fail-fast workspace75926 is now terminal100 in254.439s: all7465 run,
7464PASS (35slow),1FAIL,9skipped. The only failure remains the original
Tree-width public test with UnsupportedProjectFeature support_type/style.
Unlike68966, no tests are omitted by fail-fast. This does not activate the
reference-gated multi-object test, whose separate complete-output RED still
blocks the skirt slice. All handles launched for this origin checkpoint
are terminal; preserve all logs and stream manifests for the next hull
rewrite and independent re-review. Commit/push remains unapproved.

### Combined skirt coordinate contract

The next source-owned rewrite slice uses Print::_make_skirt's per-object
local hull followed by per-instance integer translation and combined-group
hull (`Print.cpp:2669-2696,2794-2803,2842-2845,2973-2985`). SkirtPlan stores
plate scaled integers and emits after source set_origin(0,0); it must not
interpret those points in whichever object's origin remained from the
previous layer. Object paths keep their own local frame. First-layer bounds
combine translated instance contours, plate skirt bounds and the existing
translated first-object brim shell; no later model-center addition.
Functional occupied and first_layer modules keep each Rust source sub400.
Adjacent per-object grouping, tower geometry and full multi-object brim
remain unclaimed. This is not a new Ares-owned pipeline or RNG policy.

### Combined skirt witnessed GREEN and regression evidence

Fresh baseline27440 is terminal100/0.985s, case-z5XQ2b under
`/tmp/ares-multi-object-hull-red-output`. After the plate-hull rewrite,
public87788 is terminal0/0.978s, case-zA1CwX under
`/tmp/ares-multi-object-plate-hull-output`. The test/input/settings/activity
assertions and generator-only comparator are unchanged. InputSHA remains
c25524af86d95069a9e5919c35f0091ecd2b6b26d2fbfb360cad5ad9deda5738;
complete Ares165916bytesSHA
b9b83b33bc43af91740753570902a8f2b62f10bd05a92ecd46e5c6cca77a65d2;
native165922bytesSHA
8a0be83c0f27397d72b8499414b8b3f61371414ff5d9723440e8a664edb0999d.
Independent review rereads both full streams and confirms only the
generator line differs, as well as input identity and source coordinate
contracts. This bounded GREEN does not establish every skirt option.

The first Clippy65096 exits0/1m15s with278core-lib-test warnings
(262duplicates), including the newly orphaned model_center helper. Delete
that helper only after both initial runs terminate. Final Clippy24884 exits
0/47.69s with277warnings(261duplicates); existing crate-wide lint debt
remains, and the new first-layer nesting warning is not a -Dwarnings PASS.
Fmt/diff checks exit0; source sizes skirt253,occupied141,footprint280,
first_layer114. No include-based source split or structural pinning test.

Five public ordinary-support/cooling cases53477 all pass complete output
in3.173s under `/tmp/ares-plate-hull-normal-support-output`. Original raft
dependency test31652 is terminal100/18.205s: nine cases PASS and the two
automatic-width lifecycle cases still DIVERGENT. Full streams retained at
`/tmp/ares-plate-hull-raft-dependencies-output`; case-70dYG6 Ares/native
generator-normalized hashes exactly match the previously diagnosed lower/
higher variants (2ef34a9a...77102b /9f668270...fc9a), proving no new output
change for this witness. This is not a test PASS or permission to alter the
native static-cache policy. The policy question remains unanswered.

Full no-fail-fast workspace83469 exits100/261.512s: all7465run,7464PASS
(36slow),1originalTree-widthFAIL,9skip. Reference-gated CLI cases require
their separate explicit runs. WindowsGNU95206 exits0/13.64s and macOSx86_64
63771 exits0/25.11s including lock wait: compile-only, not native runtime.

Current release WASM3035 exits0/4m38s; bindgen terminates0 in0.966s,
bindings `/tmp/ares-multi-object-plate-browser-bindings.p3UgDw`, WASMSHA
86edd8eaa96ea854315ece7120d7a3fac5accf29a21a2200bef11079151de8d0.
Fresh untouched official27519 exits0/result Success in
`/tmp/ares-multi-object-plate-browser-native.y86CX1`. Browser86903 terminates
0: Chromium143.0.7499.4 produces165916bytes and compares complete output
to host and that fresh native run, except generator line. Retained input,
browser/native streams and manifest:
`/tmp/ares-normal-support-browser-output.KHpNNy`; inputSHA above,
browserSHA24fa580a9030db2c1efcfc126396b86630792aa233f93f6cf81eb172f59f363a,
nativeSHAc595f5fb5d28874f3c7188a223938bf8c63a0a3c575a287d2a48d25970c65947.
Host cache build identity in that temporary manifest stays explicitly
unknown; do not infer broader cache provenance. This is a release-browser
finite-case PASS, not a fix for the earlier debug-page crash.

Review additionally notes the pre-existing Skirt/Brim avoid-wall routing
skip in plan_route versus native generate_skirt's external motion planner.
Keep that adjacent behavior deferred, not silently covered by this case.
Independent final re-review, Tree/static-cache/full-matrix gates and the
containing task's commit/push remain open; no global completion.

Current repository-wide LOC audit counts3109 Rust files. No file exceeds
400, but one pre-existing file is exactly400:
`project/effective_config/phases.rs`. Therefore the user's stricter
every-file-below400 requirement is not yet proved globally; the four
changed skirt/footprint modules are individually below400. Do not turn
the no-over400 result into a strict sub400 global completion claim.

Final independent review confirms host/browser complete-output GREEN and
unchanged known raft variants but requests removal of the new first-layer
excessive-nesting warning. Replace only the nested layer/polygon loops with
the ordered selected_layers.flatten() iteration, without new helpers or
lint suppression. This lowers first_layer.rs to112lines. Launch separate
final-snapshot Clippy/public-output/no-fail-fast workspace/release WASM and
cross-compile runs; earlier results belong to the pre-flatten snapshot.
Do not call those new runs complete from intent or build-lock state.
