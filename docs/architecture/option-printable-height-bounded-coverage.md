# Printable-height bounded coverage: proposed rewrite-boundary witnesses

Status: selection approved after G2 `84f8fe274`; spec pending review.
This authorizes read-only analysis and this document only, not Native execution,
classification promotion or production edits. Worker3 owns this single key;
APA (worker1), G4 tool mappings (worker2), wrapping (worker6) remain excluded.

## Non-negotiable source and destination boundary

Pinned OrcaSlicer v2.4.2 source `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Upstream owner | Included behavior / exact Rust boundary |
| --- | --- |
| `PrintConfig.cpp:779-787` | `printable_height`, scalar coFloat/mm, min 0, max 214700, default 100; existing typed `PrinterRemainingOptions::printable_height` and loaded config validation |
| `PrintConfig.cpp:10530-10564` | Scalar range validation map; existing public loaded-validation API, no new validator |
| `BuildVolume.hpp:105-113`, `BuildVolume.cpp:377-421` | Rectangle object/bbox containment; `SceneEpsilon=EPSILON`, zero-height sentinel disables the scene ceiling; `project_slice/state/print_volume.rs::PrintVolume` / `count_inside_instances` |
| `Model.cpp:692-702,3423-3469`, `OrcaSlicer.cpp:5718-5726` | Instance print-volume classification; zero fully-inside instances exits -50 before apply; `project_slice/state.rs` existing unconditional gate |
| `Print.cpp:1354-1396::Print::validate` | Generated-layer upper Z check, EPSILON and diagnostic selection; existing validated project entry / `print/validate.rs`; any missing parity is reported before authorizing a rewrite |
| `Print.cpp:833-930` | Active model-index sequential order; final instance initially uses printable_height with strict limit < max_z rejection; existing `print/validate/sequential_clearance.rs` |
| `GCode.cpp:3045` (adjacent, deferred) | `max_print_height` placeholder is ConfigOptionInt, not floating; existing `project_slice/gcode_emit/placeholders.rs`; no new template witness in this slice |

`extruder_printable_height` is a different nullable vector: max 1000 at
`PrintConfig.cpp:788-796`. Do not apply its bounds to this scalar, conflate its
per-tool consumer, or grow G4 coverage. 214700 is the authored schema endpoint,
not certification that a printer can physically print that high.

This is a test-harness admission around the cited upstream owners, not an
Ares-designed pipeline. No new geometry engine, scalar validation registry,
public API, cache or platform-specific core behavior. `printable_height.rs`
legacy STL layer checking is a temporary scaffold, not the project/CLI oracle:
it has a separate tolerance and first-extruder policy. Do not retrofit it as a
fallback or use unchecked `slice_project` to test applied validation.

## Read-only findings and limits of evidence

The historical actual `acf100550` option audit lists this key INCOMPLETE despite
matching interior samples. Current probes retain `probe-min=50`, `probe-max=400`;
these do not prove a context or either schema boundary. There is no observed
runtime production disparity yet, and no production RED is manufactured.

Source inspection identifies two distinct consumers: BuildVolume treats zero
as an unbounded scene ceiling, while Print checks generated layers against
height + EPSILON. Current Rust scene code reads a literal max_z and the applied
validator does not visibly port that generated-layer branch. These are hypotheses
requiring Native-backed validated API/CLI evidence, not permission to patch core
or change diagnostics/classification. If a proposed control exposes a mismatch,
retain it and stop for coordinator ruling before any production edit.

`EPSILON`/float conversions are source behavior, not comparator allowances.
Record actual transformed mesh bounds, layer schedule and effective inputs before
interpreting a boundary. Do not silently round a requested height, change geometry,
raise defaults, compensate coordinates or compare a successful prefix/footer only.

## Declared finite contexts and candidates

Use the existing Ender-3 0.4 smoke export, one filament/nozzle, rectangular plate
1 at its existing origin, cube10 at world Z 0..10, ordinary by-layer sequence.
No supports/raft/skirt/draft shield/prime tower, no shrinkage compensation or
custom variable layers, traditional timelapse and existing M73 policy. Keep XY
comfortably inside; do not vary plate origins or reimplement worker4's XY gates.
Verify that effective configuration and transforms remain exactly requested.
All values below are candidates, not already observed success/rejection results.

### A. Canonical application and schema witnesses

Preserve both original requests/labels 50 and 400; add 214700 as a schema-ceiling
request on the same small cube. All three need full Native/Ares G-code equality
and exact application proof before being counted successful. This does not
claim universal feasibility throughout [0,214700]. Do not change the shared
application verifier or G2's four-key ownership to admit this key.

Independently request -0.00001 and 214700.00001, keeping all other values legal.
Retain Native loaded-validation map, export phase/status/channels/result; compare
the public Rust loaded map. No Native-generated project is fabricated if export
rejects. The expected range-map path is -18/exit238, not a sequential collision.

Zero is a separate schema-legal endpoint control: export/application must preserve
0, then classify the actual scene/apply/slice outcome. In particular, it must not
be declared globally invalid by copying G2's strictly-positive clearance domain.
Zero need not produce G-code to be accounted for, but a Rust/Native phase/diagnostic
difference blocks promotion pending owner review. Record schema acceptance and
print feasibility separately; no blanket rejection waiver.

### B. Rectangular scene and generated-layer witnesses

On the unchanged cube10, request heights 10, 10.0002, 9.99995 and 9.9998.
10/elevated/within-source-scene-epsilon are acceptance candidates; 9.9998 is a
no-fully-inside scene rejection candidate. Observe actual float conversion and
layer scheduling rather than inferring result solely from arithmetic. Successful
controls compare complete output; scene rejection compares validated API and
actual CLI -50/code206, channels, result and no-output inventory. A different
upstream phase is preserved and reported, not coerced to the proposed label.

The generated-layer check is an owning boundary to trace in these runs, not an
assertion that equality of mesh max_z proves last-layer containment. If these
fixed controls do not independently reach it, metadata explicitly defers its
last-layer-overrun/shrinkage diagnostic branches; no complete Print::validate
coverage claim. No speculative new adversarial schedule is added without review.

### C. Reuse of G2's printable-height dependency

G2 already proved its tall-last lid context at unchanged printable_height:
centers (60,60)/(60,90), objects 10mm then 20mm, rod40/lid10/radius10/nozzle2.5,
by-object, zero skirts and world Z0. Reuse that authored fixture/export/transform
recipe and existing sequential reference runner, not a second validation engine.
Do not rewrite or relabel the existing 14 G2 controls or their receipts.

Add only printable_height=20 and 19.99995 on that same verified scene. Equality
is an acceptance candidate; the lower request should retain scene containment
under its epsilon but activate the last-instance strict limit < max_z collision
(-63/code193). Use `ares_core::slice_project_plate_validated`, as production CLI
does. Rejections compare typed diagnostic and full CLI channels/result/no G-code;
acceptance compares the entire stream. If an earlier guard preempts this intended
consumer, report before changing the layout, requests or spec. G2's historical
receipt proves fixture precedent, not execution of these new requested heights.

## Harness boundary, RED/GREEN and exit gates

After spec approval, new test-only code belongs in
`ares-cli/tests/orca_parity/option_coverage/printable_height.rs` with a child file
if exceeding 400 LOC. Only minimal `domains.rs`/`tests.rs` wiring is anticipated.
No sweep dispatch is preauthorized; any scientific-serialization application
issue must be reported and justified key-locally first. Keep G2, bridge and
filament-flow dispatch/admission unchanged, and prove all other 649 plans equal
against the selected clean baseline. No independent Ares feature or legacy fallback.

1. Commit Native-backed RED: preserve all outcomes, compare successful full streams
   and separate loaded/scene/sequential rejections, then fail the current bounded
   classification assertion only if all required behavior witnesses match.
2. A genuine validated API/CLI mismatch is a production RED: stop and report raw
   evidence/source owner; this spec grants no core edit. No retry-to-PASS, dropped
   value, hidden skip or weakened comparator.
3. Harness-only GREEN, if warranted, admits only the explicitly witnessed finite
   context. Preserve a nonempty matched-but-unproved INCOMPLETE countercontrol,
   original labels, dependency metadata and inventory. Zero/global range/adjacent
   deferred behaviors must not be implied complete by sample equality.
4. Seeded/official cold directed and same-cache hit runs; required Native absence
   fails closed, optional absence visibly skips before artifact access. Hits are
   receipt reuse, not stability observations. Complete byte comparator ignores
   only whole generated-by lines; CLI result uses existing clock exemptions only.
5. Isolated-target fresh seeded/official BASE–CAND controls, exact raw failed-name
   subset (continue candidate after baseline test failures), all common/new/changed
   byte streams accounted for; no unapproved ND exemption. Workspace nextest,
   strict clippy/fmt/WASM, actual SHA/commit bindings and every failed attempt kept.

Before any Native/heavy launch inspect `${TMP_ROOT}/ares-coord-inbox.log` for unfinished
BROWSER-START; a browser reservation overrides low load. Directed permission and
load/thread budget must be confirmed before execution; full pairs require <30,
build8 and one owned heavy job. No Native work under this unapproved spec.

Deferred: nonrectangular/custom beds, XY origins/exclusions, arbitrary meshes,
nonzero Z origins, shrinkage, painted/custom/variable schedules and independently
forced last-layer overruns, post-GCodeProcessor multi-extruder height/path checks
(`GCodeProcessor.cpp:1951,1992`), extruder_printable_height, mixed tools/SEMM,
max_print_height template arithmetic, printer resizing/auto arrangement, SLA and UI. No schema-global physical safety
claim. Milestone integration requires coordinator REVIEW-REQUEST/APPROVED and
ordinary ff-only push; source/geometry/phase disparities remain blockers.

## Authorized zero-endpoint rewrite decision (RED 139a7d1dc)

- Native run `1efab6af-ee54-420f-9c80-19c3e4c23168` rejects the identical zero-height cube at generated-layer validation (-51/205); validated Ares and its CLI incorrectly reject earlier at scene admission (-50/206). Raw evidence remains `/tmp/worker3-printable-height-zero-red-v1-discrepancy.json`.
- Non-negotiable: `BuildVolume.cpp:377-421` zero is unbounded only for scene containment; `Print.cpp:1354-1396` still checks generated layer pairs against the literal printable height plus `libslic3r.h:52` EPSILON. Do not apply the sentinel to Print or sequential limits.
- Reuse the existing source-cited `project_slice::planning::plan_project`/layer pairs for Print validation, not mesh height, emitted G-code, legacy STL validation, or an independently designed schedule. New scene-height and generated-layer modules get minimal owner hooks; keep worker6 wrapping hooks intact.
- The demonstrated fixed, unpainted, no-raft/no-shrinkage request is the GREEN obligation. Adjacent painted, material shrinkage, forced-overrun and mixed-tool witnesses remain deferred; no extra coverage claim follows from shared planner reuse. Full verification queues after worker6; no integration before review.
